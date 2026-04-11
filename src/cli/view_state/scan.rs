#[cfg(test)]
#[path = "scan_test.rs"]
mod scan_test;

#[cfg(test)]
#[path = "scan_child_test.rs"]
mod scan_child_test;

#[cfg(test)]
#[path = "scan_grandchild_test.rs"]
mod scan_grandchild_test;

#[cfg(test)]
#[path = "scan_grandchild_order_test.rs"]
mod scan_grandchild_order_test;

#[cfg(test)]
#[path = "scan_grandchild_resort_test.rs"]
mod scan_grandchild_resort_test;

#[cfg(test)]
#[path = "scan_spinner_test.rs"]
mod scan_spinner_test;

#[cfg(test)]
#[path = "scan_spinner_count_test.rs"]
mod scan_spinner_count_test;

#[cfg(test)]
#[path = "scan_sort_test.rs"]
mod scan_sort_test;

#[cfg(test)]
#[path = "scan_auto_expand_test.rs"]
mod scan_auto_expand_test;

use super::scan_helpers::{
    build_child_row, find_child_by_name, find_descendant_by_path, insert_child_into_root,
};
#[cfg(test)]
use super::scan_helpers::{insert_grandchild_into_parent, update_root_for_grandchild};
use super::ViewState;
use crate::cli::row_item::RowItem;
use space_rs::{DirectoryItem, DirectoryItemType, Size};
use std::{cell::RefCell, rc::Rc};

impl ViewState {
    pub(crate) fn add_scanned_item(&mut self, item: DirectoryItem) {
        self.total_size_in_bytes += item.size_in_bytes.get_value();

        let descendant_count = item.descendant_count;
        let row_item = RowItem::from_directory_item(&item, self.total_size_in_bytes, None, 0);
        row_item
            .borrow_mut()
            .update_tree_prefix(&String::default(), false);
        self.item_tree.push(row_item);
        self.total_items_in_tree += 1 + descendant_count;
        self.visible_rows_dirty = true;
    }

    pub(crate) fn add_scanned_child(&mut self, child_item: DirectoryItem) {
        let child_size = child_item.size_in_bytes.get_value();
        self.total_size_in_bytes += child_size;

        let Some(root) = self.item_tree.last().cloned() else {
            return;
        };

        let is_directory = child_item.item_type == DirectoryItemType::Directory;
        let child_row = build_child_row(&child_item, self.total_size_in_bytes, &root, is_directory);
        let child_descendant_count = child_item.descendant_count;

        insert_child_into_root(&root, child_row, child_size, child_descendant_count);

        self.total_items_in_tree += 1 + child_descendant_count;
        self.visible_rows_dirty = true;
    }

    #[cfg(test)]
    pub(crate) fn add_scanned_grandchild(&mut self, parent_name: &str, child_item: DirectoryItem) {
        let child_size = child_item.size_in_bytes.get_value();
        self.total_size_in_bytes += child_size;

        let Some(root) = self.item_tree.last().cloned() else {
            return;
        };
        let Some(parent_child) = find_child_by_name(&root, parent_name) else {
            return;
        };

        let is_directory = child_item.item_type == DirectoryItemType::Directory;
        let grandchild_row = RowItem::from_directory_item(
            &child_item,
            self.total_size_in_bytes,
            Some(Rc::downgrade(&parent_child)),
            0,
        );
        // Directory grandchildren will receive progressive children.
        if is_directory {
            grandchild_row.borrow_mut().is_scanning = true;
        }
        let child_descendant_count = child_item.descendant_count;

        insert_grandchild_into_parent(
            &parent_child,
            grandchild_row,
            child_size,
            child_descendant_count,
        );
        update_root_for_grandchild(&root, parent_name, child_size, child_descendant_count);

        self.total_items_in_tree += 1 + child_descendant_count;
        self.visible_rows_dirty = true;
    }

    /// Marks a directory as access-denied.  An empty `ancestor_path` targets
    /// the root item itself; otherwise the path is walked from the root.
    pub(crate) fn mark_access_denied(&mut self, ancestor_path: &[String]) {
        let Some(root) = self.item_tree.last().cloned() else {
            return;
        };
        let target = if ancestor_path.is_empty() {
            Some(root)
        } else {
            find_descendant_by_path(&root, ancestor_path)
        };
        if let Some(node) = target {
            let mut node_ref = node.borrow_mut();
            node_ref.access_denied = true;
            node_ref.is_scanning = false;
        }
        self.visible_rows_dirty = true;
    }

    pub(crate) fn mark_child_scan_complete(&mut self, name: &str) {
        let Some(root) = self.item_tree.last().cloned() else {
            return;
        };
        if let Some(child) = find_child_by_name(&root, name) {
            child.borrow_mut().is_scanning = false;
            self.visible_rows_dirty = true;
        }
    }

    /// Marks a descendant directory's scan as complete.  `ancestor_path` lists
    /// names from the root's child down to the completed directory.
    #[cfg(test)]
    pub(crate) fn mark_descendant_scan_complete(&mut self, ancestor_path: &[String]) {
        let Some(root) = self.item_tree.last().cloned() else {
            return;
        };
        if let Some(node) = find_descendant_by_path(&root, ancestor_path) {
            node.borrow_mut().is_scanning = false;
            self.visible_rows_dirty = true;
        }
    }

