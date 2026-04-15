#[cfg(test)]
#[path = "regex_filter_test.rs"]
mod regex_filter_test;

#[cfg(test)]
#[path = "regex_filter_edge_test.rs"]
mod regex_filter_edge_test;

use super::ViewState;
use crate::cli::row_item::RowItem;
use regex::Regex;
use std::{cell::RefCell, rc::Rc};

impl ViewState {
    /// Applies the current `filter_regex` to all items in the tree, updating the
    /// `regex_visible` cached flag on each node. Call this once after the regex
    /// changes or new items arrive while a filter is active. The render path reads
    /// only the cached flag - no regex work happens per frame.
    pub(crate) fn apply_regex_filter(&mut self) {
        match &self.filter_regex {
            Some(regex) => {
                let regex = regex.clone();
                for item in &self.item_tree {
                    apply_filter_recursive(item, &regex, "");
                }
            }
            None => {
                for item in &self.item_tree {
                    clear_filter_recursive(item);
                }
            }
        }
        self.visible_rows_dirty = true;
    }

    /// Sets the regex filter, recomputes the visibility cache, and resets the
    /// selection/scroll position. Pass `None` to clear the filter.
    #[allow(dead_code)]
    pub(crate) fn set_filter_regex(&mut self, filter_regex: Option<Regex>) {
        self.filter_regex = filter_regex;
        self.table_selected_index = 0;
        self.set_visible_offset(0);
        self.apply_regex_filter();
    }
}

/// Recursively marks `regex_visible` for an item and all its descendants.
/// Returns `true` if this item or any of its descendants match the regex.
fn apply_filter_recursive(item: &Rc<RefCell<RowItem>>, regex: &Regex, path_prefix: &str) -> bool {
    let (path_segment, children) = {
        let item_ref = item.borrow();
        (item_ref.path_segment.clone(), item_ref.children.clone())
    };

    let path = if path_prefix.is_empty() {
        path_segment
    } else {
        format!("{}/{}", path_prefix, path_segment)
    };

    let self_matches = regex.is_match(&path);

    // Iterate all children (not short-circuit) so every node gets its flag set.
    let mut any_child_matches = false;
    for child in &children {
        if apply_filter_recursive(child, regex, &path) {
            any_child_matches = true;
        }
    }

    let is_visible = self_matches || any_child_matches;
    item.borrow_mut().regex_visible = is_visible;
    is_visible
}

/// Recursively resets `regex_visible` to `true` for an item and all its descendants.
fn clear_filter_recursive(item: &Rc<RefCell<RowItem>>) {
    item.borrow_mut().regex_visible = true;
    let children = item.borrow().children.clone();
    for child in &children {
        clear_filter_recursive(child);
    }
}
