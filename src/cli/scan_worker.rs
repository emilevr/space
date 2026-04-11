use space_rs::{is_reparse_point, DirectoryItem, DirectoryItemType, Size};

#[cfg(test)]
#[path = "./scan_worker_test.rs"]
mod scan_worker_test;

pub(crate) type ScanSender = crossfire::MTx<crossfire::mpsc::List<ScanMessage>>;
pub(crate) type ScanReceiver = crossfire::Rx<crossfire::mpsc::List<ScanMessage>>;
use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

pub(crate) enum ScanMessage {
    Item(DirectoryItem),
    ChildItem(DirectoryItem),
    /// All immediate children of a descendant directory, sent as a single batch.
    /// `ancestor_path` identifies the parent directory (e.g. `["child"]` or
    /// `["child", "grandchild"]`).  This replaces per-file messages - one batch
    /// per directory instead of one message per file.
    DescendantBatch {
        ancestor_path: Vec<String>,
        children: Vec<DirectoryItem>,
    },
    /// The directory at `ancestor_path` could not be read (e.g. access denied).
    AccessDenied(Vec<String>),
    ChildScanComplete(String),
    Complete,
}

pub(crate) fn spawn_scan(
    mut paths: Vec<PathBuf>,
    should_exit: Arc<AtomicBool>,
    sender: ScanSender,
) {
    paths.sort();
    paths.dedup();

    thread::spawn(move || {
        scan_paths(paths, &should_exit, &sender);
    });
}

/// Spawns a rescan of a single directory, sending `DescendantBatch` messages
/// with `ancestor_segments` prepended to each `ancestor_path`.  Reuses the
/// same BFS infrastructure as the initial scan.  Sends `Complete` when done.
pub(crate) fn spawn_rescan(
    ancestor_segments: Vec<String>,
    path: PathBuf,
    should_exit: Arc<AtomicBool>,
    sender: ScanSender,
) {
    thread::spawn(move || {
        let _ = rescan_directory(&ancestor_segments, &path, &should_exit, &sender);
        let _ = sender.send(ScanMessage::Complete);
    });
}

fn rescan_directory(
    ancestor_segments: &[String],
    path: &std::path::Path,
    should_exit: &Arc<AtomicBool>,
    sender: &ScanSender,
) -> Result<(), ()> {
    let mut bfs_queue: VecDeque<(Vec<String>, PathBuf)> = VecDeque::new();
    bfs_queue.push_back((ancestor_segments.to_vec(), path.to_path_buf()));

    while !bfs_queue.is_empty() {
        if should_exit.load(Ordering::Relaxed) {
            break;
        }

        let current_level: Vec<_> = bfs_queue.drain(..).collect();
        let next_level = Mutex::new(Vec::new());
        let send_failed = AtomicBool::new(false);

        rayon::scope(|s| {
            for (ancestor_path, dir_path) in current_level {
                let sender = sender.clone();
                let should_exit = should_exit.clone();
                let next_level = &next_level;
                let send_failed = &send_failed;

                s.spawn(move |_| {
                    if should_exit.load(Ordering::Relaxed) || send_failed.load(Ordering::Relaxed) {
                        return;
                    }
                    if process_directory_bfs(
                        &ancestor_path,
                        &dir_path,
                        &should_exit,
                        &sender,
                        next_level,
                    )
                    .is_err()
                    {
                        send_failed.store(true, Ordering::Relaxed);
                    }
                });
            }
        });

        if send_failed.load(Ordering::Relaxed) {
            return Err(());
        }

        bfs_queue.extend(next_level.into_inner().unwrap());
    }

    Ok(())
}

fn scan_paths(paths: Vec<PathBuf>, should_exit: &Arc<AtomicBool>, sender: &ScanSender) {
    for path in &paths {
        if should_exit.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        if path.is_dir() {
            if send_directory_progressively(path, should_exit, sender).is_err() {
                return;
            }
        } else {
            let item = DirectoryItem::from_root(path, should_exit);
            if sender.send(ScanMessage::Item(item)).is_err() {
                return;
            }
        }
    }

    // Ignore send error - receiver may have been dropped.
    let _ = sender.send(ScanMessage::Complete);
}

