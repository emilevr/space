//! Provides functionality to analyze disk space usage.

use crate::Size;
use rayon::{
    prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
};

#[cfg(test)]
#[path = "./directory_item_test.rs"]
mod directory_item_test;

const FILE_NAME_ERROR_VALUE: &str = "!error!";

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
    /// The last part of the path that ends with this item.
    pub path_segment: String,
    /// The item type.
    pub item_type: DirectoryItemType,
    /// The size in bytes.
    pub size_in_bytes: Size,
    /// If the item is a directory it may also have descendants.
    pub descendant_count: usize,
    /// If the item is a directory, it may also have child items.
    pub children: Vec<DirectoryItem>,
}

impl DirectoryItem {
    /// Builds one or more DirectoryItem trees.
    #[inline(always)]
    pub fn build(mut paths: Vec<PathBuf>) -> Vec<DirectoryItem> {
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

    #[inline(always)]
    fn from_root(path: &PathBuf) -> DirectoryItem {
        let mut item = if let Ok(metadata) = fs::symlink_metadata(path) {
            if metadata.is_file() {
                Self::from_file_size(path, metadata.len())
            } else if metadata.is_symlink() {
                Self::from_link(path)
            } else {
                Self::from_directory(path)
            }
        } else {
            Self::from_failure(path)
        };

        item.update_stats_from_descendant();

        item.path_segment = path.to_string_lossy().to_string();

        item
    }

    #[inline(always)]
    fn from_file_size(path: &Path, size_in_bytes: u64) -> DirectoryItem {
        DirectoryItem {
            path_segment: get_file_name_from_path(path),
            item_type: DirectoryItemType::File,
            size_in_bytes: Size::new(size_in_bytes),
            descendant_count: 0,
            children: vec![],
        }
    }

    #[inline(always)]
    fn from_link(path: &Path) -> DirectoryItem {
        DirectoryItem {
            path_segment: get_file_name_from_path(path),
            item_type: DirectoryItemType::SymbolicLink,
            size_in_bytes: Size::default(),
            descendant_count: 0,
            children: vec![],
        }
    }

    #[inline(always)]
    fn from_failure(path: &Path) -> DirectoryItem {
        DirectoryItem {
            path_segment: get_file_name_from_path(path),
            item_type: DirectoryItemType::Unknown,
            size_in_bytes: Size::default(),
            descendant_count: 0,
            children: vec![],
        }
    }

    #[inline(always)]
    fn from_directory(path: &Path) -> DirectoryItem {
        DirectoryItem {
            path_segment: get_file_name_from_path(path),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::default(),
            descendant_count: 0,
            children: Self::get_child_items(path),
        }
    }

    #[inline(always)]
    fn get_child_items(path: &Path) -> Vec<DirectoryItem> {
        let entries: Vec<_> = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return vec![Self::from_failure(path)], // TODO: report error
        }
        .filter_map(|result| match result {
            Ok(entry) => Some(entry),
            Err(_) => {
                // TODO: report the error
                None
            }
        })
        .collect();

        match entries.len() {
            0 => vec![],
            1 => {
                let path = &entries[0].path();
                if path.is_file() {
                    let size_in_bytes = match path.symlink_metadata() {
                        Ok(metadata) => metadata.len(),
                        _ => 0,
                    };
                    vec![Self::from_file_size(path, size_in_bytes)]
                } else if path.is_symlink() {
                    vec![Self::from_link(path)]
                } else {
                    vec![Self::from_directory(path)]
                }
            }
            _ => entries
                .par_iter()
                .map(|entry| {
                    let path = &entry.path();
                    if path.is_file() {
                        let size_in_bytes = match path.symlink_metadata() {
                            Ok(metadata) => metadata.len(),
                            _ => 0,
                        };
                        Self::from_file_size(path, size_in_bytes)
                    } else if path.is_symlink() {
                        Self::from_link(path)
                    } else {
                        Self::from_directory(path)
                    }
                })
                .collect(),
        }
    }

    /// Given the total size in bytes, returns the fraction of that total that his item uses.
    #[inline(always)]
    pub fn get_fraction(&self, total_size_in_bytes: u64) -> f32 {
        if total_size_in_bytes == 0 {
            0f32
        } else {
            self.size_in_bytes.get_value() as f32 / total_size_in_bytes as f32
        }
    }

    /// Updates all descendents
    #[inline(always)]
    fn update_stats_from_descendant(&mut self) {
        let child_count = self.children.len();
        if child_count == 0 {
            return;
        }

        // First we update our descendants, so their sizes will be correct.
        self.children.par_iter_mut().for_each(|child| {
            if !child.children.is_empty() {
                child.update_stats_from_descendant();
            }
        });

        // Then we sort.
        if child_count > 1 {
            self.children.par_sort_by(|a, b| a.cmp(b));
        }

        // Update our own count and size from our descendants' stats.
        let mut size_in_bytes = 0;
        let mut descendant_count = 0;
        self.children.iter().for_each(|child| {
            if child.item_type == DirectoryItemType::Directory {
                descendant_count += child.descendant_count;
            }
            descendant_count += 1;
            size_in_bytes += child.size_in_bytes.get_value();
        });

        self.descendant_count = descendant_count;
        self.size_in_bytes = Size::new(size_in_bytes);
    }
}

impl Ord for DirectoryItem {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        // We want the ordering to be descending by size, so we swap the operands.
        if self.size_in_bytes == other.size_in_bytes {
            self.path_segment.cmp(&other.path_segment)
        } else {
            other.size_in_bytes.cmp(&self.size_in_bytes)
        }
    }
}

impl PartialOrd for DirectoryItem {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DirectoryItem {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.size_in_bytes == other.size_in_bytes
    }
}

#[inline(always)]
fn get_file_name_from_path(path: &Path) -> String {
    match path.file_name() {
        Some(file_name) => file_name.to_string_lossy().to_string(),
        _ => FILE_NAME_ERROR_VALUE.to_string(),
    }
}
