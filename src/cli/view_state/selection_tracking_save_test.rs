use crate::cli::view_state::ViewState;
use space_rs::{DirectoryItem, DirectoryItemType, Size};
use std::path::PathBuf;

// Helpers are duplicated from selection_tracking_test.rs to keep this module independent.

fn make_file_item(path_segment: &str, size: u64) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: DirectoryItemType::File,
        size_in_bytes: Size::new(size),
        descendant_count: 0,
        children: vec![],
    }
}

fn make_root_dir_item() -> DirectoryItem {
    DirectoryItem {
        path_segment: "/root".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(0),
        descendant_count: 0,
        children: vec![],
    }
}

fn make_view_state_with_children(children: &[(&str, u64)]) -> ViewState {
    let mut view_state = ViewState {
        visible_height: 10,
        table_width: 80,
        ..Default::default()
    };
    view_state.add_scanned_item(make_root_dir_item());
    for (name, size) in children {
        view_state.add_scanned_child(make_file_item(name, *size));
    }
    view_state.update_visible_rows();
    view_state
}

// ─── Tests for save_selected_path ────────────────────────────────────────────

#[test]
fn save_selected_path_returns_none_when_no_items() {
    let mut view_state = ViewState::default();

    assert!(view_state.save_selected_path().is_none());
}

#[test]
fn save_selected_path_returns_path_and_screen_position_of_selected_item() {
    // Tree: root(0), big_child(1), small_child(2) - descending size order.
    let mut view_state = make_view_state_with_children(&[("big_child", 100), ("small_child", 10)]);
    view_state.table_selected_index = 1; // selects big_child

    let result = view_state.save_selected_path();

    assert!(result.is_some());
    let (path, screen_position) = result.unwrap();
    assert_eq!(1, screen_position);
    assert_eq!(PathBuf::from("/root").join("big_child"), path);
}

#[test]
fn save_selected_path_returns_root_item_path_when_root_selected() {
    let mut view_state = make_view_state_with_children(&[("child", 100)]);
    view_state.table_selected_index = 0; // selects the root

    let (path, screen_position) = view_state.save_selected_path().unwrap();

    assert_eq!(0, screen_position);
    assert_eq!(PathBuf::from("/root"), path);
}
