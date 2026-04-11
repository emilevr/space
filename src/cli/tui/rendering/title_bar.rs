#[cfg(test)]
#[path = "title_bar_test.rs"]
mod title_bar_test;

use super::super::{
    COLLAPSE_CHILDREN_KEY_SYMBOL, COLLAPSE_KEY_SYMBOL, DELETE_KEY, EXPAND_CHILDREN_KEY_SYMBOL,
    EXPAND_KEY_SYMBOL, FILTER_KEY, HELP_KEY, QUIT_KEY_1, QUIT_KEY_2_SYMBOL, SELECT_NEXT_KEY_SYMBOL,
    SELECT_PREV_KEY_SYMBOL, VERSION,
};
use crate::cli::{skin::Skin, view_state::ViewState};
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, Borders, Cell, Row, Table},
};
use std::cmp::max;

pub(super) fn render_title_bar<B: Backend>(
    f: &mut Frame<B>,
    data: &ViewState,
    area: &Rect,
    skin: &Skin,
) {
    let title_style = Style::default()
        .fg(skin.title_fg_color)
        .bg(skin.title_bg_color);
    let version_style = Style::default()
        .fg(skin.version_fg_color)
        .bg(skin.title_bg_color);

    let title = "Space";
    let version_display = format!("v{VERSION}");
    let (scanning_display, scanning_style) = build_scanning_display(data, skin);
    let size_filter_display = format!("\u{2265} {:.0}%", data.size_threshold_fraction * 100f32);
    let filter_display_text = build_filter_display_text(data);
    let filter_style = build_filter_style(data, skin);

    let available_width = area.width as i32
        - (title.len() + version_display.len() + scanning_display.len() + size_filter_display.len() + filter_display_text.len()) as i32
        - 5  // Subtract 5 for the column separators (one extra for the filter column)
        - 1; // Subtract 1 to account for table scrollbar.

    let (key_help, available_width) = get_key_help(available_width, skin);

    let key_help_len = key_help.width();
    let scanning_display_len = scanning_display.len() as u16;
    let title_cells = [
        Cell::from(title),
        Cell::from(version_display.as_str()).set_style(version_style),
        Cell::from(scanning_display).set_style(scanning_style),
        Cell::from(""), // Spacer
        Cell::from(key_help),
        Cell::from(""), // Spacer
        Cell::from(filter_display_text.clone()).set_style(filter_style),
        Cell::from(size_filter_display.as_str()),
    ];

    let title_row = Row::new(title_cells)
        .style(title_style)
        .height(1)
        .bottom_margin(0);

    let spacer_length = max(0, available_width / 2) as u16;

    let title_widths = [
        Constraint::Length(title.len() as u16),
        Constraint::Length(version_display.len() as u16),
        Constraint::Length(scanning_display_len),
        Constraint::Length(spacer_length),
        Constraint::Length(key_help_len as u16),
        Constraint::Length(spacer_length),
        Constraint::Length(filter_display_text.len() as u16),
        Constraint::Length(size_filter_display.len() as u16),
    ];
    let title_table = Table::new([])
        .header(title_row)
        .block(Block::default().borders(Borders::NONE))
        .widths(&title_widths);
    f.render_widget(title_table, *area);
}

fn build_scanning_display(data: &ViewState, skin: &Skin) -> (String, Style) {
    let text = if let Some(ref msg) = data.status_message {
        format!(" {msg}")
    } else if data.is_scanning {
        " Scanning...".to_string()
    } else {
        String::new()
    };
    let style = if data.status_message.is_some() {
        Style::default()
            .fg(skin.key_help_key_fg_color)
            .bg(skin.title_bg_color)
    } else {
        Style::default()
            .fg(skin.version_fg_color)
            .bg(skin.title_bg_color)
    };
    (text, style)
}

fn build_filter_style(data: &ViewState, skin: &Skin) -> Style {
    if data.is_filter_input_active {
        Style::default()
            .fg(skin.title_bg_color)
            .bg(skin.key_help_key_fg_color)
    } else {
        Style::default()
            .fg(skin.key_help_key_fg_color)
            .bg(skin.title_bg_color)
    }
}

fn build_filter_display_text(data: &ViewState) -> String {
    if data.is_filter_input_active {
        format!("/{}_", data.filter_input_buffer)
    } else if let Some(pattern) = &data.filter_display {
        format!("/{pattern}")
    } else {
        String::new()
    }
}

/// Add only as many key help entries as we have space for. We add the more important ones first.
fn get_key_help<'a>(available_width: i32, skin: &Skin) -> (Line<'a>, i32) {
    let mut available_width = available_width;
    let key_style = Style::default()
        .bg(skin.title_bg_color)
        .fg(skin.key_help_key_fg_color);
    let key_help_style = Style::default()
        .bg(skin.title_bg_color)
        .fg(skin.title_fg_color);

    #[rustfmt::skip]
    let all_key_help = vec![
        Span::styled(format!(" {HELP_KEY}"), key_style), Span::styled(" Help ", key_help_style),
        Span::styled(format!(" {QUIT_KEY_1}/{QUIT_KEY_2_SYMBOL}"), key_style), Span::styled(" Quit ", key_help_style),
        Span::styled(format!(" {DELETE_KEY}"), key_style), Span::styled(" Delete ", key_help_style),
        Span::styled(format!(" {FILTER_KEY}"), key_style), Span::styled(" Filter ", key_help_style),
        Span::styled(format!(" {SELECT_PREV_KEY_SYMBOL}{SELECT_NEXT_KEY_SYMBOL}"), key_style), Span::styled(" Selection ", key_help_style),
        Span::styled(format!(" {COLLAPSE_KEY_SYMBOL}{EXPAND_KEY_SYMBOL}"), key_style), Span::styled(" Collapse/Expand ", key_help_style),
        Span::styled(format!(" {COLLAPSE_CHILDREN_KEY_SYMBOL}{EXPAND_CHILDREN_KEY_SYMBOL}"), key_style), Span::styled(" Collapse/Expand Children", key_help_style),
    ];
    let mut key_help = vec![];
    for i in (0..all_key_help.len()).step_by(2) {
        let shortcut = &all_key_help[i];
        let shortcut_width = shortcut.width();
        let help = &all_key_help[i + 1];
        let help_width = help.width();
        if shortcut_width as i32 + help_width as i32 <= available_width {
            key_help.push(shortcut.clone());
            key_help.push(help.clone());
            available_width -= shortcut_width as i32 + help_width as i32;
        } else {
            break;
        }
    }

    (Line::from(key_help), available_width)
}
