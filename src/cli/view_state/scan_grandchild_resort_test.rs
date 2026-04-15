use crate::cli::view_state::ViewState;
use space_rs::{DirectoryItem, DirectoryItemType, Size};

// Helpers are duplicated from scan_grandchild_test.rs to keep this module independent.

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

// ─── Parent re-sorting among root's children ─────────────────────────────────

/// Helper: returns the names of root's children in current order.
fn child_names(view_state: &ViewState) -> Vec<String> {
    view_state.item_tree[0]
        .borrow()
        .children
        .iter()
        .map(|c| c.borrow().path_segment.clone())
        .collect()
}

/// Helper: returns the sizes of root's children in current order.
fn child_sizes(view_state: &ViewState) -> Vec<u64> {
    view_state.item_tree[0]
        .borrow()
        .children
        .iter()
        .map(|c| c.borrow().size.get_value())
        .collect()
}

#[test]
fn add_scanned_grandchild_resorts_parent_to_first_position_when_it_becomes_largest() {
    // Setup: root with two dir children, small_dir starts smaller than big_dir.
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("big_dir"));
    view_state.add_scanned_child(make_empty_dir("small_dir"));

    // Give big_dir some size so it sorts first.
    view_state.add_scanned_grandchild("big_dir", make_file_item("f1", 100));
    assert_eq!(vec!["big_dir", "small_dir"], child_names(&view_state));

    // Add a large grandchild to small_dir so it overtakes big_dir.
    view_state.add_scanned_grandchild("small_dir", make_file_item("huge", 500));

    // small_dir should now be at position 0 (largest).
    assert_eq!(vec!["small_dir", "big_dir"], child_names(&view_state));
}

#[test]
fn add_scanned_grandchild_root_children_remain_in_descending_size_order() {
    // Three dir children; grandchild additions shift their relative sizes.
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));
    view_state.add_scanned_child(make_empty_dir("dir_c"));

    view_state.add_scanned_grandchild("dir_a", make_file_item("f", 300));
    view_state.add_scanned_grandchild("dir_b", make_file_item("f", 200));
    view_state.add_scanned_grandchild("dir_c", make_file_item("f", 100));

    // dir_c grows to beat everyone.
    view_state.add_scanned_grandchild("dir_c", make_file_item("g", 1000));

    let sizes = child_sizes(&view_state);
    assert!(
        sizes.windows(2).all(|w| w[0] >= w[1]),
        "root's children must remain in descending size order, got: {sizes:?}"
    );
    assert_eq!(vec!["dir_c", "dir_a", "dir_b"], child_names(&view_state));
}

#[test]
fn add_scanned_grandchild_no_resort_when_parent_stays_at_same_position() {
    // Largest dir remains largest after receiving more grandchildren.
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("big"));
    view_state.add_scanned_child(make_empty_dir("small"));

    view_state.add_scanned_grandchild("big", make_file_item("f1", 1000));
    view_state.add_scanned_grandchild("small", make_file_item("f2", 10));
    assert_eq!(vec!["big", "small"], child_names(&view_state));

    // Add more to big; it should remain at position 0.
    view_state.add_scanned_grandchild("big", make_file_item("f3", 50));

    assert_eq!(vec!["big", "small"], child_names(&view_state));
}

