mod config;
pub(crate) mod deletion;
mod navigation;
mod regex_filter;
mod rescan;
mod scan;
pub(crate) mod scan_helpers;
mod selection;
pub(crate) mod table_rows;
mod visible_rows;

use super::{row_item::RowItem, skin::Skin};
use serde::{Deserialize, Serialize};
use space_rs::SizeDisplayFormat;
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeletionState {
    Idle,
    InProgress,
    Cancelling,
}

#[derive(Debug)]
pub(crate) enum DeletionResult {
    Success,
    Cancelled,
    Error(String),
}

#[cfg(test)]
#[path = "../view_state_test.rs"]
mod view_state_test;

#[cfg(test)]
#[path = "../view_state_selection_test.rs"]
mod view_state_selection_test;

#[cfg(test)]
#[path = "navigation_test.rs"]
mod navigation_test;

#[cfg(test)]
#[path = "navigation_expand_test.rs"]
mod navigation_expand_test;

#[cfg(test)]
#[path = "selection_tracking_test.rs"]
mod selection_tracking_test;

#[cfg(test)]
#[path = "selection_tracking_save_test.rs"]
mod selection_tracking_save_test;

const CONFIG_FILE_NAME: &str = "config.yaml";

pub(crate) const APPARENT_SIZE_COLUMN_WIDTH: u16 = 7;
pub(crate) const EXPAND_INDICATOR_COLUMN_WIDTH: u16 = 1;
pub(crate) const INCL_PERCENTAGE_COLUMN_WIDTH: u16 = 4;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    accepted_license_terms: bool,
}

pub(crate) struct ViewState {
    pub table_selected_index: usize,
    pub item_tree: Vec<Rc<RefCell<RowItem>>>,
    pub total_items_in_tree: usize,
    pub size_display_format: SizeDisplayFormat,
    pub size_threshold_fraction: f32,
    pub visible_height: usize,
    pub visible_offset: usize,
    // The number of items that can be displayed, i.e. not hidden or filtered out. This may be a number
    // greater than what is currently visible in the UI.
    pub displayable_item_count: usize,
    /// The currently visible row items. Note: There could potentially be one additional item that is not
    /// visible, which is used as a lookahead to check if there are additional items. visible_height will
    /// be the maximum number of visible items.
    pub visible_row_items: Vec<Rc<RefCell<RowItem>>>,
    pub show_help: bool,
    pub show_delete_dialog: bool,
    pub accepted_license_terms: bool,
    pub table_width: u16,
    pub config_file_path: Option<PathBuf>,
    pub skin: Skin,
    pub is_scanning: bool,
    pub total_size_in_bytes: u64,
    pub visible_rows_dirty: bool,
    pub auto_expand_done: bool,
    pub spinner_tick: usize,
    pub status_message: Option<String>,
    pub deletion_state: DeletionState,
    pub deletion_cancel_flag: Option<Arc<AtomicBool>>,
    pub deletion_receiver: Option<crossfire::Rx<crossfire::mpsc::List<DeletionResult>>>,
    pub rescan_request: Option<(PathBuf, Vec<String>)>,
    pub filter_regex: Option<regex::Regex>,
    pub is_filter_input_active: bool,
    pub filter_input_buffer: String,
    pub filter_display: Option<String>,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState {
            table_selected_index: 0,
            item_tree: vec![],
            total_items_in_tree: 0,
            size_display_format: SizeDisplayFormat::Metric,
            size_threshold_fraction: 0f32,
            visible_height: 0,
            visible_offset: 0,
            displayable_item_count: 0,
            visible_row_items: vec![],
            show_help: false,
            show_delete_dialog: false,
            accepted_license_terms: false,
            table_width: 0,
            config_file_path: None,
            skin: Skin::default(),
            is_scanning: false,
            total_size_in_bytes: 0,
            visible_rows_dirty: true,
            auto_expand_done: false,
            spinner_tick: 0,
            status_message: None,
            deletion_state: DeletionState::Idle,
            deletion_cancel_flag: None,
            deletion_receiver: None,
            rescan_request: None,
            filter_regex: None,
            is_filter_input_active: false,
            filter_input_buffer: String::new(),
            filter_display: None,
        }
    }
}