fn send_directory_progressively(
    path: &PathBuf,
    should_exit: &Arc<AtomicBool>,
    sender: &ScanSender,
) -> Result<(), ()> {
    // Send the root as an empty shell.
    let root_item = DirectoryItem {
        path_segment: path.to_string_lossy().to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::default(),
        descendant_count: 0,
        children: vec![],
    };
    if sender.send(ScanMessage::Item(root_item)).is_err() {
        return Err(());
    }

    let entries: Vec<_> = match fs::read_dir(path) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => {
            let _ = sender.send(ScanMessage::AccessDenied(vec![]));
            return Ok(());
        }
    };

    // Phase 1: Send all root children immediately (breadth-first).
    // Files and symlinks are sent fully built; directories as empty shells.
    let mut bfs_queue: VecDeque<(Vec<String>, PathBuf)> = VecDeque::new();
    let mut root_child_names: Vec<String> = Vec::new();
    for entry in &entries {
        if should_exit.load(Ordering::Relaxed) {
            return Ok(());
        }
        let entry_path = entry.path();
        if !entry_path.is_symlink() && !is_reparse_point(&entry_path) && entry_path.is_dir() {
            let name = entry_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let shell = DirectoryItem {
                path_segment: name.clone(),
                item_type: DirectoryItemType::Directory,
                size_in_bytes: Size::default(),
                descendant_count: 0,
                children: vec![],
            };
            if sender.send(ScanMessage::ChildItem(shell)).is_err() {
                return Err(());
            }
            bfs_queue.push_back((vec![name.clone()], entry_path));
            root_child_names.push(name);
        } else {
            let child = DirectoryItem::build_subtree(&entry_path, should_exit);
            if sender.send(ScanMessage::ChildItem(child)).is_err() {
                return Err(());
            }
        }
    }

    // Phase 2: BFS - process directories level by level, each level in
    // parallel via rayon.  All directories at the same depth are discovered
    // before descending further, so sizes grow uniformly across the tree.
    while !bfs_queue.is_empty() {
        if should_exit.load(Ordering::Relaxed) {
            break;
        }

        let current_level: Vec<_> = bfs_queue.drain(..).collect();
        let next_level = Mutex::new(Vec::new());
        let send_failed = AtomicBool::new(false);

        rayon::scope(|s| {
            for (ancestor_path, dir_path) in current_level {
                let sender = sender.clone();
                let should_exit = should_exit.clone();
                let next_level = &next_level;
                let send_failed = &send_failed;

                s.spawn(move |_| {
                    if should_exit.load(Ordering::Relaxed) || send_failed.load(Ordering::Relaxed) {
                        return;
                    }
                    if process_directory_bfs(
                        &ancestor_path,
                        &dir_path,
                        &should_exit,
                        &sender,
                        next_level,
                    )
                    .is_err()
                    {
                        send_failed.store(true, Ordering::Relaxed);
                    }
                });
            }
        });

        if send_failed.load(Ordering::Relaxed) {
            return Err(());
        }

        bfs_queue.extend(next_level.into_inner().unwrap());
    }

    // Phase 3: Signal all root children as scan-complete.
    for name in root_child_names {
        sender
            .send(ScanMessage::ChildScanComplete(name))
            .map_err(|_| ())?;
    }

    Ok(())
}

/// Processes a single directory in the BFS: reads its entries, builds all
/// children (files fully built, subdirectories as empty shells), and sends
/// them as a single `DescendantBatch`.  Subdirectories are added to
/// `next_level` for the next BFS iteration.
fn process_directory_bfs(
    ancestor_path: &[String],
    dir_path: &std::path::Path,
    should_exit: &Arc<AtomicBool>,
    sender: &ScanSender,
    next_level: &Mutex<Vec<(Vec<String>, PathBuf)>>,
) -> Result<(), ()> {
    let entries: Vec<_> = match fs::read_dir(dir_path) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => {
            let _ = sender.send(ScanMessage::AccessDenied(ancestor_path.to_vec()));
            return Ok(());
        }
    };

    let mut children = Vec::with_capacity(entries.len());
    let mut subdirs = Vec::new();

    for entry in entries {
        if should_exit.load(Ordering::Relaxed) {
            break;
        }
        let path = entry.path();

        if !path.is_symlink() && !is_reparse_point(&path) && path.is_dir() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            // Directory: add empty shell to the batch.
            children.push(DirectoryItem {
                path_segment: name.clone(),
                item_type: DirectoryItemType::Directory,
                size_in_bytes: Size::default(),
                descendant_count: 0,
                children: vec![],
            });
            let mut child_ancestor = ancestor_path.to_vec();
            child_ancestor.push(name);
            subdirs.push((child_ancestor, path));
        } else {
            // File or symlink: build fully (instant) and add to the batch.
            children.push(DirectoryItem::build_subtree(&path, should_exit));
        }
    }

    // Send all children as a single batch.
    sender
        .send(ScanMessage::DescendantBatch {
            ancestor_path: ancestor_path.to_vec(),
            children,
        })
        .map_err(|_| ())?;

    // Add discovered subdirectories to the next BFS level.
    next_level.lock().unwrap().extend(subdirs);

    Ok(())
}