#[test]
fn add_scanned_grandchild_resorts_parent_from_middle_to_first_not_last_branch() {
    // [A(1000), B(500), C(100)] - B moves from middle (index=1) to first (index=0).
    // old_index=1 != original_len-1=2, so only the moved child's prefix is updated;
    // C remains last and its prefix must stay unchanged (└──).
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));
    view_state.add_scanned_child(make_empty_dir("dir_c"));

    view_state.add_scanned_grandchild("dir_a", make_file_item("f", 1000));
    view_state.add_scanned_grandchild("dir_b", make_file_item("f", 500));
    view_state.add_scanned_grandchild("dir_c", make_file_item("f", 100));
    assert_eq!(vec!["dir_a", "dir_b", "dir_c"], child_names(&view_state));

    // dir_b overtakes dir_a -> moves from index 1 to index 0.
    view_state.add_scanned_grandchild("dir_b", make_file_item("g", 2000));

    assert_eq!(vec!["dir_b", "dir_a", "dir_c"], child_names(&view_state));

    // dir_c is still last - its prefix must end with └──.
    let root = view_state.item_tree[0].borrow();
    let last_prefix = root.children[2].borrow().tree_prefix.clone();
    assert!(
        last_prefix.ends_with("└──") || last_prefix.ends_with("└─┬"),
        "last child should keep its 'last' prefix, got: {last_prefix:?}"
    );
}

#[test]
fn add_scanned_grandchild_resort_fixes_prefix_of_new_last_child_when_old_last_moves_earlier() {
    // [dir_a(1000), dir_b(500), dir_c(100,last)] - dir_c (previously last) grows to 5000.
    // After resort: [dir_c(5000), dir_a(1000), dir_b(500)].
    // dir_b is now last and must gain the "└──" prefix.
    // dir_c was last and must now have the "├──" prefix (non-last).
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("dir_a"));
    view_state.add_scanned_child(make_empty_dir("dir_b"));
    view_state.add_scanned_child(make_empty_dir("dir_c"));

    view_state.add_scanned_grandchild("dir_a", make_file_item("f", 1000));
    view_state.add_scanned_grandchild("dir_b", make_file_item("f", 500));
    view_state.add_scanned_grandchild("dir_c", make_file_item("f", 100));
    assert_eq!(vec!["dir_a", "dir_b", "dir_c"], child_names(&view_state));

    // dir_c was last (index=2); add a huge grandchild so it moves to first.
    view_state.add_scanned_grandchild("dir_c", make_file_item("huge", 5000));

    assert_eq!(vec!["dir_c", "dir_a", "dir_b"], child_names(&view_state));

    let root = view_state.item_tree[0].borrow();
    let first_prefix = root.children[0].borrow().tree_prefix.clone();
    let last_prefix = root.children[2].borrow().tree_prefix.clone();

    // dir_c moved to first: must now be non-last (├──).
    assert!(
        first_prefix.ends_with("├──") || first_prefix.ends_with("├─┬"),
        "dir_c (now first) should have non-last prefix, got: {first_prefix:?}"
    );
    // dir_b became last: must have last-child prefix (└──).
    assert!(
        last_prefix.ends_with("└──") || last_prefix.ends_with("└─┬"),
        "dir_b (now last) should have last-child prefix, got: {last_prefix:?}"
    );
}

#[test]
fn add_scanned_grandchild_resort_no_crash_with_single_root_child() {
    // Only one child in root - resort is a structural no-op (position stays 0).
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("only_dir"));

    view_state.add_scanned_grandchild("only_dir", make_file_item("f", 100));
    view_state.add_scanned_grandchild("only_dir", make_file_item("g", 200));

    // Must not crash; the single child must remain at position 0.
    assert_eq!(vec!["only_dir"], child_names(&view_state));
    let root = view_state.item_tree[0].borrow();
    assert_eq!(1, root.children.len());
}

// ─── Tree prefix updates ──────────────────────────────────────────────────────

#[test]
fn add_scanned_grandchild_first_grandchild_updates_parent_prefix_to_branch() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");
    // After add_scanned_child, parent_dir is the only child of root -> has └── prefix.
    {
        let root = view_state.item_tree[0].borrow();
        let parent_prefix = root.children[0].borrow().tree_prefix.clone();
        assert!(
            parent_prefix.ends_with("──"),
            "expected leaf prefix, got: {parent_prefix:?}"
        );
    }

    view_state.add_scanned_grandchild("parent_dir", make_file_item("first.txt", 100));

    let root = view_state.item_tree[0].borrow();
    let parent_prefix = root.children[0].borrow().tree_prefix.clone();
    // After first grandchild, parent should have a branch marker (─┬).
    assert!(
        parent_prefix.ends_with("─┬"),
        "expected branch prefix after first grandchild, got: {parent_prefix:?}"
    );
}

