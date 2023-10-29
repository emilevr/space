#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
//#![forbid(unsafe_code)]

//! Space may be the final frontier, but it sure is annoying when you run out. 🖖
//!
//! This library can be used to analyze disk space usage, in one or more directory trees.
//!
//! # Description
//!
//! The *space-rs* library provides an efficient way to read one or more directory trees and the apparent
//! size of the contained files. See the [directory_item][`DirectoryItem`] struct.
//!
//! > **NOTE:** The *apparent size* of a file is the size of the file content in bytes, which is typically
//! > slighly less than the actual space based on allocated blocks on the disk. The larger the file the less
//! > significant the difference.
//!
//! > **NOTE:** Symbolic links will be listed but not followed.
//!
//! [`DirectoryItem`]: directory_item/struct.DirectoryItem.html
//!
//! # Links
//!
//! - See [the backlog][backlog].
//!
//! [backlog]: https://github.com/users/emilevr/projects/1

pub mod directory_item;
pub mod size;

#[cfg(test)]
mod test_directory_utils;

pub use directory_item::DirectoryItem;
pub use directory_item::DirectoryItemType;
pub mod rapid_arena;
pub use size::Size;
pub use size::SizeDisplayFormat;
