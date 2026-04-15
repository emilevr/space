use super::{handle_filter_input_key, handle_key_input, handle_normal_key};
use crate::cli::tui::*;
use crate::cli::view_state::ViewState;
use crossterm::event::KeyCode;

#[test]
fn handle_normal_key_filter_key_activates_input_mode() {
    let mut view_state = ViewState::default();
    handle_normal_key(&mut view_state, KeyCode::Char(FILTER_KEY));
    assert!(view_state.is_filter_input_active);
}

#[test]
fn handle_normal_key_filter_key_clears_buffer() {
    let mut view_state = ViewState {
        filter_input_buffer: "old".to_string(),
        ..Default::default()
    };
    handle_normal_key(&mut view_state, KeyCode::Char(FILTER_KEY));
    assert!(view_state.filter_input_buffer.is_empty());
}

#[test]
fn handle_key_input_routes_to_filter_input_when_active() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        ..Default::default()
    };
    handle_key_input(&mut view_state, KeyCode::Char('a'));
    assert_eq!("a", view_state.filter_input_buffer);
}

#[test]
fn handle_filter_input_key_char_appends_to_buffer() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Char('s'));
    handle_filter_input_key(&mut view_state, KeyCode::Char('r'));
    handle_filter_input_key(&mut view_state, KeyCode::Char('c'));
    assert_eq!("src", view_state.filter_input_buffer);
}

#[test]
fn handle_filter_input_key_backspace_removes_last_char() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "src".to_string(),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Backspace);
    assert_eq!("sr", view_state.filter_input_buffer);
}

#[test]
fn handle_filter_input_key_backspace_on_empty_buffer_does_not_panic() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: String::new(),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Backspace);
    assert!(view_state.filter_input_buffer.is_empty());
}

#[test]
fn handle_filter_input_key_esc_deactivates_input_mode_and_discards_buffer() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "abc".to_string(),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Esc);
    assert!(!view_state.is_filter_input_active);
    assert!(view_state.filter_input_buffer.is_empty());
}

#[test]
fn handle_filter_input_key_esc_does_not_change_active_filter() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "abc".to_string(),
        filter_display: Some("old_pattern".to_string()),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Esc);
    assert_eq!(Some("old_pattern".to_string()), view_state.filter_display);
    assert!(view_state.filter_regex.is_none());
}

#[test]
fn handle_filter_input_key_enter_with_valid_regex_applies_filter() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "src".to_string(),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Enter);
    assert!(!view_state.is_filter_input_active);
    assert!(
        view_state.filter_regex.is_some(),
        "filter_regex should be set"
    );
    assert_eq!(Some("src".to_string()), view_state.filter_display);
}

#[test]
fn handle_filter_input_key_enter_with_empty_buffer_clears_filter() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: String::new(),
        filter_display: Some("old".to_string()),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Enter);
    assert!(!view_state.is_filter_input_active);
    assert!(view_state.filter_regex.is_none());
    assert!(view_state.filter_display.is_none());
}

#[test]
fn handle_filter_input_key_enter_with_invalid_regex_shows_error_and_keeps_input_active() {
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "[invalid".to_string(),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Enter);
    assert!(
        view_state.is_filter_input_active,
        "input mode should stay active on invalid regex"
    );
    assert!(
        view_state.status_message.is_some(),
        "status_message should be set on invalid regex"
    );
    assert!(view_state.filter_regex.is_none());
}

#[test]
fn handle_key_input_filter_active_quit_key_appends_to_buffer_not_exit() {
    // Pressing 'q' while in filter input mode must NOT exit - it appends to buffer.
    let mut view_state = ViewState {
        is_filter_input_active: true,
        ..Default::default()
    };
    let should_exit = handle_key_input(&mut view_state, KeyCode::Char(QUIT_KEY_1));
    assert!(
        !should_exit,
        "quit key must not exit while filter input is active"
    );
    assert_eq!(
        QUIT_KEY_1.to_string(),
        view_state.filter_input_buffer,
        "quit char should be appended to filter buffer"
    );
}

#[test]
fn handle_filter_input_key_enter_invalid_regex_preserves_previous_filter_display() {
    // When a previous filter was active, submitting invalid regex must not clear it.
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "[bad".to_string(),
        filter_display: Some("previous_pattern".to_string()),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Enter);
    assert!(
        view_state.is_filter_input_active,
        "input mode should stay active"
    );
    assert_eq!(
        Some("previous_pattern".to_string()),
        view_state.filter_display,
        "previous filter_display should be preserved on invalid regex"
    );
}

#[test]
fn handle_filter_input_key_enter_valid_regex_updates_filter_display() {
    // After applying a new valid filter the display shows the new pattern.
    let mut view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "new_pattern".to_string(),
        filter_display: Some("old_pattern".to_string()),
        ..Default::default()
    };
    handle_filter_input_key(&mut view_state, KeyCode::Enter);
    assert!(!view_state.is_filter_input_active);
    assert_eq!(
        Some("new_pattern".to_string()),
        view_state.filter_display,
        "filter_display should be updated to the new pattern"
    );
}
