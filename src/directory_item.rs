//! Provides functionality to analyze disk space usage.

use crate::Size;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::{cmp::Ordering, fs, path::PathBuf, sync::Arc};

#[cfg(test)]
#[path = "./directory_item_test.rs"]
mod directory_item_test;

/// The supported directory item types.
#[derive(Debug, Eq, PartialEq)]
pub enum DirectoryItemType {
    /// A directory.
    Directory,
    /// A file.
    File,
    /// A symbolic link.
    SymbolicLink,
    /// The type could not be determined or is not supported.
    Unknown,
}

/// A directory item.
#[derive(Debug, Eq)]
pub struct DirectoryItem {
    /// The path to the item.
    pub path: Arc<PathBuf>,
    /// The item type.
    pub item_type: DirectoryItemType,
    /// The size in bytes.
    pub size_in_bytes: Size,
    /// If the item is a directory, it may also have child items.
    pub children: Vec<DirectoryItem>,
}

impl DirectoryItem {
    /// Builds one or more DirectoryItem trees.
    #[inline(always)]
    pub fn build(mut paths: Vec<Arc<PathBuf>>) -> Vec<DirectoryItem> {
        if !paths.is_empty() {
            paths.sort();
            paths.dedup();
        }

        let mut items = vec![];
        for path in paths {
            items.push(Self::from_root(&path));
        }

        items
    }

    fn from_failure(path: &Arc<PathBuf>) -> DirectoryItem {
        DirectoryItem {
            path: path.clone(),
            item_type: DirectoryItemType::Unknown,
            size_in_bytes: Size::default(),
            children: vec![],
        }
    }

    fn from_root(path: &Arc<PathBuf>) -> DirectoryItem {
        if let Ok(metadata) = fs::symlink_metadata(path.as_ref()) {
            if metadata.is_file() {
                Self::from_file_size(path, metadata.len())
            } else if metadata.is_symlink() {
                Self::from_link(path)
            } else {
                let mut item = Self::from_directory(path);
                item.path = path.clone();
                item
            }
        } else {
            Self::from_failure(path)
        }
    }

    #[inline(always)]
    fn from_file_size(path: &Arc<PathBuf>, size_in_bytes: u64) -> DirectoryItem {
        DirectoryItem {
            path: path.clone(),
            item_type: DirectoryItemType::File,
            size_in_bytes: Size::new(size_in_bytes),
            children: vec![],
        }
    }

    #[inline(always)]
    fn from_link(path: &Arc<PathBuf>) -> DirectoryItem {
        DirectoryItem {
            path: path.clone(),
            item_type: DirectoryItemType::SymbolicLink,
            size_in_bytes: Size::default(),
            children: vec![],
        }
    }

    fn from_directory(path: &Arc<PathBuf>) -> DirectoryItem {
        let children = {
            let mut children = Self::get_child_items(path);
            children.sort_by(|a, b| b.partial_cmp(a).unwrap());
            children
        };
        DirectoryItem {
            path: path.clone(),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::new(
                children
                    .iter()
                    .map(|child| child.size_in_bytes.get_value())
                    .sum(),
            ),
            children,
        }
    }

    fn get_child_items(path: &Arc<PathBuf>) -> Vec<DirectoryItem> {
        fs::read_dir(path.as_ref())
            .into_iter()
            .flatten()
            .par_bridge()
            .filter_map(|result| result.ok())
            .map(|entry| {
                let path = Arc::new(entry.path());
                if path.is_file() {
                    let size_in_bytes = match path.metadata() {
                        Ok(metadata) => metadata.len(),
                        _ => 0,
                    };
                    Self::from_file_size(&path, size_in_bytes)
                } else if path.is_symlink() {
                    Self::from_link(&path)
                } else {
                    Self::from_directory(&path)
                }
            })
            .collect()
    }

    /// Given the total size in bytes, returns the fraction of that total that his item uses.
    pub fn get_fraction(&self, total_size_in_bytes: u64) -> f32 {
        if total_size_in_bytes == 0 {
            0f32
        } else {
            self.size_in_bytes.get_value() as f32 / total_size_in_bytes as f32
        }
    }
}

impl Ord for DirectoryItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size_in_bytes.cmp(&other.size_in_bytes)
    }
}

impl PartialOrd for DirectoryItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DirectoryItem {
    fn eq(&self, other: &Self) -> bool {
        self.size_in_bytes == other.size_in_bytes
    }
}
