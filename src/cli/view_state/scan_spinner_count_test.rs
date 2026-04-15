use crate::cli::view_state::ViewState;
use space_rs::{DirectoryItem, DirectoryItemType, Size};

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

// ─── Derived is_scanning propagation ────────────────────────────────────────

#[test]
fn derive_scanning_state_sets_root_is_scanning_when_child_is_scanning() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_child"));

    view_state.derive_scanning_state();

    assert!(
        view_state.item_tree[0].borrow().is_scanning,
        "Root should be is_scanning=true after derive when a child is scanning"
    );
}

#[test]
fn derive_scanning_state_does_not_set_root_is_scanning_for_file_children() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_file_item("file.txt", 100));

    view_state.derive_scanning_state();

    assert!(
        !view_state.item_tree[0].borrow().is_scanning,
        "Root should not be is_scanning when only file children exist"
    );
}

#[test]
fn derive_scanning_state_root_stays_scanning_while_any_child_is_scanning() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));

    // Complete dir_a; dir_b is still scanning.
    view_state.mark_child_scan_complete("dir_a");
    view_state.derive_scanning_state();

    assert!(
        view_state.item_tree[0].borrow().is_scanning,
        "Root should remain is_scanning=true while dir_b is still scanning"
    );
}

#[test]
fn derive_scanning_state_root_clears_when_all_children_complete() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));

    view_state.mark_child_scan_complete("dir_a");
    view_state.mark_child_scan_complete("dir_b");
    view_state.derive_scanning_state();

    assert!(
        !view_state.item_tree[0].borrow().is_scanning,
        "Root should be is_scanning=false once all children complete"
    );
}

#[test]
fn derive_scanning_state_propagates_through_grandchild() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    // Add a directory grandchild that is scanning.
    view_state.add_scanned_grandchild("dir_a", make_empty_dir("subdir"));

    view_state.derive_scanning_state();

    let root = view_state.item_tree[0].borrow();
    let dir_a = root.children[0].borrow();
    assert!(
        dir_a.is_scanning,
        "dir_a should be is_scanning=true because subdir is scanning"
    );
    assert!(
        root.is_scanning,
        "Root should be is_scanning=true because dir_a is scanning"
    );
}

#[test]
fn derive_scanning_state_clears_ancestor_when_grandchild_completes() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_grandchild("dir_a", make_empty_dir("subdir"));

    // Complete both subdir and dir_a.
    view_state.mark_descendant_scan_complete(&["dir_a".to_string(), "subdir".to_string()]);
    view_state.mark_child_scan_complete("dir_a");
    view_state.derive_scanning_state();

    let root = view_state.item_tree[0].borrow();
    assert!(
        !root.is_scanning,
        "Root should be is_scanning=false after all descendants complete"
    );
}

// ─── Sibling short-circuit regression ────────────────────────────────────────

#[test]
fn derive_scanning_state_visits_all_siblings_not_just_first_scanning() {
    // Regression: derive used .any() which short-circuits after the first
    // scanning sibling, skipping derive for later siblings whose own children
    // might be scanning.
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a")); // is_scanning=true
    view_state.add_scanned_child(make_empty_dir("dir_b")); // is_scanning=true

    // Simulate dir_b receiving its batch (clears is_scanning) but having a
    // scanning grandchild.
    view_state
        .add_scanned_descendant_batch(&["dir_b".to_string()], vec![make_empty_dir("subdir_b")]);
    // After batch: dir_b.is_scanning=false, subdir_b.is_scanning=true.

    view_state.derive_scanning_state();

    let root = view_state.item_tree[0].borrow();
    let dir_b = root
        .children
        .iter()
        .find(|c| c.borrow().path_segment == "dir_b")
        .unwrap();
    assert!(
        dir_b.borrow().is_scanning,
        "dir_b should be is_scanning=true because subdir_b is still scanning"
    );
}

#[test]
fn derive_scanning_state_clears_parent_when_all_descendants_complete() {
    // Regression: derive only set is_scanning=true, never false, so a parent
    // marked scanning by a prior derive tick would stay scanning forever even
    // after all descendants completed.
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_grandchild("dir_a", make_empty_dir("subdir"));

    // First derive: dir_a and root get is_scanning=true from subdir.
    view_state.derive_scanning_state();
    assert!(view_state.item_tree[0].borrow().is_scanning);

    // Complete subdir and dir_a.
    view_state.mark_descendant_scan_complete(&["dir_a".to_string(), "subdir".to_string()]);
    view_state.mark_child_scan_complete("dir_a");

    // Second derive should clear root and dir_a.
    view_state.derive_scanning_state();

    let root = view_state.item_tree[0].borrow();
    let dir_a = &root.children[0].borrow();
    assert!(
        !dir_a.is_scanning,
        "dir_a should be is_scanning=false after all its descendants completed"
    );
    assert!(
        !root.is_scanning,
        "Root should be is_scanning=false after all descendants completed"
    );
}

// ─── sort_root_children safety-net clearing ───────────────────────────────────

#[test]
fn sort_root_children_clears_all_is_scanning_flags() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));
    // Both should be is_scanning=true after add_scanned_child.
    {
        let root = view_state.item_tree[0].borrow();
        assert!(root.children.iter().all(|c| c.borrow().is_scanning));
    }

    view_state.sort_root_children();

    let root = view_state.item_tree[0].borrow();
    for child in &root.children {
        assert!(
            !child.borrow().is_scanning,
            "Expected is_scanning=false after sort_root_children for child '{}'",
            child.borrow().path_segment
        );
    }
}

#[test]
fn sort_root_children_resets_root_scanning_state() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));

    view_state.sort_root_children();

    assert!(
        !view_state.item_tree[0].borrow().is_scanning,
        "Root is_scanning should be false after sort_root_children"
    );
    assert_eq!(
        0,
        view_state.item_tree[0].borrow().scanning_child_count,
        "Root scanning_child_count should be 0 after sort_root_children"
    );
}

#[test]
fn sort_root_children_clears_flags_even_when_some_were_already_cleared() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));
    // Clear one manually to simulate partial completion.
    view_state.mark_child_scan_complete("dir_a");

    view_state.sort_root_children();

    let root = view_state.item_tree[0].borrow();
    for child in &root.children {
        assert!(
            !child.borrow().is_scanning,
            "All children should have is_scanning=false after sort_root_children"
        );
    }
}
