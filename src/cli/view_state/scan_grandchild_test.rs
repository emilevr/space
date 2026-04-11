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

/// ViewState with a root directory and one empty directory child ready to
/// receive grandchildren.
fn make_view_state_with_parent_dir(parent_name: &str) -> ViewState {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir(parent_name));
    view_state
}

// ─── Basic addition ──────────────────────────────────────────────────────────

#[test]
fn add_scanned_grandchild_adds_grandchild_to_parent() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    view_state.add_scanned_grandchild("parent_dir", make_file_item("file.txt", 1000));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    assert_eq!(1, parent.children.len());
    assert!(parent.has_children);
    assert!(
        !parent.expanded,
        "Non-root items should stay collapsed during scan"
    );
    assert_eq!("file.txt", parent.children[0].borrow().path_segment);
}

#[test]
fn add_scanned_grandchild_sets_visible_rows_dirty() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.add_scanned_grandchild("parent_dir", make_file_item("file.txt", 100));

    assert!(view_state.visible_rows_dirty);
}

// ─── Size and count propagation ───────────────────────────────────────────────

#[test]
fn add_scanned_grandchild_propagates_size_to_parent() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    view_state.add_scanned_grandchild("parent_dir", make_file_item("file.txt", 1500));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    assert_eq!(1500, parent.size.get_value());
}

#[test]
fn add_scanned_grandchild_propagates_size_to_root_and_total() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    view_state.add_scanned_grandchild("parent_dir", make_file_item("file.txt", 2000));

    let root = view_state.item_tree[0].borrow();
    assert_eq!(2000, root.size.get_value());
    assert_eq!(2000, view_state.total_size_in_bytes);
}

#[test]
fn add_scanned_grandchild_accumulates_size_across_grandchildren() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    view_state.add_scanned_grandchild("parent_dir", make_file_item("a.txt", 1000));
    view_state.add_scanned_grandchild("parent_dir", make_file_item("b.txt", 2000));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    assert_eq!(3000, parent.size.get_value());
    assert_eq!(3000, root.size.get_value());
    assert_eq!(3000, view_state.total_size_in_bytes);
}

#[test]
fn add_scanned_grandchild_propagates_descendant_count_to_parent_and_root() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    view_state.add_scanned_grandchild("parent_dir", make_file_item("gc1.txt", 100));
    view_state.add_scanned_grandchild("parent_dir", make_file_item("gc2.txt", 200));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    assert_eq!(2, parent.descendant_count);
    // root: 1 (parent_dir itself) + 2 (grandchildren) = 3
    assert_eq!(3, root.descendant_count);
}

#[test]
fn add_scanned_grandchild_counts_nested_descendants_from_grandchild_subtree() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    // A grandchild that itself has 3 descendants.
    let deep_grandchild = DirectoryItem {
        path_segment: "subdir".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(6000),
        descendant_count: 3,
        children: vec![
            make_file_item("f1.txt", 2000),
            make_file_item("f2.txt", 2000),
            make_file_item("f3.txt", 2000),
        ],
    };
    view_state.add_scanned_grandchild("parent_dir", deep_grandchild);

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    // 1 (subdir) + 3 (its descendants) = 4
    assert_eq!(4, parent.descendant_count);
    assert_eq!(4, root.descendant_count - 1); // root: 1 (parent_dir) + 4 = 5 -> parent_dir descendants = 4
}

#[test]
fn add_scanned_grandchild_updates_total_items_in_tree() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");
    let before = view_state.total_items_in_tree;

    view_state.add_scanned_grandchild("parent_dir", make_file_item("file.txt", 100));

    assert_eq!(before + 1, view_state.total_items_in_tree);
}
