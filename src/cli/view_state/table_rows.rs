use crate::cli::{
    row_item::{RowItem, RowItemType},
    skin::Skin,
    view_state::{APPARENT_SIZE_COLUMN_WIDTH, INCL_PERCENTAGE_COLUMN_WIDTH},
};
use ratatui::{
    style::Style,
    widgets::{Cell, Row},
};
use space_rs::SizeDisplayFormat;
use std::{cell::RefCell, rc::Rc};

#[cfg(test)]
#[path = "table_rows_test.rs"]
mod table_rows_test;

pub(crate) const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Maximum width used to generate size bar strings.  The actual column width
/// is determined by layout constraints which will clip the string as needed.
const MAX_SIZE_BAR_WIDTH: u16 = 100;

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_table_row(
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
    skin: &Skin,
    spinner_tick: usize,
    selected_row_index: usize,
    depth: usize,
) {
    let (item_incl_fraction, item_regex_visible) = {
        let item_ref = item.as_ref().borrow();
        (item_ref.incl_fraction, item_ref.regex_visible)
    };
    if item_incl_fraction < size_threshold_fraction {
        reset_item_tree_row_indices(&item);
        return;
    }
    if !item_regex_visible {
        reset_item_tree_row_indices(&item);
        return;
    }

    *displayable_item_count += 1;

    try_add_visible_row(
        rows,
        row_items,
        &item,
        size_display_format,
        visible_offset,
        visible_height,
        *row_index,
        added_count,
        skin,
        spinner_tick,
        selected_row_index,
        depth,
    );

    {
        let mut item_ref = item.as_ref().borrow_mut();
        item_ref.row_index = *row_index;
        item_ref.depth = depth;
    }
    *row_index += 1;

    add_children_table_rows(
        rows,
        row_items,
        &item,
        size_display_format,
        size_threshold_fraction,
        visible_offset,
        visible_height,
        row_index,
        added_count,
        displayable_item_count,
        skin,
        spinner_tick,
        selected_row_index,
        depth + 1,
    );
}

#[allow(clippy::too_many_arguments)]
fn try_add_visible_row(
    rows: &mut Vec<Row>,
    row_items: &mut Vec<Rc<RefCell<RowItem>>>,
    item: &Rc<RefCell<RowItem>>,
    size_display_format: SizeDisplayFormat,
    visible_offset: usize,
    visible_height: usize,
    row_index: usize,
    added_count: &mut usize,
    skin: &Skin,
    spinner_tick: usize,
    selected_row_index: usize,
    depth: usize,
) {
    if row_index >= visible_offset && *added_count <= visible_height {
        let is_selected = *added_count == selected_row_index;
        let cells = get_row_cell_content(
            item,
            size_display_format,
            skin,
            spinner_tick,
            is_selected,
            depth,
        );
        rows.push(Row::new(cells).height(1));
        row_items.push(item.clone());
        *added_count += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn add_children_table_rows(
    rows: &mut Vec<Row>,
    row_items: &mut Vec<Rc<RefCell<RowItem>>>,
    item: &Rc<RefCell<RowItem>>,
    size_display_format: SizeDisplayFormat,
    size_threshold_fraction: f32,
    visible_offset: usize,
    visible_height: usize,
    row_index: &mut usize,
    added_count: &mut usize,
    displayable_item_count: &mut usize,
    skin: &Skin,
    spinner_tick: usize,
    selected_row_index: usize,
    depth: usize,
) {
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
                skin,
                spinner_tick,
                selected_row_index,
                depth,
            );
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

pub(crate) fn get_row_cell_content<'a>(
    item: &Rc<RefCell<RowItem>>,
    size_display_format: SizeDisplayFormat,
    skin: &Skin,
    spinner_tick: usize,
    is_selected: bool,
    depth: usize,
) -> Vec<Cell<'a>> {
    let item_ref = item.borrow();
    let level_fg = if depth % 2 == 0 {
        skin.size_bar_fg_color_even
    } else {
        skin.size_bar_fg_color_odd
    };
    // Pre-swap fg/bg on the selected row so that after the row-level REVERSED
    // modifier is applied the size bar still shows filled=light, unfilled=dark.
    let (bar_fg, bar_bg) = if is_selected {
        (skin.size_bar_bg_color, level_fg)
    } else {
        (level_fg, skin.size_bar_bg_color)
    };
    vec![
        Cell::from(format_size_cell(&item_ref, size_display_format)),
        Cell::from(format_expand_indicator(&item_ref)),
        Cell::from(format_path_cell(&item_ref, skin, spinner_tick)),
        Cell::from(format_size_bar(item_ref.peer_fraction, MAX_SIZE_BAR_WIDTH))
            .style(Style::default().fg(bar_fg).bg(bar_bg)),
        Cell::from(format_incl_percentage(item_ref.incl_fraction)),
    ]
}

