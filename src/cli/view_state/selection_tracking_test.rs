use crate::cli::view_state::ViewState;
use space_rs::{DirectoryItem, DirectoryItemType, Size};
use std::{cmp::min, path::PathBuf};

// ─── Helpers ─────────────────────────────────────────────────────────────────

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

/// Creates a ViewState with a root directory and zero or more file children.
/// Children are added in the given order; `add_scanned_child` will
/// binary-insert them in descending-size order.
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

// ─── Tests for restore_selection ─────────────────────────────────────────────

#[test]
fn restore_selection_handles_empty_tree_without_panic() {
    let mut view_state = ViewState {
        visible_height: 10,
        ..Default::default()
    };
    let path = PathBuf::from("/nonexistent");

    // Must not panic; selection stays at the default (0).
    view_state.restore_selection(&path, 0);

    assert_eq!(0, view_state.table_selected_index);
}

#[test]
fn restore_selection_does_nothing_when_path_not_found() {
    let mut view_state = make_view_state_with_children(&[("child", 100)]);
    view_state.table_selected_index = 1;

    view_state.restore_selection(&PathBuf::from("/nonexistent/path"), 1);

    // No match -> table_selected_index unchanged.
    assert_eq!(1, view_state.table_selected_index);
}

#[test]
fn restore_selection_preserves_item_when_new_item_inserted_after_it() {
    // Tree before: root(0), big_child(1).  Select big_child.
    let mut view_state = make_view_state_with_children(&[("big_child", 100)]);
    view_state.table_selected_index = 1;
    let (path, screen_pos) = view_state.save_selected_path().unwrap();

    // Insert small_child(10) -> goes after big_child.
    // Tree after: root(0), big_child(1), small_child(2).
    view_state.add_scanned_child(make_file_item("small_child", 10));

    view_state.restore_selection(&path, screen_pos);

    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("big_child", selected.borrow().path_segment);
    assert_eq!(1, view_state.table_selected_index);
}

#[test]
fn restore_selection_preserves_item_when_new_item_inserted_before_it() {
    // Tree before: root(0), small_child(1).  Select small_child.
    let mut view_state = make_view_state_with_children(&[("small_child", 10)]);
    view_state.table_selected_index = 1;
    let (path, screen_pos) = view_state.save_selected_path().unwrap();

    // Insert big_child(100) -> goes BEFORE small_child.
    // Tree after: root(0), big_child(1), small_child(2).
    view_state.add_scanned_child(make_file_item("big_child", 100));

    view_state.restore_selection(&path, screen_pos);

    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("small_child", selected.borrow().path_segment);
    // screen_pos=1, row_index=2 -> visible_offset=1, table_selected_index=2-1=1.
    assert_eq!(1, view_state.table_selected_index);
}

#[test]
fn restore_selection_preserves_root_item_when_children_added() {
    // Select root at screen position 0 before any children are added.
    let mut view_state = make_view_state_with_children(&[]);
    view_state.table_selected_index = 0;
    let (path, screen_pos) = view_state.save_selected_path().unwrap();
    assert_eq!(PathBuf::from("/root"), path);

    // Add two children; root row_index stays 0.
    view_state.add_scanned_child(make_file_item("child_a", 200));
    view_state.add_scanned_child(make_file_item("child_b", 50));

    view_state.restore_selection(&path, screen_pos);

    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("/root", selected.borrow().path_segment);
    assert_eq!(0, view_state.table_selected_index);
    assert_eq!(0, view_state.visible_offset);
}

#[test]
fn restore_selection_clamps_when_selected_item_scrolled_off_screen() {
    // visible_height=2 so only 2 items are visible at a time.
    let mut view_state = ViewState {
        visible_height: 2,
        table_width: 80,
        ..Default::default()
    };
    view_state.add_scanned_item(make_root_dir_item());
    view_state.add_scanned_child(make_file_item("a", 10));
    view_state.update_visible_rows();
    view_state.table_selected_index = 1; // selects "a" (row_index=1)
    let (path, screen_pos) = view_state.save_selected_path().unwrap();
    assert_eq!(PathBuf::from("/root").join("a"), path);

    // Insert b(100) which sorts before a: root(0), b(1), a(2).
    // With visible_height=2 and offset=0 -> [root, b] visible; "a" is off screen.
    view_state.add_scanned_child(make_file_item("b", 100));

    view_state.restore_selection(&path, screen_pos);

    // "a" is not in the visible window -> selection is clamped to a valid index.
    let max_valid = min(
        view_state.visible_row_items.len(),
        view_state.visible_height,
    ) - 1;
    assert!(view_state.table_selected_index <= max_valid);
}

