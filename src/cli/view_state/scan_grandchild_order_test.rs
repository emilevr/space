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

fn make_view_state_with_parent_dir(parent_name: &str) -> ViewState {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir(parent_name));
    view_state
}

// ─── Insertion order ─────────────────────────────────────────────────────────

#[test]
fn add_scanned_grandchild_inserts_in_descending_size_order() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    // Add out of order; binary insert must sort them descending by size.
    view_state.add_scanned_grandchild("parent_dir", make_file_item("small", 100));
    view_state.add_scanned_grandchild("parent_dir", make_file_item("large", 9000));
    view_state.add_scanned_grandchild("parent_dir", make_file_item("medium", 1000));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    let names: Vec<String> = parent
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    assert_eq!(vec!["large", "medium", "small"], names);
}

#[test]
fn add_scanned_grandchild_equal_sizes_sorted_by_name_ascending() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    view_state.add_scanned_grandchild("parent_dir", make_file_item("zebra", 500));
    view_state.add_scanned_grandchild("parent_dir", make_file_item("apple", 500));
    view_state.add_scanned_grandchild("parent_dir", make_file_item("mango", 500));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    let names: Vec<String> = parent
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    assert_eq!(vec!["apple", "mango", "zebra"], names);
}

// ─── Parent lookup by name ────────────────────────────────────────────────────

#[test]
fn add_scanned_grandchild_looks_up_parent_by_name() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));

    // Grandchildren go to the correct named parent.
    view_state.add_scanned_grandchild("dir_a", make_file_item("for_a.txt", 100));
    view_state.add_scanned_grandchild("dir_b", make_file_item("for_b.txt", 200));

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

    assert_eq!(1, dir_a.children.len());
    assert_eq!("for_a.txt", dir_a.children[0].borrow().path_segment);
    assert_eq!(1, dir_b.children.len());
    assert_eq!("for_b.txt", dir_b.children[0].borrow().path_segment);
}

#[test]
fn add_scanned_grandchild_multiple_parents_each_receive_correct_grandchildren() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));

    // Interleave grandchildren for both parents.
    view_state.add_scanned_grandchild("dir_a", make_file_item("gc_a1", 100));
    view_state.add_scanned_grandchild("dir_b", make_file_item("gc_b1", 200));
    view_state.add_scanned_grandchild("dir_a", make_file_item("gc_a2", 300));

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

    assert_eq!(2, dir_a.children.len());
    assert_eq!(1, dir_b.children.len());
    // dir_a's children sorted descending: gc_a2(300), gc_a1(100).
    let dir_a_names: Vec<String> = dir_a
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    assert_eq!(vec!["gc_a2", "gc_a1"], dir_a_names);
}

// ─── Tests for expansion state ───────────────────────────────────────────────

#[test]
fn add_scanned_grandchild_new_grandchild_starts_collapsed() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    view_state.add_scanned_grandchild("parent_dir", make_file_item("file.txt", 1000));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    assert!(
        !parent.children[0].borrow().expanded,
        "New grandchild should start collapsed"
    );
}

#[test]
fn add_scanned_grandchild_does_not_expand_parent() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");
    // Parent starts collapsed.
    {
        let root = view_state.item_tree[0].borrow();
        assert!(!root.children[0].borrow().expanded);
    }

    // Grandchildren arrive; parent must stay collapsed.
    view_state.add_scanned_grandchild("parent_dir", make_file_item("first.txt", 1000));
    view_state.add_scanned_grandchild("parent_dir", make_file_item("second.txt", 500));

    let root = view_state.item_tree[0].borrow();
    assert!(
        !root.children[0].borrow().expanded,
        "Non-root parent must stay collapsed during scan"
    );
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn add_scanned_grandchild_with_no_root_does_not_panic() {
    let mut view_state = ViewState::default();

    // No root item - should be a no-op (no panic).
    view_state.add_scanned_grandchild("parent_dir", make_file_item("orphan", 100));

    assert_eq!(0, view_state.item_tree.len());
}

#[test]
fn add_scanned_grandchild_with_nonexistent_parent_does_not_panic() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("real_parent"));

    // "ghost_parent" does not exist - should be a no-op (no panic).
    view_state.add_scanned_grandchild("ghost_parent", make_file_item("orphan", 100));

    let root = view_state.item_tree[0].borrow();
    assert_eq!(1, root.children.len()); // only real_parent
    let real_parent = root.children[0].borrow();
    assert_eq!(0, real_parent.children.len());
}

#[test]
fn add_scanned_grandchild_to_empty_parent_directory() {
    // An empty directory child has no grandchildren yet. Adding one should work.
    let mut view_state = make_view_state_with_parent_dir("empty_dir");

    view_state.add_scanned_grandchild("empty_dir", make_file_item("first_file.txt", 500));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    assert_eq!(1, parent.children.len());
    assert_eq!(500, parent.size.get_value());
}