impl ViewState {
    pub(crate) fn new(
        item_tree: Vec<Rc<RefCell<RowItem>>>,
        size_display_format: SizeDisplayFormat,
        size_threshold_fraction: f32,
        filter_regex: Option<regex::Regex>,
        skin: &Skin,
    ) -> ViewState {
        let mut view_state = ViewState {
            item_tree,
            size_display_format,
            size_threshold_fraction,
            filter_regex,
            skin: *skin,
            ..Default::default()
        };
        view_state.update_total_items_in_tree();
        view_state
    }

    pub(crate) fn set_size_threshold_fraction(&mut self, size_threshold_fraction: f32) {
        if size_threshold_fraction != self.size_threshold_fraction {
            let prev_selected_item = self.get_selected_item();

            self.size_threshold_fraction = size_threshold_fraction;
            self.table_selected_index = 0;
            self.set_visible_offset(0);

            if let Some(prev_selected_item) = prev_selected_item {
                let prev_selected_row_index = { prev_selected_item.as_ref().borrow().row_index };
                self.select_item(prev_selected_row_index);
            }
        }
    }

    pub(crate) fn expand_selected_item(&mut self) {
        if let Some(selected_item) = self.get_selected_item() {
            let mut selected_item_ref = selected_item.as_ref().borrow_mut();
            if selected_item_ref.has_children {
                selected_item_ref.expanded = true;
                self.visible_rows_dirty = true;
            }
        }
    }

    pub(crate) fn collapse_selected_item(&mut self) {
        if let Some(selected_item) = self.get_selected_item() {
            let selected_item_ref = selected_item.as_ref().borrow();
            if selected_item_ref.has_children && selected_item_ref.expanded {
                // Expanded directory: collapse it.
                drop(selected_item_ref);
                selected_item.as_ref().borrow_mut().expanded = false;
                self.visible_rows_dirty = true;
            } else if let Some(parent) = &selected_item_ref.parent {
                // Collapsed directory or leaf: navigate to parent.
                if let Some(parent) = parent.upgrade() {
                    let row_index = parent.as_ref().borrow().row_index;
                    drop(selected_item_ref);
                    self.select_item(row_index);
                }
            }
        }
    }

    pub(crate) fn collapse_selected_children(&mut self) {
        if let Some(selected_item) = self.get_selected_item() {
            let expanded_children_count = selected_item
                .as_ref()
                .borrow()
                .children
                .iter()
                .filter(|item| item.borrow().expanded)
                .count();
            if expanded_children_count == 0 {
                self.collapse_selected_item();
            } else {
                selected_item.as_ref().borrow_mut().collapse_all_children();
                self.visible_rows_dirty = true;
            }
        }
    }

    pub(crate) fn expand_selected_children(&mut self) {
        if let Some(selected_item) = self.get_selected_item() {
            selected_item.as_ref().borrow_mut().expand_all_children();
            self.visible_rows_dirty = true;
        }
    }

    /// Returns `true` if the selected item's direct parent is still being scanned,
    /// meaning deletion of the selected item should be blocked.  For root items
    /// (no parent), checks the root's own `is_scanning` flag since the root
    /// itself is the container whose children are still being discovered.
    pub(crate) fn is_selected_items_parent_scanning(&self) -> bool {
        let Some(selected) = self.get_selected_item() else {
            return false;
        };
        let selected_ref = selected.borrow();
        if let Some(parent_weak) = &selected_ref.parent {
            if let Some(parent_rc) = parent_weak.upgrade() {
                return parent_rc.borrow().is_scanning;
            }
        }
        // Root item (no parent): check its own is_scanning flag, since the root
        // is the container whose children are still being discovered.
        selected_ref.is_scanning
    }

    pub(super) fn update_total_items_in_tree(&mut self) {
        let mut count = 0;
        for item in &self.item_tree {
            count += get_item_tree_count(item);
        }
        self.total_items_in_tree = count
    }
}

fn get_item_tree_count(item: &Rc<RefCell<RowItem>>) -> usize {
    let mut count = 1;
    for child in &item.borrow().children {
        count += get_item_tree_count(child);
    }
    count
}
