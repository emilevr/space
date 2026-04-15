use crate::cli::view_state::ViewState;
use crate::cli::view_state_test_utils::{
    make_test_view_state, select_item_by_name, TEST_DIRECTORY_TREE_ITEM_COUNT,
};
use crate::test_directory_utils::delete_test_directory_tree;
use rstest::rstest;
#[rstest]
#[case("", 2)]
#[case("1", 14)]
#[case("1.5.3", TEST_DIRECTORY_TREE_ITEM_COUNT - 5)]
#[case("1.5.3.5", TEST_DIRECTORY_TREE_ITEM_COUNT)]
fn collapse_selected_children_filters_correctly(
    #[case] selected_item_name: &str,
    #[case] expected_displayable_item_count: usize,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    select_item_by_name(selected_item_name, &mut view_state)?;

    // Act
    view_state.collapse_selected_children();
    view_state.update_visible_rows();

    // Assert
    assert_eq!(
        expected_displayable_item_count,
        view_state.displayable_item_count
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case("", 2)]
#[case("1", 14)]
fn expand_selected_children(
    #[case] selected_item_name: &str,
    #[case] expected_displayable_item_count: usize,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    select_item_by_name(selected_item_name, &mut view_state)?;
    view_state.collapse_selected_children();
    view_state.update_visible_rows();
    assert_eq!(
        expected_displayable_item_count,
        view_state.displayable_item_count
    );

    // Act
    view_state.expand_selected_children();
    view_state.update_visible_rows();

    // Assert
    assert_eq!(
        TEST_DIRECTORY_TREE_ITEM_COUNT,
        view_state.displayable_item_count
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

// ─── Tests for visible_rows_dirty in navigation mutations ─────────────────────

#[rstest]
fn first_sets_visible_rows_dirty() {
    let mut view_state = ViewState::default();
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.first();

    assert!(view_state.visible_rows_dirty);
}

#[rstest]
fn previous_sets_visible_rows_dirty_when_offset_changes() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    // Move to the end so visible_offset is non-zero.
    view_state.last();
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);
    assert!(view_state.visible_offset > 0);

    // Navigate up enough to change the offset.
    view_state.previous(view_state.table_selected_index + 1);

    assert!(view_state.visible_rows_dirty);

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}

#[rstest]
fn previous_does_not_set_visible_rows_dirty_when_offset_unchanged() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    // Move down a bit so there's room to go back without changing offset.
    view_state.next(1);
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);
    assert_eq!(0, view_state.visible_offset);

    view_state.previous(1);

    assert!(!view_state.visible_rows_dirty);

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}

#[rstest]
fn next_sets_visible_rows_dirty_when_offset_changes() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    // Navigate far enough to push visible_offset past 0.
    view_state.next(view_state.visible_height);

    assert!(view_state.visible_rows_dirty);

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}

#[rstest]
fn next_does_not_set_visible_rows_dirty_when_offset_unchanged() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.next(1);

    assert!(!view_state.visible_rows_dirty);

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}

#[rstest]
fn previous_does_not_set_visible_rows_dirty_at_first_item() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    // Ensure we are at the very first item (offset=0, index=0).
    view_state.first();
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);
    assert_eq!(0, view_state.visible_offset);
    assert_eq!(0, view_state.table_selected_index);

    // Calling previous at the boundary should be a no-op on the offset.
    view_state.previous(1);

    assert!(!view_state.visible_rows_dirty);

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}

#[rstest]
fn next_does_not_set_visible_rows_dirty_at_last_item() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    // Move to the last item.
    view_state.last();
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);
    let offset_at_last = view_state.visible_offset;
    assert!(offset_at_last > 0);

    // Calling next at the boundary should clamp and not change the offset.
    view_state.next(1);

    assert!(!view_state.visible_rows_dirty);

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}

#[rstest]
fn last_sets_visible_rows_dirty() -> anyhow::Result<()> {
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    view_state.update_visible_rows(); // clear dirty
    assert!(!view_state.visible_rows_dirty);

    view_state.last();

    assert!(view_state.visible_rows_dirty);

    delete_test_directory_tree(&temp_dir_path);
    Ok(())
}
