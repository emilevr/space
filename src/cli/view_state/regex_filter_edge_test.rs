use crate::cli::row_item::{RowItem, RowItemType};
use crate::cli::skin::Skin;
use crate::cli::view_state::ViewState;
use regex::RegexBuilder;
use space_rs::{Size, SizeDisplayFormat};
use std::{cell::RefCell, rc::Rc};

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

// --- Integration with add_table_row ------------------------------------------

#[test]
fn add_table_row_skips_regex_hidden_items() {
    use crate::cli::view_state::table_rows::add_table_row;
    use ratatui::widgets::Row;

    let visible_item = make_item("match_me.txt", 1.0);
    let hidden_item = make_item("hidden.txt", 1.0);
    hidden_item.borrow_mut().regex_visible = false;

    let skin = Skin::default();
    let mut rows: Vec<Row> = vec![];
    let mut row_items = vec![];
    let mut row_index = 0;
    let mut added_count = 0;
    let mut displayable_count = 0;

    add_table_row(
        &mut rows,
        &mut row_items,
        visible_item.clone(),
        SizeDisplayFormat::Metric,
        0.0,
        0,
        100,
        &mut row_index,
        &mut added_count,
        &mut displayable_count,
        &skin,
        0,
        0,
        0,
    );
    add_table_row(
        &mut rows,
        &mut row_items,
        hidden_item.clone(),
        SizeDisplayFormat::Metric,
        0.0,
        0,
        100,
        &mut row_index,
        &mut added_count,
        &mut displayable_count,
        &skin,
        0,
        0,
        0,
    );

    assert_eq!(
        1, displayable_count,
        "only the visible item should be counted"
    );
    assert_eq!(1, rows.len(), "only one row should be rendered");
}

// --- Edge cases --------------------------------------------------------------

#[test]
fn apply_regex_filter_on_empty_tree_does_not_panic() {
    let mut vs = make_view_state_with_tree(vec![], None);
    // Should not panic on empty tree
    vs.apply_regex_filter();
}

#[test]
fn apply_regex_filter_marks_visible_rows_dirty() {
    let item = make_item("file.rs", 1.0);
    let mut vs = make_view_state_with_tree(vec![item], None);
    vs.visible_rows_dirty = false;

    vs.apply_regex_filter();

    assert!(
        vs.visible_rows_dirty,
        "apply_regex_filter should mark visible_rows_dirty"
    );
}

#[test]
fn set_filter_regex_marks_visible_rows_dirty() {
    let item = make_item("file.rs", 1.0);
    let mut vs = make_view_state_with_tree(vec![item], None);
    vs.visible_rows_dirty = false;

    let regex = build_case_insensitive_regex("file");
    vs.set_filter_regex(Some(regex));

    assert!(
        vs.visible_rows_dirty,
        "set_filter_regex should mark visible_rows_dirty"
    );
}

#[test]
fn apply_regex_filter_matches_across_path_segments() {
    // Regex matching "project/src" should match the accumulated path of the child dir.
    let child = make_item("utils.rs", 0.5);
    let src_dir = make_dir("src", 0.7, vec![child.clone()]);
    let root = make_dir("project", 1.0, vec![src_dir.clone()]);

    let regex = build_case_insensitive_regex("project/src");
    let mut vs = make_view_state_with_tree(vec![root.clone()], Some(regex));
    vs.apply_regex_filter();

    assert!(
        src_dir.borrow().regex_visible,
        "src dir path 'project/src' should match the regex"
    );
    assert!(
        root.borrow().regex_visible,
        "root should be visible as ancestor of matching src dir"
    );
}

#[test]
fn apply_regex_filter_sibling_isolation() {
    // Only one of two sibling dirs matches; the non-matching sibling should be hidden.
    let tests_child = make_item("mod.rs", 0.3);
    let tests_dir = make_dir("tests", 0.5, vec![tests_child.clone()]);
    let docs_child = make_item("readme.md", 0.3);
    let docs_dir = make_dir("docs", 0.5, vec![docs_child.clone()]);
    let root = make_dir("root", 1.0, vec![tests_dir.clone(), docs_dir.clone()]);

    let regex = build_case_insensitive_regex("tests");
    let mut vs = make_view_state_with_tree(vec![root.clone()], Some(regex));
    vs.apply_regex_filter();

    assert!(
        tests_dir.borrow().regex_visible,
        "matching sibling should be visible"
    );
    assert!(
        !docs_dir.borrow().regex_visible,
        "non-matching sibling should be hidden"
    );
    assert!(
        !docs_child.borrow().regex_visible,
        "child of non-matching sibling should be hidden"
    );
    assert!(
        root.borrow().regex_visible,
        "root should be visible as ancestor of match"
    );
}

#[test]
fn apply_regex_filter_with_regex_that_matches_nothing_hides_all() {
    let file_a = make_item("alpha.rs", 0.5);
    let file_b = make_item("beta.rs", 0.5);
    let root = make_dir("root", 1.0, vec![file_a.clone(), file_b.clone()]);

    let regex = build_case_insensitive_regex("zzz_no_match_xyz");
    let mut vs = make_view_state_with_tree(vec![root.clone()], Some(regex));
    vs.apply_regex_filter();

    assert!(!file_a.borrow().regex_visible, "file_a should be hidden");
    assert!(!file_b.borrow().regex_visible, "file_b should be hidden");
    assert!(
        !root.borrow().regex_visible,
        "root with no matching descendants should be hidden"
    );
}
