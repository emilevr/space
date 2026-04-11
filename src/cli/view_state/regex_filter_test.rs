use crate::cli::row_item::{RowItem, RowItemType};
use crate::cli::skin::Skin;
use crate::cli::view_state::ViewState;
use regex::RegexBuilder;
use space_rs::{Size, SizeDisplayFormat};
use std::{cell::RefCell, rc::Rc};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn make_item(name: &str, fraction: f32) -> Rc<RefCell<RowItem>> {
    Rc::new(RefCell::new(RowItem {
        size: Size::new(1000),
        has_children: false,
        expanded: true,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: fraction,
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

fn make_dir(
    name: &str,
    fraction: f32,
    children: Vec<Rc<RefCell<RowItem>>>,
) -> Rc<RefCell<RowItem>> {
    let item = Rc::new(RefCell::new(RowItem {
        size: Size::new(5000),
        has_children: !children.is_empty(),
        expanded: true,
        tree_prefix: String::default(),
        item_type: RowItemType::Directory,
        incl_fraction: fraction,
        peer_fraction: 0.0,
        path_segment: name.to_string(),
        children: children.clone(),
        parent: None,
        descendant_count: children.len(),
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }));
    for child in &children {
        child.borrow_mut().parent = Some(Rc::downgrade(&item));
    }
    item
}

fn build_case_insensitive_regex(pattern: &str) -> regex::Regex {
    RegexBuilder::new(pattern)
        .case_insensitive(true)
        .build()
        .expect("valid test regex")
}

fn make_view_state_with_tree(
    tree: Vec<Rc<RefCell<RowItem>>>,
    filter: Option<regex::Regex>,
) -> ViewState {
    ViewState::new(
        tree,
        SizeDisplayFormat::Metric,
        0.0,
        filter,
        &Skin::default(),
    )
}

// ─── Tests for apply_regex_filter ────────────────────────────────────────────

#[test]
fn apply_regex_filter_with_no_regex_sets_all_visible() {
    let child = make_item("notes.txt", 0.1);
    let root = make_dir("root", 1.0, vec![child.clone()]);
    child.borrow_mut().regex_visible = false;
    root.borrow_mut().regex_visible = false;

    let mut vs = make_view_state_with_tree(vec![root.clone()], None);
    vs.apply_regex_filter();

    assert!(root.borrow().regex_visible, "root should be visible");
    assert!(child.borrow().regex_visible, "child should be visible");
}

#[test]
fn apply_regex_filter_matching_item_is_visible() {
    let matching = make_item("src_main.rs", 0.5);
    let non_matching = make_item("readme.md", 0.5);
    let root = make_dir("project", 1.0, vec![matching.clone(), non_matching.clone()]);

    let regex = build_case_insensitive_regex("src");
    let mut vs = make_view_state_with_tree(vec![root.clone()], Some(regex));
    vs.apply_regex_filter();

    assert!(
        matching.borrow().regex_visible,
        "matching item should be visible"
    );
    assert!(
        !non_matching.borrow().regex_visible,
        "non-matching item should be hidden"
    );
}

#[test]
fn apply_regex_filter_ancestor_of_matching_item_is_visible() {
    let leaf = make_item("test_helper.rs", 0.3);
    let parent_dir = make_dir("tests", 0.5, vec![leaf.clone()]);
    leaf.borrow_mut().parent = Some(Rc::downgrade(&parent_dir));
    let root = make_dir("root", 1.0, vec![parent_dir.clone()]);
    parent_dir.borrow_mut().parent = Some(Rc::downgrade(&root));

    let regex = build_case_insensitive_regex("helper");
    let mut vs = make_view_state_with_tree(vec![root.clone()], Some(regex));
    vs.apply_regex_filter();

    assert!(
        leaf.borrow().regex_visible,
        "matching leaf should be visible"
    );
    assert!(
        parent_dir.borrow().regex_visible,
        "parent of matching leaf should be visible"
    );
    assert!(
        root.borrow().regex_visible,
        "root ancestor of matching leaf should be visible"
    );
}

#[test]
fn apply_regex_filter_non_matching_branch_is_hidden() {
    let unrelated = make_item("config.yaml", 0.3);
    let dir = make_dir("docs", 0.5, vec![unrelated.clone()]);

    let regex = build_case_insensitive_regex("src");
    let mut vs = make_view_state_with_tree(vec![dir.clone()], Some(regex));
    vs.apply_regex_filter();

    assert!(
        !unrelated.borrow().regex_visible,
        "non-matching item should be hidden"
    );
    assert!(
        !dir.borrow().regex_visible,
        "parent with no matching descendants should be hidden"
    );
}

#[test]
fn apply_regex_filter_is_case_insensitive() {
    let lower = make_item("readme.md", 0.5);
    let upper = make_item("README.txt", 0.5);
    let root = make_dir("root", 1.0, vec![lower.clone(), upper.clone()]);

    let regex = build_case_insensitive_regex("readme");
    let mut vs = make_view_state_with_tree(vec![root.clone()], Some(regex));
    vs.apply_regex_filter();

    assert!(
        lower.borrow().regex_visible,
        "lowercase match should be visible"
    );
    assert!(
        upper.borrow().regex_visible,
        "uppercase match should be visible"
    );
}

#[test]
fn set_filter_regex_clears_filter_when_none() {
    let item = make_item("notes.txt", 1.0);
    item.borrow_mut().regex_visible = false;
    let root = make_dir("root", 1.0, vec![item.clone()]);
    root.borrow_mut().regex_visible = false;

    let regex = build_case_insensitive_regex("xyz_no_match");
    let mut vs = make_view_state_with_tree(vec![root.clone()], Some(regex));
    vs.apply_regex_filter();
    assert!(
        !item.borrow().regex_visible,
        "non-matching item should be hidden after filter applied"
    );

    vs.set_filter_regex(None);
    assert!(
        item.borrow().regex_visible,
        "item should be visible after filter cleared"
    );
    assert!(
        root.borrow().regex_visible,
        "root should be visible after filter cleared"
    );
}

#[test]
fn set_filter_regex_resets_selection_and_offset() {
    let item = make_item("file.txt", 1.0);
    let mut vs = make_view_state_with_tree(vec![item], None);
    vs.table_selected_index = 5;
    vs.visible_offset = 3;

    let regex = build_case_insensitive_regex("file");
    vs.set_filter_regex(Some(regex));

    assert_eq!(0, vs.table_selected_index);
    assert_eq!(0, vs.visible_offset);
}
