use super::build_filter_display_text;
use crate::cli::view_state::ViewState;

// ─── Tests for build_filter_display_text ─────────────────────────────────────

#[test]
fn build_filter_display_text_when_no_filter_returns_empty_string() {
    let view_state = ViewState {
        is_filter_input_active: false,
        filter_display: None,
        ..Default::default()
    };
    assert_eq!("", build_filter_display_text(&view_state));
}

#[test]
fn build_filter_display_text_when_input_active_shows_buffer_with_cursor() {
    let view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "src".to_string(),
        ..Default::default()
    };
    assert_eq!("/src_", build_filter_display_text(&view_state));
}

#[test]
fn build_filter_display_text_when_input_active_with_empty_buffer_shows_cursor_only() {
    let view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: String::new(),
        ..Default::default()
    };
    assert_eq!("/_", build_filter_display_text(&view_state));
}

#[test]
fn build_filter_display_text_when_filter_display_set_shows_pattern() {
    let view_state = ViewState {
        is_filter_input_active: false,
        filter_display: Some("tests.*rs".to_string()),
        ..Default::default()
    };
    assert_eq!("/tests.*rs", build_filter_display_text(&view_state));
}

#[test]
fn build_filter_display_text_input_active_takes_precedence_over_filter_display() {
    // When input is active, show the live buffer (with cursor), not the committed pattern.
    let view_state = ViewState {
        is_filter_input_active: true,
        filter_input_buffer: "new".to_string(),
        filter_display: Some("old_pattern".to_string()),
        ..Default::default()
    };
    assert_eq!("/new_", build_filter_display_text(&view_state));
}
