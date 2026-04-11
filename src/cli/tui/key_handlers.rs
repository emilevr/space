#[cfg(test)]
#[path = "key_handlers_test.rs"]
mod key_handlers_test;

#[cfg(test)]
#[path = "key_handlers_normal_test.rs"]
mod key_handlers_normal_test;

#[cfg(test)]
#[path = "key_handlers_filter_test.rs"]
mod key_handlers_filter_test;

use super::{
    ACCEPT_LICENSE_TERMS_KEY, COLLAPSE_SELECTED_CHILDREN_KEY, COLLAPSE_SELECTED_CHILDREN_KEY_ALT,
    CONFIRM_DELETE_KEY, DELETE_KEY, EXPAND_SELECTED_CHILDREN_KEY, EXPAND_SELECTED_CHILDREN_KEY_ALT,
    FILTER_KEY, HELP_KEY, QUIT_KEY_1, RESCAN_KEY, VIEW_SIZE_THRESHOLD_0_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_10_PERCENT_KEY, VIEW_SIZE_THRESHOLD_20_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_30_PERCENT_KEY, VIEW_SIZE_THRESHOLD_40_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_50_PERCENT_KEY, VIEW_SIZE_THRESHOLD_60_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_70_PERCENT_KEY, VIEW_SIZE_THRESHOLD_80_PERCENT_KEY,
    VIEW_SIZE_THRESHOLD_90_PERCENT_KEY,
};
use crate::cli::view_state::{DeletionState, ViewState};
use crossterm::event::KeyCode;
use regex::RegexBuilder;

/// Returns true if the loop should exit.
pub(crate) fn handle_key_input(view_state: &mut ViewState, code: KeyCode) -> bool {
    if view_state.is_filter_input_active {
        handle_filter_input_key(view_state, code);
    } else if view_state.show_help {
        view_state.show_help = false;
    } else if view_state.show_delete_dialog {
        handle_delete_dialog_key(view_state, code);
    } else {
        return handle_normal_key(view_state, code);
    }
    false
}

pub(crate) fn handle_delete_dialog_key(view_state: &mut ViewState, code: KeyCode) {
    match view_state.deletion_state {
        DeletionState::InProgress => {
            if code == KeyCode::Esc {
                view_state.cancel_deletion();
            }
            // Ignore all other keys during deletion.
        }
        DeletionState::Cancelling => {
            // Ignore all keys while waiting for cancellation to complete.
        }
        DeletionState::Idle => {
            if view_state.accepted_license_terms {
                match code {
                    KeyCode::Char(CONFIRM_DELETE_KEY) => {
                        view_state.start_async_deletion();
                    }
                    _ => view_state.show_delete_dialog = false,
                }
            } else {
                match code {
                    KeyCode::Char(ACCEPT_LICENSE_TERMS_KEY) => {
                        view_state.accept_license_terms();
                    }
                    _ => view_state.show_delete_dialog = false,
                }
            }
        }
    }
}

/// Returns true if the loop should exit.
pub(crate) fn handle_normal_key(view_state: &mut ViewState, code: KeyCode) -> bool {
    view_state.status_message = None;
    match code {
        KeyCode::Char(HELP_KEY) => view_state.show_help = true,
        KeyCode::Char(DELETE_KEY) => {
            if view_state.is_selected_items_parent_scanning() {
                view_state.status_message =
                    Some("Cannot delete while parent is scanning".to_string());
            } else {
                view_state.show_delete_dialog = true;
            }
        }
        KeyCode::Char(RESCAN_KEY) | KeyCode::F(5) => {
            view_state.prepare_rescan();
        }
        KeyCode::Char(QUIT_KEY_1) | KeyCode::Esc => return true,
        KeyCode::Left => view_state.collapse_selected_item(),
        KeyCode::Right => view_state.expand_selected_item(),
        KeyCode::Up => view_state.previous(1),
        KeyCode::Down => view_state.next(1),
        KeyCode::PageUp => view_state.previous(view_state.visible_height),
        KeyCode::PageDown => view_state.next(view_state.visible_height),
        KeyCode::Home => view_state.first(),
        KeyCode::End => view_state.last(),
        KeyCode::Char(COLLAPSE_SELECTED_CHILDREN_KEY)
        | KeyCode::Char(COLLAPSE_SELECTED_CHILDREN_KEY_ALT) => {
            view_state.collapse_selected_children()
        }
        KeyCode::Char(EXPAND_SELECTED_CHILDREN_KEY)
        | KeyCode::Char(EXPAND_SELECTED_CHILDREN_KEY_ALT) => view_state.expand_selected_children(),
        KeyCode::Char(FILTER_KEY) => {
            view_state.is_filter_input_active = true;
            view_state.filter_input_buffer.clear();
        }
        KeyCode::Char(c) => handle_size_threshold_key(view_state, c),
        _ => {}
    }
    false
}

pub(crate) fn handle_filter_input_key(view_state: &mut ViewState, code: KeyCode) {
    match code {
        KeyCode::Char(c) => view_state.filter_input_buffer.push(c),
        KeyCode::Backspace => {
            view_state.filter_input_buffer.pop();
        }
        KeyCode::Esc => {
            view_state.is_filter_input_active = false;
            view_state.filter_input_buffer.clear();
        }
        KeyCode::Enter => apply_filter_input(view_state),
        _ => {}
    }
}

fn apply_filter_input(view_state: &mut ViewState) {
    if view_state.filter_input_buffer.is_empty() {
        view_state.set_filter_regex(None);
        view_state.filter_display = None;
        view_state.is_filter_input_active = false;
        return;
    }
    match RegexBuilder::new(&view_state.filter_input_buffer)
        .case_insensitive(true)
        .build()
    {
        Ok(regex) => {
            let pattern = view_state.filter_input_buffer.clone();
            view_state.set_filter_regex(Some(regex));
            view_state.filter_display = Some(pattern);
            view_state.is_filter_input_active = false;
        }
        Err(e) => {
            view_state.status_message = Some(format!("Invalid regex: {e}"));
        }
    }
}

fn handle_size_threshold_key(view_state: &mut ViewState, c: char) {
    let fraction = match c {
        VIEW_SIZE_THRESHOLD_0_PERCENT_KEY => Some(0f32),
        VIEW_SIZE_THRESHOLD_10_PERCENT_KEY => Some(0.01f32),
        VIEW_SIZE_THRESHOLD_20_PERCENT_KEY => Some(0.02f32),
        VIEW_SIZE_THRESHOLD_30_PERCENT_KEY => Some(0.03f32),
        VIEW_SIZE_THRESHOLD_40_PERCENT_KEY => Some(0.04f32),
        VIEW_SIZE_THRESHOLD_50_PERCENT_KEY => Some(0.05f32),
        VIEW_SIZE_THRESHOLD_60_PERCENT_KEY => Some(0.06f32),
        VIEW_SIZE_THRESHOLD_70_PERCENT_KEY => Some(0.07f32),
        VIEW_SIZE_THRESHOLD_80_PERCENT_KEY => Some(0.08f32),
        VIEW_SIZE_THRESHOLD_90_PERCENT_KEY => Some(0.09f32),
        _ => None,
    };
    if let Some(fraction) = fraction {
        view_state.set_size_threshold_fraction(fraction);
    }
}
