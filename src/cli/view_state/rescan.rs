use super::ViewState;
use crate::cli::row_item::{RowItem, RowItemType};
use space_rs::Size;
use std::{cell::RefCell, rc::Rc};

impl ViewState {
    /// Prepares a rescan of the currently selected directory.  Clears the
    /// item's children, subtracts its old size from ancestors and
    /// `total_size_in_bytes`, sets scanning flags, and stores a
    /// `rescan_request` for the render loop to pick up.
    pub(crate) fn prepare_rescan(&mut self) {
        let Some(selected) = self.get_selected_item() else {
            return;
        };
        {
            let item_ref = selected.borrow();
            if item_ref.item_type != RowItemType::Directory {
                self.status_message = Some("Can only rescan directories".to_string());
                return;
            }
        }

        let path = selected.borrow().get_path();
        let ancestor_segments = get_ancestor_segments(&selected);
        let old_size = selected.borrow().size.get_value();
        let old_descendants = selected.borrow().descendant_count;

        // Subtract old size from ancestors.
        subtract_from_ancestors(&selected, old_size, old_descendants);

        // Subtract from total.
        self.total_size_in_bytes = self.total_size_in_bytes.saturating_sub(old_size);
        self.total_items_in_tree = self.total_items_in_tree.saturating_sub(old_descendants);

        // Clear the item.
        {
            let mut item_ref = selected.borrow_mut();
            item_ref.children.clear();
            item_ref.size = Size::new(0);
            item_ref.descendant_count = 0;
            item_ref.max_child_size = 0;
            item_ref.has_children = false;
            item_ref.expanded = false;
            item_ref.is_scanning = true;
        }

        self.is_scanning = true;
        self.visible_rows_dirty = true;
        self.rescan_request = Some((path, ancestor_segments));
    }
}

/// Walks the parent chain collecting `path_segment` names from the item up
/// to (but not including) the root item (which has no parent).  Returns the
/// segments in root-to-item order, suitable for use as an `ancestor_path`
/// prefix in `DescendantBatch` messages.
fn get_ancestor_segments(item: &Rc<RefCell<RowItem>>) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = item.clone();
    loop {
        let name = current.borrow().path_segment.clone();
        let parent = current.borrow().parent.as_ref().and_then(|p| p.upgrade());
        match parent {
            Some(p) => {
                segments.push(name);
                current = p;
            }
            None => break, // root item - don't include its name
        }
    }
    segments.reverse();
    segments
}

/// Subtracts `size` and `descendants` from all ancestors of `item` up to the
/// root.
fn subtract_from_ancestors(item: &Rc<RefCell<RowItem>>, size: u64, descendants: usize) {
    let mut current = item.clone();
    loop {
        let parent = current.borrow().parent.as_ref().and_then(|p| p.upgrade());
        let Some(parent) = parent else { break };
        {
            let mut p_ref = parent.borrow_mut();
            let old = p_ref.size.get_value();
            p_ref.size = Size::new(old.saturating_sub(size));
            p_ref.descendant_count = p_ref.descendant_count.saturating_sub(descendants);
        }
        current = parent;
    }
}