pub(crate) fn get_row_cell_content_plain(
    item: &Rc<RefCell<RowItem>>,
    size_display_format: SizeDisplayFormat,
    skin: &Skin,
    spinner_tick: usize,
) -> Vec<String> {
    let item_ref = item.borrow();
    vec![
        format_size_cell(&item_ref, size_display_format),
        format_path_cell(&item_ref, skin, spinner_tick),
        format_size_bar(item_ref.peer_fraction, MAX_SIZE_BAR_WIDTH),
        format_incl_percentage(item_ref.incl_fraction),
    ]
}

fn format_size_cell(
    item_ref: &std::cell::Ref<'_, RowItem>,
    size_display_format: SizeDisplayFormat,
) -> String {
    format!(
        "{:>1$}",
        item_ref.size.to_string(size_display_format),
        APPARENT_SIZE_COLUMN_WIDTH as usize
    )
}

fn format_expand_indicator(item_ref: &std::cell::Ref<'_, RowItem>) -> String {
    if item_ref.has_children {
        if item_ref.expanded {
            "▽".to_string()
        } else {
            "▶".to_string()
        }
    } else {
        Default::default()
    }
}

fn format_path_cell(
    item_ref: &std::cell::Ref<'_, RowItem>,
    skin: &Skin,
    spinner_tick: usize,
) -> String {
    let descendant_count_suffix = if item_ref.descendant_count > 0 {
        format!(" [{}]", item_ref.descendant_count)
    } else {
        String::default()
    };
    let icon = match item_ref.item_type {
        RowItemType::Directory => {
            if item_ref.access_denied {
                skin.item_type_access_denied_symbol
            } else if item_ref.is_scanning {
                SPINNER_FRAMES[spinner_tick % SPINNER_FRAMES.len()]
            } else {
                skin.item_type_directory_symbol
            }
        }
        RowItemType::File => skin.item_type_file_symbol,
        RowItemType::SymbolicLink => skin.item_type_symbolic_link_symbol,
        RowItemType::Unknown => skin.item_type_unknown_symbol,
    };
    format!(
        "{}{}{}{}",
        item_ref.tree_prefix, icon, item_ref.path_segment, descendant_count_suffix
    )
}

const EIGHTH_BLOCKS: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];

fn format_size_bar(fraction: f32, column_width: u16) -> String {
    let total_eighths = (fraction * column_width as f32 * 8.0) as usize;
    let full_blocks = total_eighths / 8;
    let remainder = total_eighths % 8;

    let mut bar = "█".repeat(full_blocks);
    if remainder > 0 {
        bar.push(EIGHTH_BLOCKS[remainder]);
    } else if full_blocks == 0 {
        // Always show at least the smallest fill so every item is visible.
        bar.push(EIGHTH_BLOCKS[1]);
    }
    bar
}

fn format_incl_percentage(incl_fraction: f32) -> String {
    format!(
        "{:>1$.0}%",
        (incl_fraction * 100f32).floor(),
        INCL_PERCENTAGE_COLUMN_WIDTH as usize - 1
    )
}
