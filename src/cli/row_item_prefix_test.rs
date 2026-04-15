use crate::cli::row_item::{RowItem, RowItemType};
use space_rs::Size;
use std::{cell::RefCell, rc::Rc};

fn make_child_rc(name: &str) -> Rc<RefCell<RowItem>> {
    Rc::new(RefCell::new(RowItem {
        size: Size::new(0),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: 0f32,
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

fn make_parent_with_children(children: Vec<Rc<RefCell<RowItem>>>) -> RowItem {
    RowItem {
        size: Size::new(0),
        has_children: !children.is_empty(),
        expanded: true,
        tree_prefix: "─┬".to_string(),
        item_type: RowItemType::Directory,
        incl_fraction: 0f32,
        peer_fraction: 0.0,
        path_segment: "root".to_string(),
        children,
        parent: None,
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }
}

// ─── Tests for update_inserted_child_prefix ───────────────────────────────────

#[test]
fn update_inserted_child_prefix_single_child_gets_last_child_prefix() {
    let child = make_child_rc("only");
    let mut parent = make_parent_with_children(vec![child.clone()]);

    parent.update_inserted_child_prefix(0, " ");

    assert_eq!(" └──", child.borrow().tree_prefix);
}

#[test]
fn update_inserted_child_prefix_new_last_child_gets_last_prefix_and_previous_updated() {
    let first = make_child_rc("first");
    let second = make_child_rc("second");
    // Simulate first being the only child (was last): set its prefix as if it was last.
    first.borrow_mut().tree_prefix = " └──".to_string();
    let mut parent = make_parent_with_children(vec![first.clone(), second.clone()]);

    // Insert second at index 1 (new last).
    parent.update_inserted_child_prefix(1, " ");

    // New last child gets └── prefix.
    assert_eq!(" └──", second.borrow().tree_prefix);
    // Previously-last child is no longer last - gets ├── prefix.
    assert_eq!(" ├──", first.borrow().tree_prefix);
}

#[test]
fn update_inserted_child_prefix_insert_at_front_gives_non_last_prefix() {
    // Existing last child with its correct last prefix.
    let existing_last = make_child_rc("last");
    existing_last.borrow_mut().tree_prefix = " └──".to_string();
    let new_first = make_child_rc("new_first");
    let mut parent = make_parent_with_children(vec![new_first.clone(), existing_last.clone()]);

    // Insert at index 0 (not last).
    parent.update_inserted_child_prefix(0, " ");

    // New child at front is not last - gets ├── prefix.
    assert_eq!(" ├──", new_first.borrow().tree_prefix);
    // Existing last child is still last - its prefix should remain └──.
    assert_eq!(" └──", existing_last.borrow().tree_prefix);
}

#[test]
fn update_inserted_child_prefix_insert_at_middle_only_updates_new_child() {
    let large = make_child_rc("large");
    let medium = make_child_rc("medium");
    let small = make_child_rc("small");
    // Simulate pre-existing prefixes for large (non-last) and small (last).
    large.borrow_mut().tree_prefix = " ├──".to_string();
    small.borrow_mut().tree_prefix = " └──".to_string();
    let mut parent = make_parent_with_children(vec![large.clone(), medium.clone(), small.clone()]);

    // Insert medium at index 1 (middle, not last).
    parent.update_inserted_child_prefix(1, " ");

    // New middle child is not last - gets ├── prefix.
    assert_eq!(" ├──", medium.borrow().tree_prefix);
    // Neighbors are unchanged.
    assert_eq!(" ├──", large.borrow().tree_prefix);
    assert_eq!(" └──", small.borrow().tree_prefix);
}
