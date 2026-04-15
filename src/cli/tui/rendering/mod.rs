pub(in crate::cli::tui) mod area;
mod table;
mod title_bar;

use crate::cli::{skin::Skin, view_state::ViewState};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
};

pub(in crate::cli) fn create_frame<B: Backend>(
    f: &mut Frame<B>,
    view_state: &mut ViewState,
    skin: &Skin,
) {
    let view_height = f.size().height;
    let table_height = view_height - 1; // Subtract one for the help header.

    view_state.visible_height = table_height as usize - 1; // Subtract one for the column headers.

    let vertical_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(table_height)].as_ref())
        .split(f.size());
    let width = vertical_rects[0].width;
    let horizontal_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(width - 1), Constraint::Length(1)].as_ref())
        .split(vertical_rects[1]);

    title_bar::render_title_bar(f, view_state, &vertical_rects[0], skin);
    table::render_table(f, view_state, &horizontal_rects[0], skin);
    table::render_vertical_scrollbar(f, view_state, &horizontal_rects[1], skin);

    if view_state.show_help {
        super::help::render_help(f, skin);
    } else if view_state.show_delete_dialog {
        if view_state.accepted_license_terms {
            super::dialogs::render_delete_dialog(f, view_state, skin);
        } else {
            super::dialogs::render_accept_license_terms_dialog(f, skin);
        }
    }
}

pub(in crate::cli::tui) use area::{contract_area, expand_area, remove_area_top};
pub(crate) use table::table_column_constraints;