#[test]
fn add_scanned_grandchild_single_grandchild_gets_last_child_prefix() {
    let mut view_state = make_view_state_with_parent_dir("parent_dir");

    view_state.add_scanned_grandchild("parent_dir", make_file_item("only.txt", 100));

    let root = view_state.item_tree[0].borrow();
    let parent = root.children[0].borrow();
    let grandchild_prefix = parent.children[0].borrow().tree_prefix.clone();
    // Only grandchild is last - should end with └──.
    assert!(
        grandchild_prefix.ends_with("└──"),
        "expected last-child prefix, got: {grandchild_prefix:?}"
    );
}

// ─── Grandchild prefix accuracy (full prefix, not just suffix) ───────────────

#[test]
fn grandchild_prefix_under_last_parent_uses_spaces_not_branch_chars() {
    // Root with two dir children; parent_b is last (└─┬).
    // Grandchildren of parent_b must show spaces at the parent level, not └─.
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("parent_a"));
    view_state.add_scanned_child(make_empty_dir("parent_b"));

    // parent_a stays largest.
    view_state.add_scanned_grandchild("parent_a", make_file_item("fa", 1000));

    // parent_b is last; add two grandchildren.
    view_state.add_scanned_grandchild("parent_b", make_file_item("fb1", 300));
    view_state.add_scanned_grandchild("parent_b", make_file_item("fb2", 200));

    let root = view_state.item_tree[0].borrow();
    let parent_b = root.children[1].borrow();
    assert!(
        parent_b.tree_prefix.ends_with("└─┬"),
        "parent_b should be last with children, got: {:?}",
        parent_b.tree_prefix
    );

    // Grandchildren under a last parent (└) must not have └ or ─ from the
    // parent level in their prefix - only spaces for the empty continuation column.
    let gc1_prefix = parent_b.children[0].borrow().tree_prefix.clone();
    let gc2_prefix = parent_b.children[1].borrow().tree_prefix.clone();

    assert_eq!(
        "   ├──", gc1_prefix,
        "non-last grandchild under last parent"
    );
    assert_eq!("   └──", gc2_prefix, "last grandchild under last parent");
}

#[test]
fn grandchild_prefix_under_non_last_parent_shows_continuation_line() {
    // Root with two dir children; parent_a is non-last (├─┬).
    // Grandchildren of parent_a must show │ at the parent level.
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_dir("/root"));
    view_state.add_scanned_child(make_empty_dir("parent_a"));
    view_state.add_scanned_child(make_empty_dir("parent_b"));

    // parent_a gets grandchildren.
    view_state.add_scanned_grandchild("parent_a", make_file_item("fa1", 1000));
    view_state.add_scanned_grandchild("parent_a", make_file_item("fa2", 500));

    // parent_b stays smaller so parent_a remains first (non-last).
    view_state.add_scanned_grandchild("parent_b", make_file_item("fb", 100));

    let root = view_state.item_tree[0].borrow();
    let parent_a = root.children[0].borrow();
    assert!(
        parent_a.tree_prefix.ends_with("├─┬"),
        "parent_a should be non-last with children, got: {:?}",
        parent_a.tree_prefix
    );

    let gc1_prefix = parent_a.children[0].borrow().tree_prefix.clone();
    let gc2_prefix = parent_a.children[1].borrow().tree_prefix.clone();

    assert_eq!(
        " │ ├──", gc1_prefix,
        "non-last grandchild under non-last parent"
    );
    assert_eq!(
        " │ └──", gc2_prefix,
        "last grandchild under non-last parent"
    );
}