    /// Adds all immediate children of a descendant directory in one batch.
    /// During scanning, children are appended unsorted with no prefix updates
    /// or ancestor resorting - `sort_root_children` rebuilds everything at the
    /// end.  This makes per-batch processing O(n) instead of O(n² log n).
    pub(crate) fn add_scanned_descendant_batch(
        &mut self,
        ancestor_path: &[String],
        children: Vec<DirectoryItem>,
    ) {
        if children.is_empty() {
            return;
        }

        let Some(root) = self.item_tree.last().cloned() else {
            return;
        };
        let Some(parent) = find_descendant_by_path(&root, ancestor_path) else {
            return;
        };

        let mut total_size: u64 = 0;
        let mut total_descendants: usize = 0;

        {
            let mut parent_ref = parent.borrow_mut();
            for child_item in &children {
                let child_size = child_item.size_in_bytes.get_value();
                let child_descendant_count = child_item.descendant_count;
                self.total_size_in_bytes += child_size;
                total_size += child_size;
                total_descendants += 1 + child_descendant_count;

                let is_directory = child_item.item_type == DirectoryItemType::Directory;
                let child_row = RowItem::from_directory_item(
                    child_item,
                    self.total_size_in_bytes,
                    Some(Rc::downgrade(&parent)),
                    0,
                );
                if is_directory {
                    child_row.borrow_mut().is_scanning = true;
                }

                parent_ref.max_child_size = parent_ref.max_child_size.max(child_size);
                // Append without sorting or prefix updates - rebuilt at scan end.
                parent_ref.children.push(child_row);
            }
            parent_ref.has_children = true;
            let parent_size = parent_ref.size.get_value();
            parent_ref.size = Size::new(parent_size + total_size);
            parent_ref.descendant_count += total_descendants;
            // Batch received - clear scanning flag (derive_scanning_state
            // will re-set it if subdirectory children are pending).
            parent_ref.is_scanning = false;
        }

        // Propagate size up ancestors and update each parent's max_child_size.
        let mut current = root.clone();
        for name in ancestor_path {
            let Some(next) = find_child_by_name(&current, name) else {
                break;
            };
            // Skip the parent itself - already updated above.
            if !Rc::ptr_eq(&next, &parent) {
                let mut node_ref = next.borrow_mut();
                let old_size = node_ref.size.get_value();
                node_ref.size = Size::new(old_size + total_size);
                node_ref.descendant_count += total_descendants;
            }
            // Update current's max_child_size with next's (possibly grown) size.
            let next_size = next.borrow().size.get_value();
            let mut cur_ref = current.borrow_mut();
            cur_ref.max_child_size = cur_ref.max_child_size.max(next_size);
            drop(cur_ref);
            current = next;
        }
        // Update root size and its max_child_size.
        {
            let mut root_ref = root.borrow_mut();
            let old_size = root_ref.size.get_value();
            root_ref.size = Size::new(old_size + total_size);
            root_ref.descendant_count += total_descendants;
        }

        self.total_items_in_tree += total_descendants;
        self.visible_rows_dirty = true;
    }

    /// Derives `is_scanning` on parent nodes from their children.  Leaf-level
    /// directories have `is_scanning` set/cleared directly by scan messages;
    /// this method propagates that state upward so ancestors show the spinner
    /// whenever any descendant is still scanning.
    pub(crate) fn derive_scanning_state(&mut self) {
        for root in &self.item_tree {
            derive_scanning_recursive(root);
        }
    }

    pub(crate) fn sort_root_children(&mut self) {
        for root in &self.item_tree {
            // Sort children at every level (during scanning, children were
            // appended unsorted for speed).
            sort_children_recursive(root);
            // Rebuild tree prefixes from scratch.
            root.borrow_mut()
                .update_tree_prefix(&String::default(), false);
            // Clear all scanning flags.
            clear_scanning_recursive(root);
        }
        self.visible_rows_dirty = true;
    }

    /// Progressively expands tree levels to fill the visible height without
    /// causing scrolling.  Called after each scan drain.  Once expanding would
    /// exceed `visible_height`, sets `auto_expand_done` to skip future checks.
    pub(crate) fn try_auto_expand_to_fill(&mut self) {
        if self.auto_expand_done || self.visible_height == 0 {
            return;
        }

        let mut any_expanded = false;
        loop {
            let current_count = count_visible_items(&self.item_tree, self.size_threshold_fraction);
            if current_count >= self.visible_height {
                self.auto_expand_done = true;
                break;
            }

            let mut frontier: Vec<Rc<RefCell<RowItem>>> = Vec::new();
            let mut expansion_cost: usize = 0;
            for root in &self.item_tree {
                collect_expansion_frontier(
                    root,
                    self.size_threshold_fraction,
                    &mut frontier,
                    &mut expansion_cost,
                );
            }

            if frontier.is_empty() {
                // No expandable items yet - children may arrive in later BFS levels.
                break;
            }

            if current_count + expansion_cost > self.visible_height {
                // Expanding would overshoot.  With BFS this is permanent since
                // children only grow, so mark done.
                self.auto_expand_done = true;
                break;
            }

            for item in &frontier {
                item.borrow_mut().expanded = true;
            }
            any_expanded = true;
        }

        if any_expanded {
            // Rebuild tree prefixes so newly visible items have correct
            // indentation characters.
            for root in &self.item_tree {
                root.borrow_mut()
                    .update_tree_prefix(&String::default(), false);
            }
            self.visible_rows_dirty = true;
        }
    }

