use crate::cli::{
    row_item::RowItem,
    skin::Skin,
    view_state::{
        ViewState, APPARENT_SIZE_COLUMN_WIDTH, EXPAND_INDICATOR_COLUMN_WIDTH,
        INCL_PERCENTAGE_COLUMN_WIDTH,
    },
};
use ratatui::{
    layout::Constraint,
    prelude::*,
    style::{Modifier, Style},
    widgets::{
        Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
    Frame,
};
use std::{cell::RefCell, rc::Rc};

pub(super) fn render_table<B: Backend>(
    f: &mut Frame<B>,
    view_state: &mut ViewState,
    area: &Rect,
    skin: &Skin,
) {
    let table_header_style = Style::default()
        .bg(skin.table_header_bg_color)
        .fg(skin.table_header_fg_color);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let table_selected_index = view_state.table_selected_index;

    let header_cells = ["Size", "", "Path", "", "Incl"]
        .iter()
        .map(|h| Cell::from(*h));
    let header = Row::new(header_cells)
        .style(table_header_style)
        .height(1)
        .bottom_margin(0);

    // NB! Update the table width with the current available width.
    view_state.table_width = area.width;

    // Measure the widest path cell among visible rows (from the previous frame)
    // so the path column fits its content and the size bar fills remaining space.
    let max_path_width = measure_max_path_width(&view_state.visible_row_items);

    // Only then update the visible rows, as we need the available width here.
    let rows = view_state.update_visible_rows();

    let constraints = table_column_constraints(area.width, max_path_width);
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(selected_style)
        .highlight_symbol("")
        .widths(&constraints);
    f.render_stateful_widget(
        table,
        *area,
        &mut TableState::default().with_selected(Some(table_selected_index)),
    );
}

pub(super) fn render_vertical_scrollbar<B: Backend>(
    f: &mut Frame<B>,
    data: &mut ViewState,
    area: &Rect,
    skin: &Skin,
) {
    // The content length must be the total number of rows minus the number of visible rows.
    let content_length = if data.displayable_item_count > data.visible_height {
        data.displayable_item_count - data.visible_height
    } else {
        data.displayable_item_count
    };

    let mut vertical_scroll_state = ScrollbarState::default()
        .position(data.visible_offset as u16)
        .content_length(content_length as u16)
        .viewport_content_length(data.visible_height as u16);

    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(symbols::scrollbar::Set {
                track: symbols::line::VERTICAL,
                thumb: symbols::block::FULL,
                begin: "▲",
                end: "▼",
            })
            .style(
                Style::default()
                    .bg(skin.title_bg_color)
                    .fg(skin.title_fg_color),
            ),
        *area,
        &mut vertical_scroll_state,
    );
}

pub(crate) fn table_column_constraints(width: u16, path_width: u16) -> [Constraint; 5] {
    let fixed = APPARENT_SIZE_COLUMN_WIDTH
        + EXPAND_INDICATOR_COLUMN_WIDTH
        + INCL_PERCENTAGE_COLUMN_WIDTH
        + 4; // column separators
    let available = (width as i32 - fixed as i32).max(0) as u16;
    // Size bar gets leftover space after path, capped at 50% of total width.
    // Any excess beyond the cap goes back to the path column.
    let max_bar = width / 2;
    let bar_col = available.saturating_sub(path_width).min(max_bar);
    let path_col = available.saturating_sub(bar_col);

    [
        Constraint::Length(APPARENT_SIZE_COLUMN_WIDTH),
        Constraint::Length(EXPAND_INDICATOR_COLUMN_WIDTH),
        Constraint::Length(path_col),
        Constraint::Length(bar_col),
        Constraint::Length(INCL_PERCENTAGE_COLUMN_WIDTH),
    ]
}

/// Returns the maximum path cell width across visible rows.  Uses +2 for the
/// icon character (emoji icons occupy ~2 terminal cells).
fn measure_max_path_width(visible_row_items: &[Rc<RefCell<RowItem>>]) -> u16 {
    visible_row_items
        .iter()
        .map(|item| {
            let r = item.borrow();
            let suffix_len = if r.descendant_count > 0 {
                format!(" [{}]", r.descendant_count).len()
            } else {
                0
            };
            r.tree_prefix.chars().count() + 2 + r.path_segment.len() + suffix_len
        })
        .max()
        .unwrap_or(0) as u16
}
