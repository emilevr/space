use super::row_item::{RowItem, RowItemType};
use anyhow::bail;
use log::error;
use ratatui::widgets::Row;
use serde::{Deserialize, Serialize};
use space_rs::SizeDisplayFormat;
use std::{
    cell::RefCell,
    cmp::min,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    rc::Rc,
};

#[cfg(test)]
#[path = "./view_state_test.rs"]
mod view_state_test;

const CONFIG_FILE_NAME: &str = ".space.yaml";

pub(crate) const APPARENT_SIZE_COLUMN_WIDTH: u16 = 7;
pub(crate) const EXPAND_INDICATOR_COLUMN_WIDTH: u16 = 1;
pub(crate) const INCL_PERCENTAGE_COLUMN_WIDTH: u16 = 4;

pub(crate) const EXCL_PERCENTAGE_MAX_COLUMN_WIDTH: u16 = 20;

pub(crate) const ITEM_TYPE_DIRECTORY_SYMBOL: char = '📁';
pub(crate) const ITEM_TYPE_FILE_SYMBOL: char = '📄';
pub(crate) const ITEM_TYPE_SYMBOLIC_LINK_SYMBOL: char = '🔗';
pub(crate) const ITEM_TYPE_UNKNOWN_SYMBOL: char = '❓';

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    accepted_license_terms: bool,
}

pub(crate) fn get_excl_percentage_column_width(table_width: u16) -> u16 {
    min(
        EXCL_PERCENTAGE_MAX_COLUMN_WIDTH,
        (table_width as f32 * 0.1f32) as u16,
    )
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
        }
    }
}

impl ViewState {
    pub(crate) fn new(
        item_tree: Vec<Rc<RefCell<RowItem>>>,
        size_display_format: SizeDisplayFormat,
        size_threshold_fraction: f32,
    ) -> ViewState {
        let mut view_state = ViewState {
            item_tree,
            size_display_format,
            size_threshold_fraction,
            ..Default::default()
        };
        view_state.update_total_items_in_tree();
        view_state
    }

    pub(crate) fn update_visible_rows(&mut self) -> Vec<Row> {
        let mut rows = vec![];
        self.visible_row_items.clear();

        let mut row_index = 0;
        let mut added_count = 0;
        let mut displayable_item_count = 0;

        for item in &self.item_tree {
            // We don't exit early here as we need to count the total number of visible items.

            add_table_row(
                &mut rows,
                &mut self.visible_row_items,
                item.clone(),
                self.size_display_format,
                self.size_threshold_fraction,
                self.visible_offset,
                self.visible_height,
                &mut row_index,
                &mut added_count,
                &mut displayable_item_count,
                self.table_width,
            );
        }

        self.displayable_item_count = displayable_item_count;

        rows
    }

    pub(crate) fn set_size_threshold_fraction(&mut self, size_threshold_fraction: f32) {
        if size_threshold_fraction != self.size_threshold_fraction {
            // Store the currently selected item so that we can check if we can reselect it.
            let prev_selected_item = self.get_selected_item();

            self.size_threshold_fraction = size_threshold_fraction;
            // Reset the selection.
            self.table_selected_index = 0;
            // Set the visible offset to 0, to ensure that there is at least something visible after filtering.
            self.set_visible_offset(0);

            // Now try and reselect the previously selected item, which will only succeed if it is still
            // visible.
            if let Some(prev_selected_item) = prev_selected_item {
                // Note: We get the prev row index in a block so that the borrow will be returned before we
                // call select_item(), to avoid a RefCell borrow check failure at runtime.
                let prev_selected_row_index = { prev_selected_item.as_ref().borrow().row_index };
                self.select_item(prev_selected_row_index);
            }
        }
    }

    pub(crate) fn expand_selected_item(&self) {
        if let Some(selected_item) = self.get_selected_item() {
            let mut selected_item_ref = selected_item.as_ref().borrow_mut();
            if selected_item_ref.has_children {
                selected_item_ref.expanded = true;
            }
        }
    }

    pub(crate) fn collapse_selected_item(&mut self) {
        if let Some(selected_item) = self.get_selected_item() {
            let mut selected_item_ref = selected_item.as_ref().borrow_mut();
            if selected_item_ref.has_children {
                selected_item_ref.expanded = false;
            } else if let Some(parent) = &selected_item_ref.parent {
                if let Some(parent) = parent.upgrade() {
                    parent.as_ref().borrow_mut().expanded = false;
                    // Note: We get the row index in a block so that the borrow will be returned before we
                    // call select_item(), to avoid a RefCell borrow check failure at runtime.
                    let row_index = { parent.as_ref().borrow().row_index };
                    self.select_item(row_index);
                }
            }
        }
    }

    pub(crate) fn previous(&mut self, increment: usize) {
        let mut i = self.table_selected_index;
        if i >= increment {
            i -= increment;
        } else if self.visible_offset > increment {
            self.visible_offset -= increment;
            i = 0;
        } else {
            if self.visible_offset + i >= increment {
                self.visible_offset = self.visible_offset + i - increment;
            } else {
                self.visible_offset = 0;
            }
            i = 0;
        }
        self.table_selected_index = i;
    }

