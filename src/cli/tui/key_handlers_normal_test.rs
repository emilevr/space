use super::handle_normal_key;
use crate::cli::row_item::{RowItem, RowItemType};
use crate::cli::tui::*;
use crate::cli::view_state::ViewState;
use crossterm::event::KeyCode;
use rstest::rstest;
use space_rs::Size;
use std::{cell::RefCell, rc::Rc};

/// Creates a minimal RowItem wrapped in Rc<RefCell>.
fn make_row_item(name: &str) -> Rc<RefCell<RowItem>> {
    Rc::new(RefCell::new(RowItem {
        size: Size::new(0),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: 0.0,
        peer_fraction: 0.0,
        path_segment: name.to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }))
}

/// Creates a ViewState with a selected child item whose parent has the given
/// `is_scanning` value.  The global `view_state.is_scanning` is set to `true`.
fn make_scanning_view_state_with_selected_child(parent_scanning: bool) -> ViewState {
    let parent = make_row_item("parent");
    parent.borrow_mut().is_scanning = parent_scanning;
    parent.borrow_mut().has_children = true;

    let child = make_row_item("child");
    child.borrow_mut().parent = Some(Rc::downgrade(&parent));
    parent.borrow_mut().children.push(child.clone());

    ViewState {
        is_scanning: true,
        item_tree: vec![parent],
        visible_row_items: vec![child],
        displayable_item_count: 1,
        visible_height: 10,
        ..Default::default()
    }
}

#[test]
fn handle_normal_key_quit_returns_true() {
    let mut view_state = ViewState::default();
    assert!(handle_normal_key(
        &mut view_state,
        KeyCode::Char(QUIT_KEY_1)
    ));
}

#[test]
fn handle_normal_key_esc_returns_true() {
    let mut view_state = ViewState::default();
    assert!(handle_normal_key(&mut view_state, KeyCode::Esc));
}

#[test]
fn handle_normal_key_help_sets_show_help_flag() {
    let mut view_state = ViewState::default();
    handle_normal_key(&mut view_state, KeyCode::Char(HELP_KEY));
    assert!(view_state.show_help);
}

#[test]
fn handle_normal_key_delete_sets_show_delete_dialog_flag() {
    let mut view_state = ViewState::default();
    handle_normal_key(&mut view_state, KeyCode::Char(DELETE_KEY));
    assert!(view_state.show_delete_dialog);
}

#[test]
fn handle_normal_key_delete_blocked_when_parent_is_scanning() {
    let mut view_state = make_scanning_view_state_with_selected_child(true);

    handle_normal_key(&mut view_state, KeyCode::Char(DELETE_KEY));

    assert!(
        !view_state.show_delete_dialog,
        "Delete dialog must not open while parent is scanning"
    );
    assert!(
        view_state
            .status_message
            .as_deref()
            .unwrap_or("")
            .contains("scanning"),
        "Status message should mention scanning"
    );
}

#[test]
fn handle_normal_key_delete_allowed_when_parent_finished_scanning() {
    // Global scan is still in progress, but THIS child's parent is done.
    let mut view_state = make_scanning_view_state_with_selected_child(false);

    handle_normal_key(&mut view_state, KeyCode::Char(DELETE_KEY));

    assert!(
        view_state.show_delete_dialog,
        "Delete dialog should open when parent is not scanning"
    );
    assert!(
        view_state.status_message.is_none(),
        "No status message should be set when delete is allowed"
    );
}

#[test]
fn handle_normal_key_delete_not_scanning_opens_dialog_and_no_status_message() {
    let mut view_state = ViewState {
        is_scanning: false,
        ..Default::default()
    };

    handle_normal_key(&mut view_state, KeyCode::Char(DELETE_KEY));

    assert!(
        view_state.show_delete_dialog,
        "Delete dialog should open when not scanning"
    );
    assert!(
        view_state.status_message.is_none(),
        "No status message should be set when delete is allowed"
    );
}

#[test]
fn handle_normal_key_clears_status_message_on_any_key() {
    let mut view_state = ViewState {
        is_scanning: true,
        status_message: Some("Cannot delete while parent is scanning".to_string()),
        ..Default::default()
    };

    // Any subsequent key press (here: help) should clear the message.
    handle_normal_key(&mut view_state, KeyCode::Char(HELP_KEY));

    assert!(
        view_state.status_message.is_none(),
        "Status message must be cleared on next key press"
    );
}

#[test]
fn handle_normal_key_clears_status_message_before_setting_new_one() {
    // Pre-existing message from a previous key press.
    let mut view_state = make_scanning_view_state_with_selected_child(true);
    view_state.status_message = Some("old message".to_string());

    // Press delete again - old message is replaced by the new one.
    handle_normal_key(&mut view_state, KeyCode::Char(DELETE_KEY));

    let msg = view_state.status_message.as_deref().unwrap_or("");
    assert!(
        !msg.contains("old message"),
        "Old status message should be replaced"
    );
    assert!(
        msg.contains("scanning"),
        "New message should mention scanning"
    );
}

#[rstest]
#[case(VIEW_SIZE_THRESHOLD_0_PERCENT_KEY, 0f32)]
#[case(VIEW_SIZE_THRESHOLD_10_PERCENT_KEY, 0.01f32)]
#[case(VIEW_SIZE_THRESHOLD_20_PERCENT_KEY, 0.02f32)]
#[case(VIEW_SIZE_THRESHOLD_30_PERCENT_KEY, 0.03f32)]
#[case(VIEW_SIZE_THRESHOLD_40_PERCENT_KEY, 0.04f32)]
#[case(VIEW_SIZE_THRESHOLD_50_PERCENT_KEY, 0.05f32)]
#[case(VIEW_SIZE_THRESHOLD_60_PERCENT_KEY, 0.06f32)]
#[case(VIEW_SIZE_THRESHOLD_70_PERCENT_KEY, 0.07f32)]
#[case(VIEW_SIZE_THRESHOLD_80_PERCENT_KEY, 0.08f32)]
#[case(VIEW_SIZE_THRESHOLD_90_PERCENT_KEY, 0.09f32)]
fn handle_normal_key_size_threshold_keys_set_correct_fraction(
    #[case] key: char,
    #[case] expected_fraction: f32,
) {
    let mut view_state = ViewState::default();
    handle_normal_key(&mut view_state, KeyCode::Char(key));
    assert!(
        (view_state.size_threshold_fraction - expected_fraction).abs() < f32::EPSILON,
        "Expected fraction {expected_fraction}, got {}",
        view_state.size_threshold_fraction
    );
}

#[test]
fn handle_normal_key_unknown_key_returns_false_and_leaves_state_unchanged() {
    let mut view_state = ViewState::default();
    let should_exit = handle_normal_key(&mut view_state, KeyCode::F(5));
    assert!(!should_exit);
    assert!(!view_state.show_help);
    assert!(!view_state.show_delete_dialog);
}
