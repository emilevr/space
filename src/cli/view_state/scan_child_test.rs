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

fn make_empty_root(path_segment: &str) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::default(),
        descendant_count: 0,
        children: vec![],
    }
}
#[test]
fn add_scanned_child_adds_child_to_root() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("child1", 1000));

    let root = view_state.item_tree[0].borrow();
    assert_eq!(1, root.children.len());
    assert!(root.has_children);
    assert!(root.expanded);
    assert_eq!(1000, root.size.get_value());
    assert_eq!(1, root.descendant_count);
    assert_eq!(1000, view_state.total_size_in_bytes);
}

#[test]
fn add_scanned_child_accumulates_size_across_children() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("a", 1000));
    view_state.add_scanned_child(make_file_item("b", 2000));

    let root = view_state.item_tree[0].borrow();
    assert_eq!(2, root.children.len());
    assert_eq!(3000, root.size.get_value());
    assert_eq!(2, root.descendant_count);
    assert_eq!(3000, view_state.total_size_in_bytes);
}

#[test]
fn add_scanned_child_counts_nested_descendants() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    let dir_child = DirectoryItem {
        path_segment: "subdir".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(5000),
        descendant_count: 3,
        children: vec![
            make_file_item("f1", 2000),
            make_file_item("f2", 1500),
            make_file_item("f3", 1500),
        ],
    };
    view_state.add_scanned_child(dir_child);

    let root = view_state.item_tree[0].borrow();
    // 1 (the subdir itself) + 3 (its descendants)
    assert_eq!(4, root.descendant_count);
}

#[test]
fn add_scanned_child_with_no_root_does_not_panic() {
    let mut view_state = ViewState::default();
    // No root item - should be a no-op.
    view_state.add_scanned_child(make_file_item("orphan", 100));
    assert_eq!(0, view_state.item_tree.len());
}
#[test]
fn add_scanned_child_inserts_in_sorted_order_during_scanning() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    // Add in reverse order: small first, then large - binary insert should sort.
    view_state.add_scanned_child(make_file_item("small", 100));
    view_state.add_scanned_child(make_file_item("large", 5000));

    let root = view_state.item_tree[0].borrow();
    let names: Vec<String> = root
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    // Already sorted without calling sort_root_children.
    assert_eq!(vec!["large", "small"], names);
}

#[test]
fn add_scanned_child_inserts_largest_at_front() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("small", 100));
    view_state.add_scanned_child(make_file_item("medium", 1000));
    // Inserting the largest last should place it at index 0.
    view_state.add_scanned_child(make_file_item("large", 9000));

    let root = view_state.item_tree[0].borrow();
    let names: Vec<String> = root
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    assert_eq!(vec!["large", "medium", "small"], names);
}

#[test]
fn add_scanned_child_inserts_at_middle_position() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("large", 9000));
    view_state.add_scanned_child(make_file_item("small", 100));
    // Medium should land between large and small.
    view_state.add_scanned_child(make_file_item("medium", 1000));

    let root = view_state.item_tree[0].borrow();
    let names: Vec<String> = root
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    assert_eq!(vec!["large", "medium", "small"], names);
}

#[test]
fn add_scanned_child_equal_sizes_sorted_by_name_ascending_during_scanning() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("zebra", 500));
    view_state.add_scanned_child(make_file_item("apple", 500));
    view_state.add_scanned_child(make_file_item("mango", 500));

    let root = view_state.item_tree[0].borrow();
    let names: Vec<String> = root
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect();
    // Alphabetical ascending for equal sizes, without needing sort_root_children.
    assert_eq!(vec!["apple", "mango", "zebra"], names);
}

// ─── Tests for expansion state ───────────────────────────────────────────────

#[test]
fn add_scanned_child_new_child_starts_collapsed() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("child", 1000));

    let root = view_state.item_tree[0].borrow();
    assert!(
        !root.children[0].borrow().expanded,
        "New child should start collapsed"
    );
}

#[test]
fn add_scanned_child_root_expands_only_on_first_child() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));
    // Root starts not expanded (no children yet).
    assert!(!view_state.item_tree[0].borrow().expanded);

    view_state.add_scanned_child(make_file_item("first", 1000));
    assert!(
        view_state.item_tree[0].borrow().expanded,
        "Root should expand on first child"
    );

    // Manually collapse root (simulating user action).
    view_state.item_tree[0].borrow_mut().expanded = false;

    // Adding a second child must NOT re-expand the root.
    view_state.add_scanned_child(make_file_item("second", 500));
    assert!(
        !view_state.item_tree[0].borrow().expanded,
        "Root must not re-expand on second child - user collapse state must be preserved"
    );
}

#[test]
fn add_scanned_child_does_not_re_expand_collapsed_root() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    // First child expands root.
    view_state.add_scanned_child(make_file_item("first", 1000));
    assert!(view_state.item_tree[0].borrow().expanded);

    // User collapses root.
    view_state.item_tree[0].borrow_mut().expanded = false;

    // Many more children arrive; none should re-expand root.
    for i in 0..5u64 {
        view_state.add_scanned_child(make_file_item(&format!("child_{i}"), 100 + i));
    }

    assert!(
        !view_state.item_tree[0].borrow().expanded,
        "Collapsed root must remain collapsed regardless of subsequent child insertions"
    );
}

// ─── Tests for targeted prefix updates ───────────────────────────────────────

#[test]
fn add_scanned_child_first_child_updates_root_prefix_to_branch() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    // Before any child: root should have a leaf prefix.
    assert_eq!("──", view_state.item_tree[0].borrow().tree_prefix);

    view_state.add_scanned_child(make_file_item("child", 100));

    // After first child: root should show a branch ("─┬").
    assert_eq!("─┬", view_state.item_tree[0].borrow().tree_prefix);
}

#[test]
fn add_scanned_child_single_child_gets_last_child_prefix() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    view_state.add_scanned_child(make_file_item("only", 100));

    let root = view_state.item_tree[0].borrow();
    let child_prefix = root.children[0].borrow().tree_prefix.clone();
    // Only child is last - should have └── prefix.
    assert_eq!(" └──", child_prefix);
}

#[test]
fn add_scanned_child_second_child_updates_first_childs_prefix_to_non_last() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    // First child (will be at index 0, largest - not last after second added).
    view_state.add_scanned_child(make_file_item("large", 5000));
    // Second child (smallest - inserted at end, becomes last).
    view_state.add_scanned_child(make_file_item("small", 100));

    let root = view_state.item_tree[0].borrow();
    let first_prefix = root.children[0].borrow().tree_prefix.clone();
    let last_prefix = root.children[1].borrow().tree_prefix.clone();

    // First child (not last) should have ├── prefix.
    assert_eq!(" ├──", first_prefix);
    // Second child (last) should have └── prefix.
    assert_eq!(" └──", last_prefix);
}

#[test]
fn add_scanned_child_insert_at_front_gives_new_child_non_last_prefix() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));

    // Small is added first and becomes last.
    view_state.add_scanned_child(make_file_item("small", 100));
    // Large inserted at front (larger size) - new first, not last.
    view_state.add_scanned_child(make_file_item("large", 9000));

    let root = view_state.item_tree[0].borrow();
    let large_prefix = root.children[0].borrow().tree_prefix.clone();
    let small_prefix = root.children[1].borrow().tree_prefix.clone();

    assert_eq!(" ├──", large_prefix);
    // Small remains last with └── prefix.
    assert_eq!(" └──", small_prefix);
}
