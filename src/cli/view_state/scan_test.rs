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
fn view_state_default_total_size_in_bytes_is_zero() {
    let view_state = ViewState::default();
    assert_eq!(0, view_state.total_size_in_bytes);
}

#[test]
fn add_scanned_item_updates_total_size_and_adds_to_tree() {
    let mut view_state = ViewState::default();

    view_state.add_scanned_item(make_file_item("a", 1000));

    assert_eq!(1000, view_state.total_size_in_bytes);
    assert_eq!(1, view_state.item_tree.len());
    assert_eq!(1, view_state.total_items_in_tree);
}

#[test]
fn add_scanned_item_with_multiple_items_accumulates_size() {
    let mut view_state = ViewState::default();

    view_state.add_scanned_item(make_file_item("a", 1000));
    view_state.add_scanned_item(make_file_item("b", 3000));

    assert_eq!(4000, view_state.total_size_in_bytes);
    assert_eq!(2, view_state.item_tree.len());
    assert_eq!(2, view_state.total_items_in_tree);
}

#[test]
fn add_scanned_item_with_directory_counts_descendants_in_total_items() {
    let mut view_state = ViewState::default();

    let dir_item = DirectoryItem {
        path_segment: "dir".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(5000),
        descendant_count: 3,
        children: vec![
            make_file_item("child1", 1000),
            make_file_item("child2", 2000),
            make_file_item("child3", 2000),
        ],
    };

    view_state.add_scanned_item(dir_item);

    // Root + 3 descendants = 4 total items.
    assert_eq!(4, view_state.total_items_in_tree);
}

#[test]
fn recalculate_fractions_corrects_stale_fractions_after_multiple_adds() {
    let mut view_state = ViewState::default();

    view_state.add_scanned_item(make_file_item("a", 1000));
    view_state.add_scanned_item(make_file_item("b", 3000));

    view_state.recalculate_fractions();

    let fraction_a = view_state.item_tree[0].borrow().incl_fraction;
    let fraction_b = view_state.item_tree[1].borrow().incl_fraction;

    assert!(
        (fraction_a - 0.25f32).abs() < 0.001f32,
        "Expected fraction_a ≈ 0.25, got {fraction_a}"
    );
    assert!(
        (fraction_b - 0.75f32).abs() < 0.001f32,
        "Expected fraction_b ≈ 0.75, got {fraction_b}"
    );
}

#[test]
fn recalculate_fractions_with_zero_total_sets_all_fractions_to_zero() {
    let mut view_state = ViewState::default();

    view_state.add_scanned_item(make_file_item("a", 1000));
    view_state.total_size_in_bytes = 0;

    view_state.recalculate_fractions();

    let fraction = view_state.item_tree[0].borrow().incl_fraction;
    assert_eq!(0f32, fraction);
}

// ─── Tests for visible_rows_dirty flag ───────────────────────────────────────

#[test]
fn visible_rows_dirty_defaults_to_true() {
    let view_state = ViewState::default();
    assert!(view_state.visible_rows_dirty);
}

#[test]
fn update_visible_rows_clears_dirty_flag() {
    let mut view_state = ViewState::default();
    assert!(view_state.visible_rows_dirty);

    view_state.update_visible_rows();

    assert!(!view_state.visible_rows_dirty);
}

#[test]
fn update_visible_rows_when_not_dirty_does_not_re_walk_tree() {
    let mut view_state = ViewState {
        visible_height: 10,
        ..ViewState::default()
    };
    view_state.add_scanned_item(make_empty_root("/root"));
    view_state.add_scanned_child(make_file_item("child", 100));

    // First call: full tree walk, clears dirty, populates cache.
    view_state.update_visible_rows();
    assert!(!view_state.visible_rows_dirty);
    let cached_count = view_state.visible_row_items.len();
    assert!(cached_count > 0);

    // Second call: dirty=false, should use cache (not clear visible_row_items).
    view_state.update_visible_rows();
    assert_eq!(cached_count, view_state.visible_row_items.len());
    assert!(!view_state.visible_rows_dirty);
}

#[test]
fn add_scanned_item_sets_visible_rows_dirty() {
    let mut view_state = ViewState::default();
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.add_scanned_item(make_file_item("x", 100));

    assert!(view_state.visible_rows_dirty);
}

#[test]
fn add_scanned_child_sets_visible_rows_dirty() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.add_scanned_child(make_file_item("child", 100));

    assert!(view_state.visible_rows_dirty);
}

#[test]
fn sort_root_children_sets_visible_rows_dirty() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_empty_root("/root"));
    view_state.add_scanned_child(make_file_item("a", 100));
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.sort_root_children();

    assert!(view_state.visible_rows_dirty);
}

#[test]
fn recalculate_fractions_sets_visible_rows_dirty() {
    let mut view_state = ViewState::default();
    view_state.add_scanned_item(make_file_item("a", 1000));
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.recalculate_fractions();

    assert!(view_state.visible_rows_dirty);
}
