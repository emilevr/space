use super::ViewState;
use crate::cli::row_item::RowItem;
use std::{cell::RefCell, cmp::min, path::PathBuf, rc::Rc};

impl ViewState {
    pub(crate) fn get_selected_item(&self) -> Option<Rc<RefCell<RowItem>>> {
        if self.displayable_item_count > 0
            && self.table_selected_index < min(self.visible_row_items.len(), self.visible_height)
        {
            return Some(self.visible_row_items[self.table_selected_index].clone());
        }
        None
    }

    pub(crate) fn select_item(&mut self, row_index: usize) {
        if row_index == usize::MAX {
            return;
        }

        self.ensure_visible(row_index);

        self.table_selected_index = if row_index >= self.visible_offset
            && row_index < self.visible_offset + self.visible_height
        {
            row_index - self.visible_offset
        } else {
            0
        };
    }

    pub(super) fn ensure_visible(&mut self, row_index: usize) {
        if row_index < self.visible_offset {
            self.set_visible_offset(row_index);
        } else if row_index >= self.visible_offset + self.visible_height {
            self.set_visible_offset(row_index - self.visible_height + 1);
        }
    }

    pub(super) fn set_visible_offset(&mut self, visible_offset: usize) {
        self.visible_offset = visible_offset;
        self.visible_rows_dirty = true;
        self.update_visible_rows();
    }

    /// Captures the currently selected item's full path and screen position
    /// (table_selected_index) so it can be restored after mutations.
    pub(crate) fn save_selected_path(&mut self) -> Option<(PathBuf, usize)> {
        // Ensure visible rows reflect any pending navigation (e.g. the user
        // pressed Up/PageUp which changed visible_offset but hasn't rebuilt
        // yet).  Without this, we'd save the stale pre-navigation item and
        // restore_selection would undo the scroll.
        if self.visible_rows_dirty {
            self.update_visible_rows();
        }
        let selected = self.get_selected_item()?;
        let path = selected.borrow().get_path();
        Some((path, self.table_selected_index))
    }

    /// After mutations (e.g. scan inserts), restores the selection to the item
    /// with the given path at the same screen position.
    pub(crate) fn restore_selection(&mut self, path: &PathBuf, screen_position: usize) {
        // Rebuild visible rows so row_index values are fresh.
        self.visible_rows_dirty = true;
        self.update_visible_rows();

        // Search the full tree for the item's new row_index - it may have
        // shifted due to insertions/reordering above it.
        let found_row_index = find_row_index_by_path(&self.item_tree, path);

        if let Some(row_index) = found_row_index {
            // Place the item at the same screen position it was at before.
            let desired_offset = row_index.saturating_sub(screen_position);
            if desired_offset != self.visible_offset {
                self.visible_offset = desired_offset;
                self.visible_rows_dirty = true;
                self.update_visible_rows();
            }
            self.table_selected_index = row_index - self.visible_offset;
        } else {
            // Item not found (deleted?); clamp to a valid index.
            let max_visible = min(self.visible_row_items.len(), self.visible_height);
            if max_visible > 0 {
                self.table_selected_index = self.table_selected_index.min(max_visible - 1);
            }
        }
    }
}

/// Searches the item tree for an item matching `path` and returns its
/// `row_index`.  Only searches expanded nodes since collapsed items aren't
/// displayable.
fn find_row_index_by_path(item_tree: &[Rc<RefCell<RowItem>>], path: &PathBuf) -> Option<usize> {
    for item in item_tree {
        if let Some(idx) = find_row_index_recursive(item, path) {
            return Some(idx);
        }
    }
    None
}

fn find_row_index_recursive(item: &Rc<RefCell<RowItem>>, path: &PathBuf) -> Option<usize> {
    let item_ref = item.borrow();
    if item_ref.row_index != usize::MAX && item_ref.get_path() == *path {
        return Some(item_ref.row_index);
    }
    if item_ref.expanded {
        for child in &item_ref.children {
            if let Some(idx) = find_row_index_recursive(child, path) {
                return Some(idx);
            }
        }
    }
    None
}