    pub(crate) fn next(&mut self, increment: usize) {
        if self.displayable_item_count == 0 {
            return;
        }

        let mut i = self.table_selected_index + increment;

        let max_table_selected_index = self.visible_height - 1;
        if i > max_table_selected_index {
            self.visible_offset += i - max_table_selected_index;
            i = max_table_selected_index;
        }

        if self.visible_offset + i >= self.displayable_item_count {
            self.visible_offset =
                self.displayable_item_count - min(self.visible_height, self.displayable_item_count);
            i = min(max_table_selected_index, self.displayable_item_count - 1);
        }

        self.table_selected_index = i;
    }

    pub(crate) fn first(&mut self) {
        self.visible_offset = 0;
        self.table_selected_index = 0;
    }

    pub(crate) fn last(&mut self) {
        if self.displayable_item_count == 0 {
            return;
        }

        self.visible_offset =
            self.displayable_item_count - min(self.visible_height, self.displayable_item_count);
        self.table_selected_index = min(self.visible_height - 1, self.displayable_item_count - 1);
    }

    pub(crate) fn collapse_selected_children(&self) {
        if let Some(selected_item) = self.get_selected_item() {
            selected_item.as_ref().borrow_mut().collapse_all_children();
        }
    }

    pub(crate) fn expand_selected_children(&mut self) {
        if let Some(selected_item) = self.get_selected_item() {
            selected_item.as_ref().borrow_mut().expand_all_children();
        }
    }

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

    fn ensure_visible(&mut self, row_index: usize) {
        if row_index < self.visible_offset {
            self.set_visible_offset(row_index);
        } else if row_index >= self.visible_offset + self.visible_height {
            self.set_visible_offset(row_index - self.visible_height + 1);
        }
    }

    fn set_visible_offset(&mut self, visible_offset: usize) {
        self.visible_offset = visible_offset;
        self.update_visible_rows();
    }

    pub(crate) fn delete_selected_item(&mut self) {
        if let Some(selected_item) = self.get_selected_item() {
            let selected_item_ref = selected_item.borrow();
            let path = selected_item_ref.get_path();

            let remove_result = if path.is_dir() {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };
            match remove_result {
                Ok(()) => {
                    if let Some(parent) = &selected_item_ref.parent {
                        self.remove_child_item(parent, &selected_item);
                    } else {
                        self.remove_top_level_item(&selected_item);
                    }
                }
                Err(e) => {
                    // TODO: Push the error into some sort of error stream and expose in UI.
                    error!("Deletion of item failed: {}", e);
                }
            }
        }
    }

    fn remove_child_item(
        &mut self,
        parent: &std::rc::Weak<RefCell<RowItem>>,
        selected_item: &Rc<RefCell<RowItem>>,
    ) {
        if let Some(parent) = parent.upgrade() {
            let mut child_index = None;
            {
                let children = &parent.borrow().children;
                for (i, child) in children.iter().enumerate() {
                    if Rc::ptr_eq(child, selected_item) {
                        child_index = Some(i);
                        break;
                    }
                }
            }

            if let Some(child_index) = child_index {
                {
                    let mut parent_ref = parent.borrow_mut();
                    let children = &mut parent_ref.children;
                    children.remove(child_index);
                }

                // Update the total number of items in the tree
                self.update_total_items_in_tree();

                // Lastly remove the size of the deleted item from self and ancestors.
                subtract_item_tree_size(&parent, selected_item.borrow().size.get_value());
            }
        }
    }

    fn remove_top_level_item(&mut self, selected_item: &Rc<RefCell<RowItem>>) {
        let mut item_index = None;
        for (i, item) in self.item_tree.iter().enumerate() {
            if Rc::ptr_eq(item, selected_item) {
                item_index = Some(i);
                break;
            }
        }
        if let Some(item_index) = item_index {
            self.item_tree.remove(item_index);
            self.update_total_items_in_tree();
        }
    }

    fn update_total_items_in_tree(&mut self) {
        let mut count = 0;
        for item in &self.item_tree {
            count += get_item_tree_count(item);
        }
        self.total_items_in_tree = count
    }

    pub(crate) fn accept_license_terms(&mut self) {
        self.accepted_license_terms = true;
        // TODO: Push any error into some sort of error stream and expose in UI.
        let _ = self.write_config_file();
    }

    pub(crate) fn read_config_file(&mut self) -> anyhow::Result<()> {
        self.ensure_config_file_path()?;

        let file_path = self
            .config_file_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The default config file path was not set!"))?;

        let mut file = File::open(file_path)?;
        let mut yaml = String::new();
        file.read_to_string(&mut yaml)?;

        let config: Config = serde_yaml::from_str(&yaml)?;

        self.accepted_license_terms = config.accepted_license_terms;

        Ok(())
    }

