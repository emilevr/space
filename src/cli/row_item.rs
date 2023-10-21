use space_rs::{DirectoryItem, DirectoryItemType, Size};
use std::{
    cell::RefCell,
    ffi::OsStr,
    fmt::Display,
    path::PathBuf,
    rc::{Rc, Weak},
    sync::Arc,
};

#[cfg(test)]
#[path = "./row_item_test.rs"]
mod row_item_test;

pub(crate) enum RowItemType {
    Directory,
    File,
    SymbolicLink,
    Unknown,
}

pub(crate) struct RowItem {
    pub size: Size,
    pub has_children: bool,
    pub expanded: bool,
    pub tree_prefix: String,
    pub item_type: RowItemType,
    pub name: String,
    pub incl_fraction: f32,
    pub path: Arc<PathBuf>,
    pub children: Vec<Rc<RefCell<RowItem>>>,
    pub parent: Option<Weak<RefCell<RowItem>>>,
    pub row_index: usize,
}

impl RowItem {
    pub fn from_directory_item(
        dir_item: &DirectoryItem,
        total_size_in_bytes: u64,
        parent: Option<Weak<RefCell<RowItem>>>,
        parent_row_index: usize,
    ) -> Rc<RefCell<RowItem>> {
        let has_children = !dir_item.children.is_empty();

        let current_row_index = if parent.is_some() {
            parent_row_index + 1
        } else {
            0
        };

        let current = Rc::new(RefCell::new(RowItem {
            size: dir_item.size_in_bytes.clone(),
            has_children,
            expanded: has_children,
            tree_prefix: String::default(),
            item_type: match dir_item.item_type {
                DirectoryItemType::Directory => RowItemType::Directory,
                DirectoryItemType::File => RowItemType::File,
                DirectoryItemType::SymbolicLink => RowItemType::SymbolicLink,
                DirectoryItemType::Unknown => RowItemType::Unknown,
            },
            name: if current_row_index == 0 {
                dir_item.path.display().to_string()
            } else {
                dir_item
                    .path
                    .file_name()
                    .unwrap_or(OsStr::new("<error>"))
                    .to_str()
                    .unwrap_or("<error>")
                    .to_string()
            },
            incl_fraction: dir_item.get_fraction(total_size_in_bytes),
            path: dir_item.path.clone(),
            children: vec![],
            parent,
            row_index: current_row_index,
        }));

        if has_children {
            let mut children = vec![];
            let child_count = dir_item.children.len();
            for i in 0..child_count {
                let child = &dir_item.children[i];
                children.push(Self::from_directory_item(
                    child,
                    total_size_in_bytes,
                    Some(Rc::downgrade(&current)),
                    current_row_index,
                ));
            }
            current.borrow_mut().children = children;
        }

        current
    }

    pub fn update_tree_prefix(&mut self, parent_tree_prefix: &str, is_last_child: bool) {
        let mut parent_tree_prefix = parent_tree_prefix.replace("├ ", "│ ");
        let child_count = self.children.len();

        self.tree_prefix = parent_tree_prefix.clone();
        if is_last_child {
            self.tree_prefix.push_str("└─");
            if child_count > 0 {
                parent_tree_prefix.push(' ');
            }
        } else {
            self.tree_prefix.push('─');
        }

        parent_tree_prefix.push(' ');

        match child_count {
            0 => {
                self.tree_prefix.push('─');
            }
            _ => {
                self.tree_prefix.push('┬');
                for (index, child) in self.children.iter().enumerate() {
                    let child_is_last_child = index == self.children.len() - 1;
                    let mut child_tree_prefix = parent_tree_prefix.clone();
                    if !child_is_last_child {
                        child_tree_prefix.push('├');
                    }
                    child
                        .borrow_mut()
                        .update_tree_prefix(&child_tree_prefix, child_is_last_child);
                }
            }
        }
    }

    pub fn collapse_all_children(&mut self) {
        if self.has_children {
            for child in &self.children {
                let mut child = child.borrow_mut();
                if child.has_children {
                    child.expanded = false;
                    child.collapse_all_children();
                }
            }
        }
    }

    pub fn expand_all_children(&mut self) {
        if self.has_children {
            if self.expanded {
                for child in &self.children {
                    let mut child = child.borrow_mut();
                    if child.has_children {
                        child.expanded = true;
                        child.expand_all_children();
                    }
                }
            } else {
                self.expanded = true;
                self.collapse_all_children();
            }
        }
    }
}

impl Display for RowItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} with {} children",
            self.path.display(),
            self.children.len()
        )?;
        Ok(())
    }
}
