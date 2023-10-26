//! Provides functionality to analyze disk space usage.

use crate::Size;
use rayon::{
    prelude::{ParallelBridge, ParallelIterator},
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
    /// If the item is a directory it may also have child items.
    pub child_count: usize,
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
        item.path_segment = path.to_string_lossy().to_string();
        item
    }

    #[inline(always)]
    fn from_file_size(path: &Path, size_in_bytes: u64) -> DirectoryItem {
        DirectoryItem {
            path_segment: get_file_name_from_path(path),
            item_type: DirectoryItemType::File,
            size_in_bytes: Size::new(size_in_bytes),
            child_count: 0,
            children: vec![],
        }
    }

    #[inline(always)]
    fn from_link(path: &Path) -> DirectoryItem {
        DirectoryItem {
            path_segment: get_file_name_from_path(path),
            item_type: DirectoryItemType::SymbolicLink,
            size_in_bytes: Size::default(),
            child_count: 0,
            children: vec![],
        }
    }

    fn from_failure(path: &Path) -> DirectoryItem {
        DirectoryItem {
            path_segment: get_file_name_from_path(path),
            item_type: DirectoryItemType::Unknown,
            size_in_bytes: Size::default(),
            child_count: 0,
            children: vec![],
        }
    }

    fn from_directory(path: &Path) -> DirectoryItem {
        let children = {
            let mut children = Self::get_child_items(path);
            children.par_sort_by(|a, b| b.partial_cmp(a).unwrap());
            children
        };
        DirectoryItem {
            path_segment: get_file_name_from_path(path),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::new(
                children
                    .iter()
                    .map(|child| child.size_in_bytes.get_value())
                    .sum(),
            ),
            child_count: 0,
            children,
        }
    }

    fn get_child_items(path: &Path) -> Vec<DirectoryItem> {
        fs::read_dir(path)
            .into_iter()
            .flatten()
            .par_bridge()
            .filter_map(|result| result.ok())
            .map(|entry| {
                let path = &entry.path();
                if path.is_file() {
                    let size_in_bytes = match path.metadata() {
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

fn get_file_name_from_path(path: &Path) -> String {
    match path.file_name() {
        Some(file_name) => file_name.to_string_lossy().to_string(),
        _ => FILE_NAME_ERROR_VALUE.to_string(),
    }
}
