#[path = "help_sections.rs"]
mod help_sections;

use super::rendering::contract_area;
use crate::cli::skin::Skin;
use ratatui::{
    layout::Constraint,
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear},
    Frame,
};

pub(in crate::cli) fn render_help<B: Backend>(f: &mut Frame<B>, skin: &Skin) {
    let block = Block::default().title("Help").borders(Borders::ALL);
    let mut area = f.size();
    f.render_widget(Clear, area); // Clear out the background
    f.render_widget(block, area);

    contract_area(&mut area, 2, 2);

    let key_style = Style::default()
        .fg(skin.title_fg_color)
        .bg(skin.table_header_bg_color);
    let danger_key_style = Style::default()
        .fg(skin.title_fg_color)
        .bg(skin.key_help_danger_bg_color);
    let section_header_style = Style::default().add_modifier(Modifier::BOLD);

    let key_column_size: usize = 11;
    let column_constraints = [
        Constraint::Length(key_column_size as u16),
        Constraint::Length(key_column_size as u16),
        Constraint::Length(area.width - key_column_size as u16 * 2),
    ];

    help_sections::render_general_section(
        key_style,
        danger_key_style,
        section_header_style,
        key_column_size,
        column_constraints,
        f,
        &mut area,
    );
    help_sections::render_navigation_section(
        key_style,
        section_header_style,
        key_column_size,
        column_constraints,
        f,
        &mut area,
    );
    help_sections::render_filtering_section(
        key_style,
        section_header_style,
        key_column_size,
        column_constraints,
        f,
        &mut area,
    );
}