    pub(crate) fn recalculate_fractions(&mut self) {
        for item in &self.item_tree {
            item.borrow_mut().update_fraction(self.total_size_in_bytes);
        }
        // Root items have no parent to cache max_child_size on, so compute it
        // directly (typically just 1 root item).
        let max_root_size = self
            .item_tree
            .iter()
            .map(|i| i.borrow().size.get_value())
            .max()
            .unwrap_or(0);
        update_peer_fractions(&self.item_tree, max_root_size);
        self.visible_rows_dirty = true;
    }
}

/// Sets `peer_fraction` on a list of siblings using the provided `max_size`
/// (the largest sibling's size, cached on the parent as `max_child_size`).
/// Recurses into each sibling's children using their own `max_child_size`.
fn update_peer_fractions(siblings: &[Rc<RefCell<RowItem>>], max_size: u64) {
    for sibling in siblings {
        let mut sib_ref = sibling.borrow_mut();
        sib_ref.peer_fraction = if max_size == 0 {
            0.0
        } else {
            sib_ref.size.get_value() as f32 / max_size as f32
        };
        if !sib_ref.children.is_empty() {
            update_peer_fractions(&sib_ref.children, sib_ref.max_child_size);
        }
    }
}

/// Recursively sorts children at every level by descending size, then
/// ascending name (for equal sizes).
fn sort_children_recursive(item: &Rc<RefCell<RowItem>>) {
    let mut item_ref = item.borrow_mut();
    item_ref.children.sort_by(|a, b| {
        let a_ref = a.borrow();
        let b_ref = b.borrow();
        match b_ref.size.get_value().cmp(&a_ref.size.get_value()) {
            std::cmp::Ordering::Equal => a_ref.path_segment.cmp(&b_ref.path_segment),
            ord => ord,
        }
    });
    for child in &item_ref.children {
        sort_children_recursive(child);
    }
}

fn clear_scanning_recursive(item: &Rc<RefCell<RowItem>>) {
    let mut item_ref = item.borrow_mut();
    item_ref.is_scanning = false;
    item_ref.scanning_child_count = 0;
    for child in &item_ref.children {
        clear_scanning_recursive(child);
    }
}

/// Counts items that would be displayed given the current expansion state,
/// skipping items below the size threshold.
fn count_visible_items(item_tree: &[Rc<RefCell<RowItem>>], size_threshold_fraction: f32) -> usize {
    let mut count = 0;
    for root in item_tree {
        count += count_visible_recursive(root, size_threshold_fraction);
    }
    count
}

fn count_visible_recursive(item: &Rc<RefCell<RowItem>>, threshold: f32) -> usize {
    let item_ref = item.borrow();
    if item_ref.incl_fraction < threshold {
        return 0;
    }
    let mut count = 1;
    if item_ref.expanded {
        for child in &item_ref.children {
            count += count_visible_recursive(child, threshold);
        }
    }
    count
}

/// Collects collapsed items with children whose parents are expanded (the
/// "expansion frontier").  `cost` accumulates the total number of children
/// that would become visible if every frontier item were expanded.
fn collect_expansion_frontier(
    item: &Rc<RefCell<RowItem>>,
    threshold: f32,
    frontier: &mut Vec<Rc<RefCell<RowItem>>>,
    cost: &mut usize,
) {
    let item_ref = item.borrow();
    if item_ref.incl_fraction < threshold {
        return;
    }
    if !item_ref.expanded {
        if item_ref.has_children {
            let child_count = if threshold == 0.0 {
                item_ref.children.len()
            } else {
                item_ref
                    .children
                    .iter()
                    .filter(|c| c.borrow().incl_fraction >= threshold)
                    .count()
            };
            if child_count > 0 {
                drop(item_ref);
                frontier.push(item.clone());
                *cost += child_count;
            }
        }
        return;
    }
    for child in &item_ref.children {
        collect_expansion_frontier(child, threshold, frontier, cost);
    }
}

/// Recursively derives `is_scanning` on non-leaf nodes: a node is scanning
/// if it has `is_scanning` set directly (leaf) OR any child is scanning.
fn derive_scanning_recursive(item: &Rc<RefCell<RowItem>>) -> bool {
    let children_scanning = {
        let item_ref = item.borrow();
        item_ref.children.iter().any(derive_scanning_recursive)
    };
    let mut item_ref = item.borrow_mut();
    if children_scanning {
        item_ref.is_scanning = true;
    }
    item_ref.is_scanning
}