    fn write_config_file(&mut self) -> anyhow::Result<()> {
        self.ensure_config_file_path()?;

        let file_path = self
            .config_file_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The default config file path was not set!"))?;

        let config = Config {
            accepted_license_terms: self.accepted_license_terms,
        };

        let yaml = serde_yaml::to_string(&config)?;

        let mut file = File::create(file_path)?;
        file.write_all(yaml.as_bytes())?;

        Ok(())
    }

    fn ensure_config_file_path(&mut self) -> anyhow::Result<()> {
        if self.config_file_path.is_some() {
            return Ok(());
        }

        if let Some(home_dir) = dirs::home_dir() {
            self.config_file_path = Some(home_dir.join(PathBuf::from(CONFIG_FILE_NAME)));
            Ok(())
        } else {
            bail!("Could not determine the home directory in order to write the config file.");
        }
    }
}

fn subtract_item_tree_size(item: &RefCell<RowItem>, size: u64) {
    let mut item_ref = item.borrow_mut();
    item_ref.size.subtract(size);

    if let Some(parent) = &item_ref.parent {
        if let Some(parent) = parent.upgrade() {
            let parent = parent.as_ref();
            subtract_item_tree_size(parent, size);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn add_table_row(
    rows: &mut Vec<Row>,
    row_items: &mut Vec<Rc<RefCell<RowItem>>>,
    item: Rc<RefCell<RowItem>>,
    size_display_format: SizeDisplayFormat,
    size_threshold_fraction: f32,
    visible_offset: usize,
    visible_height: usize,
    row_index: &mut usize,
    added_count: &mut usize,
    displayable_item_count: &mut usize,
    table_width: u16,
) {
    let item_incl_fraction = item.as_ref().borrow().incl_fraction;
    if item_incl_fraction < size_threshold_fraction {
        reset_item_tree_row_indices(&item);
        return;
    }

    *displayable_item_count += 1;

    if *row_index >= visible_offset {
        // We allow one more row than the visible_height so we can use it as a lookahead, therefore we
        // check against <= visible_height rather than < visible_height.
        if *added_count <= visible_height {
            let cells = get_row_cell_content(&item, size_display_format, table_width, false);
            rows.push(Row::new(cells).height(1));
            row_items.push(item.clone());

            *added_count += 1;
        }
    }

    {
        let mut item_ref = item.as_ref().borrow_mut();
        item_ref.row_index = *row_index;
    }

    *row_index += 1;

    // We don't exit early here as we need to count the total number of visible items.
    let item_ref = item.borrow();
    if item_ref.has_children && item_ref.expanded {
        for child in &item_ref.children {
            add_table_row(
                rows,
                row_items,
                child.clone(),
                size_display_format,
                size_threshold_fraction,
                visible_offset,
                visible_height,
                row_index,
                added_count,
                displayable_item_count,
                table_width,
            );

            // We don't exit early here as we need to count the total number of visible items.
        }
    }
}

fn reset_item_tree_row_indices(item: &Rc<RefCell<RowItem>>) {
    let mut item_ref = item.as_ref().borrow_mut();
    item_ref.row_index = usize::MAX;
    if item_ref.has_children {
        for child in &item_ref.children {
            reset_item_tree_row_indices(child);
        }
    }
}

pub(crate) fn get_row_cell_content(
    item: &Rc<RefCell<RowItem>>,
    size_display_format: SizeDisplayFormat,
    table_width: u16,
    is_non_interactive_output: bool,
) -> Vec<String> {
    let item_ref = item.borrow();
    let excl_filled_count =
        (item_ref.incl_fraction * get_excl_percentage_column_width(table_width) as f32) as usize;

    let mut cells = Vec::with_capacity(if is_non_interactive_output { 4 } else { 5 });
    cells.push(format!(
        "{:>1$}",
        item_ref.size.to_string(size_display_format),
        APPARENT_SIZE_COLUMN_WIDTH as usize
    ));
    if !is_non_interactive_output {
        cells.push(if item_ref.has_children {
            if item_ref.expanded {
                "▽".to_string()
            } else {
                "▶".to_string()
            }
        } else {
            Default::default()
        });
    }
    cells.push(format!(
        "{}{}{}",
        item_ref.tree_prefix,
        match item_ref.item_type {
            RowItemType::Directory => ITEM_TYPE_DIRECTORY_SYMBOL,
            RowItemType::File => ITEM_TYPE_FILE_SYMBOL,
            RowItemType::SymbolicLink => ITEM_TYPE_SYMBOLIC_LINK_SYMBOL,
            RowItemType::Unknown => ITEM_TYPE_UNKNOWN_SYMBOL,
        },
        item_ref.path_segment
    ));
    cells.push("░".repeat(excl_filled_count));
    cells.push(format!(
        "{:>1$.0}%",
        (item_ref.incl_fraction * 100f32).floor(),
        INCL_PERCENTAGE_COLUMN_WIDTH as usize - 1
    ));

    cells
}

fn get_item_tree_count(item: &Rc<RefCell<RowItem>>) -> usize {
    let mut count = 1;
    for child in &item.borrow().children {
        count += get_item_tree_count(child);
    }
    count
}
