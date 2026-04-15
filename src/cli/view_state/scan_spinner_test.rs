use crate::cli::view_state::ViewState;
use space_rs::{DirectoryItem, DirectoryItemType, Size};

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

fn make_empty_dir(path_segment: &str) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::default(),
        descendant_count: 0,
        children: vec![],
    }
}

// ─── is_scanning flag on add_scanned_child ───────────────────────────────────

#[test]
fn add_scanned_child_sets_is_scanning_true_for_directory_child() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));

    view_state.add_scanned_child(make_empty_dir("dir_child"));

    let root = view_state.item_tree[0].borrow();
    assert!(
        root.children[0].borrow().is_scanning,
        "Directory child should have is_scanning=true after add_scanned_child"
    );
}

#[test]
fn add_scanned_child_does_not_set_is_scanning_for_file_child() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));

    view_state.add_scanned_child(make_file_item("file.txt", 100));

    let root = view_state.item_tree[0].borrow();
    assert!(
        !root.children[0].borrow().is_scanning,
        "File child should have is_scanning=false"
    );
}

#[test]
fn add_scanned_child_mixed_types_only_directories_get_is_scanning() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));

    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_file_item("file.txt", 100));
    view_state.add_scanned_child(make_empty_dir("dir_b"));

    let root = view_state.item_tree[0].borrow();
    for child in &root.children {
        let child_ref = child.borrow();
        if child_ref.item_type == crate::cli::row_item::RowItemType::Directory {
            assert!(
                child_ref.is_scanning,
                "Directory '{}' should be is_scanning=true",
                child_ref.path_segment
            );
        } else {
            assert!(
                !child_ref.is_scanning,
                "File '{}' should be is_scanning=false",
                child_ref.path_segment
            );
        }
    }
}

// ─── mark_child_scan_complete ─────────────────────────────────────────────────

#[test]
fn mark_child_scan_complete_clears_is_scanning_on_named_child() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_child"));
    // Verify flag is set.
    assert!(
        view_state.item_tree[0].borrow().children[0]
            .borrow()
            .is_scanning
    );

    view_state.mark_child_scan_complete("dir_child");

    assert!(
        !view_state.item_tree[0].borrow().children[0]
            .borrow()
            .is_scanning,
        "is_scanning should be false after mark_child_scan_complete"
    );
}

#[test]
fn mark_child_scan_complete_sets_visible_rows_dirty() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_child"));
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.mark_child_scan_complete("dir_child");

    assert!(view_state.visible_rows_dirty);
}

#[test]
fn mark_child_scan_complete_with_no_root_does_not_panic() {
    let mut view_state = ViewState::default();
    // No root - should be a no-op.
    view_state.mark_child_scan_complete("nonexistent");
}

#[test]
fn mark_child_scan_complete_with_nonexistent_name_does_nothing() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_child"));
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    // "ghost" doesn't exist - no-op.
    view_state.mark_child_scan_complete("ghost");

    // dirty stays false (not set), is_scanning unchanged.
    assert!(!view_state.visible_rows_dirty);
    assert!(
        view_state.item_tree[0].borrow().children[0]
            .borrow()
            .is_scanning
    );
}

#[test]
fn mark_child_scan_complete_only_clears_named_child() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));

    view_state.mark_child_scan_complete("dir_a");

    let root = view_state.item_tree[0].borrow();
    let dir_a = root
        .children
        .iter()
        .find(|c| c.borrow().path_segment == "dir_a")
        .unwrap()
        .borrow();
    let dir_b = root
        .children
        .iter()
        .find(|c| c.borrow().path_segment == "dir_b")
        .unwrap()
        .borrow();
    assert!(!dir_a.is_scanning, "dir_a should be cleared");
    assert!(dir_b.is_scanning, "dir_b should remain scanning");
}
