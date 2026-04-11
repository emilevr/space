use super::{handle_delete_dialog_key, handle_key_input};
use crate::cli::tui::*;
use crate::cli::view_state::ViewState;
use crossterm::event::KeyCode;

// --- Unit tests for handle_key_input -----------------------------------------

#[test]
fn handle_key_input_when_show_help_dismisses_help_and_returns_false() {
    let mut view_state = ViewState {
        show_help: true,
        ..Default::default()
    };

    let should_exit = handle_key_input(&mut view_state, KeyCode::Char('x'));

    assert!(!view_state.show_help);
    assert!(!should_exit);
}

#[test]
fn handle_key_input_when_show_delete_dialog_without_license_terms_and_other_key_closes_dialog() {
    let mut view_state = ViewState {
        show_delete_dialog: true,
        accepted_license_terms: false,
        ..Default::default()
    };

    let should_exit = handle_key_input(&mut view_state, KeyCode::Char('x'));

    assert!(!view_state.show_delete_dialog);
    assert!(!should_exit);
}

#[test]
fn handle_key_input_when_show_delete_dialog_without_license_terms_and_accept_key_accepts_terms() {
    let mut view_state = ViewState {
        show_delete_dialog: true,
        accepted_license_terms: false,
        ..Default::default()
    };

    let should_exit = handle_key_input(&mut view_state, KeyCode::Char(ACCEPT_LICENSE_TERMS_KEY));

    assert!(view_state.accepted_license_terms);
    // Dialog stays open after accepting terms (user still needs to confirm deletion).
    assert!(view_state.show_delete_dialog);
    assert!(!should_exit);
}

#[test]
fn handle_key_input_normal_quit_key_returns_true() {
    let mut view_state = ViewState::default();

    let should_exit = handle_key_input(&mut view_state, KeyCode::Char(QUIT_KEY_1));

    assert!(should_exit);
}

#[test]
fn handle_key_input_esc_returns_true() {
    let mut view_state = ViewState::default();

    let should_exit = handle_key_input(&mut view_state, KeyCode::Esc);

    assert!(should_exit);
}

#[test]
fn handle_key_input_unknown_key_returns_false() {
    let mut view_state = ViewState::default();

    let should_exit = handle_key_input(&mut view_state, KeyCode::F(12));

    assert!(!should_exit);
}

// --- Unit tests for handle_delete_dialog_key ---------------------------------

#[test]
fn handle_delete_dialog_key_with_license_terms_and_other_key_closes_dialog() {
    let mut view_state = ViewState {
        show_delete_dialog: true,
        accepted_license_terms: true,
        ..Default::default()
    };

    handle_delete_dialog_key(&mut view_state, KeyCode::Char('x'));

    assert!(!view_state.show_delete_dialog);
}

#[test]
fn handle_delete_dialog_key_without_license_terms_and_other_key_closes_dialog() {
    let mut view_state = ViewState {
        show_delete_dialog: true,
        accepted_license_terms: false,
        ..Default::default()
    };

    handle_delete_dialog_key(&mut view_state, KeyCode::Char('x'));

    assert!(!view_state.show_delete_dialog);
}

#[test]
fn handle_delete_dialog_key_without_license_terms_and_accept_key_keeps_dialog_open() {
    let mut view_state = ViewState {
        show_delete_dialog: true,
        accepted_license_terms: false,
        ..Default::default()
    };

    handle_delete_dialog_key(&mut view_state, KeyCode::Char(ACCEPT_LICENSE_TERMS_KEY));

    // Dialog stays open; user must now press the confirm key.
    assert!(view_state.show_delete_dialog);
    assert!(view_state.accepted_license_terms);
}
