use super::ViewState;
use crate::cli::view_state_test_utils::make_test_view_state_with_height;
use crate::test_directory_utils::delete_test_directory_tree;
use rstest::rstest;

// Note: make_test_view_state_with_expanded_dir is also defined in view_state_test.rs-
// duplicated here to avoid a shared module dependency.
fn make_test_view_state_with_expanded_dir() -> ViewState {
    use space_rs::{DirectoryItem, DirectoryItemType, Size};
    let root = DirectoryItem {
        path_segment: "/root".to_string(),
        item_type: DirectoryItemType::Directory,
        size_in_bytes: Size::new(1000),
        descendant_count: 1,
        children: vec![DirectoryItem {
            path_segment: "child".to_string(),
            item_type: DirectoryItemType::File,
            size_in_bytes: Size::new(1000),
            descendant_count: 0,
            children: vec![],
        }],
    };
    let mut view_state = ViewState {
        visible_height: 10,
        ..ViewState::default()
    };
    view_state.add_scanned_item(root);
    view_state.update_visible_rows();
    view_state
}
#[rstest]
fn get_selected_item_given_no_selected_item_returns_none() {
    // Arrange
    let view_state = ViewState::default();

    // Act
    let selected_item = view_state.get_selected_item();

    // Assert
    assert!(selected_item.is_none());
}

#[rstest]
#[case(3, 0, 3)] // First invisible item.
#[case(3, 27, 2)] // Near end - only last 2 items visible.
fn get_selected_item_given_non_visible_or_non_existant_selected_item_returns_none(
    #[case] visible_height: usize,
    #[case] visible_offset: usize,
    #[case] selected_index: usize,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) =
        make_test_view_state_with_height(visible_height, visible_offset, 0f32)?;
    view_state.table_selected_index = selected_index;

    // Act
    let selected_item = view_state.get_selected_item();

    // Assert
    assert!(selected_item.is_none());

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case(3, 0, 1, 1)]
#[case(7, 7, 5, 12)]
#[case(5, 22, 2, 24)]
fn get_selected_item_given_visible_selected_item_returns_selected_item(
    #[case] visible_height: usize,
    #[case] visible_offset: usize,
    #[case] visible_selected_index: usize,
    #[case] expected_selected_row_index: usize,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) =
        make_test_view_state_with_height(visible_height, visible_offset, 0f32)?;
    view_state.table_selected_index = visible_selected_index;

    // Act
    let selected_item = view_state.get_selected_item();

    // Assert
    assert!(selected_item.is_some());
    assert_eq!(
        expected_selected_row_index,
        selected_item.unwrap().borrow().row_index
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

// ─── Tests for visible_rows_dirty in expand/collapse mutations ────────────────

#[rstest]
fn expand_selected_item_sets_visible_rows_dirty() {
    // Arrange: directory with a child, collapsed so expand_selected_item will act.
    let mut view_state = make_test_view_state_with_expanded_dir();
    // Collapse the root so expand_selected_item has something to do.
    view_state.item_tree[0].borrow_mut().expanded = false;
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    // Act
    view_state.expand_selected_item();

    // Assert
    assert!(view_state.visible_rows_dirty);
}

#[rstest]
fn collapse_selected_item_sets_visible_rows_dirty() {
    // Arrange: directory with children, explicitly expanded.
    let mut view_state = make_test_view_state_with_expanded_dir();
    view_state.item_tree[0].borrow_mut().expanded = true;
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    // Act: collapse the expanded root directory.
    view_state.collapse_selected_item();

    // Assert
    assert!(view_state.visible_rows_dirty);
}

#[rstest]
fn expand_selected_children_sets_visible_rows_dirty() {
    let mut view_state = make_test_view_state_with_expanded_dir();
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.expand_selected_children();

    assert!(view_state.visible_rows_dirty);
}

#[rstest]
fn collapse_selected_children_sets_visible_rows_dirty() {
    let mut view_state = make_test_view_state_with_expanded_dir();
    view_state.item_tree[0].borrow_mut().expanded = true;
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.collapse_selected_children();

    assert!(view_state.visible_rows_dirty);
}