#[test]
fn restore_selection_finds_item_that_remains_visible_after_insert() {
    // Tree: root(0), a(1), b(2).  Select b at screen_pos=2.
    let mut view_state = make_view_state_with_children(&[("a", 50), ("b", 10)]);
    view_state.table_selected_index = 2; // selects b
    let (path, screen_pos) = view_state.save_selected_path().unwrap();
    assert_eq!(PathBuf::from("/root").join("b"), path);

    // Insert c(200) -> root(0), c(1), a(2), b(3).
    view_state.add_scanned_child(make_file_item("c", 200));

    view_state.restore_selection(&path, screen_pos);

    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("b", selected.borrow().path_segment);
}

#[test]
fn restore_selection_clamps_to_exact_last_valid_index() {
    // visible_height=2; after insert there are 3 rows (root, b, a).
    // table_selected_index was 1 ("a"); after insert "a" is at row_index=2, off screen.
    // Clamp must produce exactly max_valid = min(3, 2) - 1 = 1.
    let mut view_state = ViewState {
        visible_height: 2,
        table_width: 80,
        ..Default::default()
    };
    view_state.add_scanned_item(make_root_dir_item());
    view_state.add_scanned_child(make_file_item("a", 10));
    view_state.update_visible_rows();
    view_state.table_selected_index = 1;
    let (path, screen_pos) = view_state.save_selected_path().unwrap();

    // Insert b(100) before a -> [root(0), b(1), a(2)]; only [root, b] visible.
    view_state.add_scanned_child(make_file_item("b", 100));
    view_state.restore_selection(&path, screen_pos);

    let max_valid = min(
        view_state.visible_row_items.len(),
        view_state.visible_height,
    ) - 1;
    assert_eq!(
        max_valid, view_state.table_selected_index,
        "index should be clamped to the last valid visible position"
    );
}

#[test]
fn restore_selection_does_not_panic_when_visible_height_is_zero() {
    // visible_height=0 is degenerate; the guard `if max_visible > 0` must prevent panics.
    let mut view_state = ViewState {
        visible_height: 0,
        table_width: 80,
        ..Default::default()
    };
    view_state.add_scanned_item(make_root_dir_item());
    let path = PathBuf::from("/nonexistent");

    // Must not panic; table_selected_index stays at 0.
    view_state.restore_selection(&path, 0);

    assert_eq!(0, view_state.table_selected_index);
}

#[test]
fn restore_selection_finds_item_in_scrolled_view() {
    // Tree: root(0), a(1), b(2), c(3), d(4), e(5) (sorted desc by size).
    // visible_height=3, visible_offset=2.
    // add_table_row adds items with row_index >= visible_offset (i.e. b, c, d, e) including
    // the lookahead item, so visible_row_items = [b(idx=0), c(idx=1), d(idx=2), e(idx=3)].
    // table_selected_index=2 -> selects "d" at row_index=4.
    let mut view_state = ViewState {
        visible_height: 3,
        table_width: 80,
        ..Default::default()
    };
    view_state.add_scanned_item(make_root_dir_item());
    for (name, size) in &[("a", 500), ("b", 400), ("c", 300), ("d", 200), ("e", 100)] {
        view_state.add_scanned_child(make_file_item(name, *size));
    }
    view_state.visible_offset = 2;
    view_state.update_visible_rows();

    // Confirm visible_row_items[2] is "d".
    view_state.table_selected_index = 2;
    let (path, screen_pos) = view_state.save_selected_path().unwrap();
    assert_eq!(
        "d",
        view_state
            .get_selected_item()
            .unwrap()
            .borrow()
            .path_segment
    );

    // No structural changes - restore_selection should find "d" in the visible window.
    // desired_offset = row_index(4) - screen_pos(2) = 2 = current offset -> no second rebuild.
    view_state.restore_selection(&path, screen_pos);

    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("d", selected.borrow().path_segment);
    assert_eq!(2, view_state.visible_offset);
    assert_eq!(2, view_state.table_selected_index);
}

#[test]
fn restore_selection_preserves_item_when_items_reorder_on_size_change() {
    // Tree: root(0), child_a(100)(1), child_b(50)(2), child_c(10)(3).
    // Select child_b (row_index=2, screen_pos=2).
    let mut view_state =
        make_view_state_with_children(&[("child_a", 100), ("child_b", 50), ("child_c", 10)]);
    view_state.table_selected_index = 2;
    let (path, screen_pos) = view_state.save_selected_path().unwrap();
    assert_eq!(
        "child_b",
        view_state
            .get_selected_item()
            .unwrap()
            .borrow()
            .path_segment
    );

    // Insert child_d(75) -> goes between child_a and child_b.
    // New tree: root(0), child_a(100)(1), child_d(75)(2), child_b(50)(3), child_c(10)(4).
    view_state.add_scanned_child(make_file_item("child_d", 75));

    view_state.restore_selection(&path, screen_pos);

    let selected = view_state.get_selected_item().unwrap();
    assert_eq!("child_b", selected.borrow().path_segment);
}
