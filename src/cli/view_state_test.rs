use super::ViewState;
use crate::{
    cli::{
        view_state::subtract_item_tree_size,
        view_state_test_utils::{
            assert_selected_item_name_eq, get_row_index_by_name, make_test_view_state,
            make_test_view_state_with_height, select_item_by_name, TEST_DIRECTORY_TREE_ITEM_COUNT,
        },
    },
    test_directory_utils::delete_test_directory_tree,
};
use rstest::rstest;

#[rstest]
fn set_size_threshold_fraction_with_same_value_does_not_reset_state() {
    // Arrange
    let mut view_state = ViewState::default();
    view_state.set_size_threshold_fraction(0.1f32);
    let selected_index = 2;
    view_state.table_selected_index = selected_index;
    let size_threshold_fraction = view_state.size_threshold_fraction;

    // Act
    view_state.set_size_threshold_fraction(size_threshold_fraction);

    // Assert
    assert_eq!(selected_index, view_state.table_selected_index);
}

#[rstest]
fn set_size_threshold_fraction_given_selected_item_still_visible_reselects_item(
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let item_name = "1.4";
    select_item_by_name(item_name, &mut view_state)?;

    // Act
    view_state.set_size_threshold_fraction(0.01f32);

    // Assert
    assert_selected_item_name_eq(item_name, &view_state);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
fn set_size_threshold_fraction_given_selected_item_no_longer_visible_selects_first_item(
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    select_item_by_name("1.12.1", &mut view_state)?;

    // Act
    view_state.set_size_threshold_fraction(0.01f32);

    // Assert
    assert_eq!(0, view_state.visible_offset);
    assert_eq!(0, view_state.table_selected_index);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
fn expand_selected_item_given_leaf_item_has_no_effect() -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let item_name = "1.1";
    select_item_by_name(item_name, &mut view_state)?;

    // Act
    view_state.expand_selected_item();

    // Assert
    assert_selected_item_name_eq(item_name, &view_state);
    let selected_item = view_state.get_selected_item().unwrap();
    let selected_item = selected_item.borrow();
    assert!(!selected_item.expanded);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
fn expand_selected_item_given_already_expanded_item_has_no_effect() -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let item_name = "1";
    select_item_by_name(item_name, &mut view_state)?;

    // Act
    view_state.expand_selected_item();

    // Assert
    assert_selected_item_name_eq(item_name, &view_state);
    let selected_item = view_state.get_selected_item().unwrap();
    let selected_item = selected_item.borrow();
    assert!(selected_item.has_children);
    assert!(selected_item.expanded);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
fn expand_selected_item_given_no_selected_item_should_succeed() {
    // Arrange
    let view_state = ViewState::default();

    // Act
    view_state.expand_selected_item();

    // Assert
    assert!(view_state.get_selected_item().is_none());
}

#[rstest]
#[case("1.1", "1")]
#[case("1.5.2", "1.5")]
#[case("1.11", "1")]
fn collapse_selected_item_given_leaf_item_collapses_and_selects_parent_item(
    #[case] selected_item_name: &str,
    #[case] parent_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    select_item_by_name(selected_item_name, &mut view_state)?;

    // Act
    view_state.collapse_selected_item();

    // Assert
    assert_selected_item_name_eq(parent_item_name, &view_state);
    let selected_item = view_state.get_selected_item().unwrap();
    let selected_item = selected_item.borrow();
    assert!(!selected_item.expanded);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case("")]
#[case("1.3")]
#[case("1.5.3")]
fn collapse_selected_item_given_parent_item_collapses_and_keeps_selection(
    #[case] selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    select_item_by_name(selected_item_name, &mut view_state)?;

    // Act
    view_state.collapse_selected_item();

    // Assert
    assert_selected_item_name_eq(selected_item_name, &view_state);
    let selected_item = view_state.get_selected_item().unwrap();
    let selected_item = selected_item.borrow();
    assert!(!selected_item.expanded);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
fn collapse_selected_item_given_no_selected_item_should_succeed() {
    // Arrange
    let mut view_state = ViewState::default();

    // Act
    view_state.collapse_selected_item();

    // Assert
    assert!(view_state.get_selected_item().is_none());
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

#[rstest]
#[case(3, 0, 0, 0)]
#[case(3, 0, 1, 0)]
#[case(3, 0, 2, 0)]
#[case(3, 0, 3, 1)]
fn ensure_visible_given_visible_offset_of_zero_should_move_visible_offset_correctly(
    #[case] visible_height: usize,
    #[case] visible_offset: usize,
    #[case] index_to_ensure_visible: usize,
    #[case] expected_visible_offset: usize,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) =
        make_test_view_state_with_height(visible_height, visible_offset, 0f32)?;

    // Act
    view_state.ensure_visible(index_to_ensure_visible);

    // Assert
    assert_eq!(expected_visible_offset, view_state.visible_offset);
    assert_eq!(0, view_state.table_selected_index);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case(3, 14, 15, 14)] // Should not move as already visible
#[case(3, 14, 12, 12)] // Two above current visible_offset -> should move up 2
#[case(3, 14, 17, 15)] // One below current visible -> should move down one
fn ensure_visible_given_visible_offset_of_non_zero_should_move_visible_offset_correctly(
    #[case] visible_height: usize,
    #[case] visible_offset: usize,
    #[case] index_to_ensure_visible: usize,
    #[case] expected_visible_offset: usize,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) =
        make_test_view_state_with_height(visible_height, visible_offset, 0f32)?;

    // Act
    view_state.ensure_visible(index_to_ensure_visible);

    // Assert
    assert_eq!(expected_visible_offset, view_state.visible_offset);
    assert_eq!(0, view_state.table_selected_index);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
// First page, first item selected -> second item selected
#[case(4, 0, 0f32, 0, 1, "1")]
// First page, last item on page selected -> next item that was off page now visible and selected
#[case(4, 0, 0f32, 3, 3, "1.3")]
// First page, with page size larger than number of items, last item selected -> last item remains selected
#[case(10, 0, 0.09f32, 7, 7, "1.5")]
// Last page, last item selected -> last item remains selected
#[case(4, 25, 0f32, 3, 3, "1.12.1")]
// No visible items
#[case(4, 0, 1.1f32, 0, 0, "")]
fn next_selects_expected_item(
    #[case] visible_height: usize,
    #[case] visible_offset: usize,
    #[case] view_size_threshold_fraction: f32,
    #[case] table_selected_index: usize,
    #[case] expected_table_selected_index: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state_with_height(
        visible_height,
        visible_offset,
        view_size_threshold_fraction,
    )?;
    view_state.table_selected_index = table_selected_index;

    // Act
    view_state.next(1);
    view_state.update_visible_rows();

    // Assert
    assert_eq!(
        expected_table_selected_index,
        view_state.table_selected_index
    );
    if view_state.displayable_item_count > 0 {
        assert_selected_item_name_eq(expected_selected_item_name, &view_state);
    }

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case(5, 0, 0, 0, "")] // First page, with no selection -> top row selected
#[case(5, 5, 0, 0, "1.3")] // Second page, with top row selected -> select previous item, that was previously not-visible
#[case(5, 20, 4, 3, "1.9")] // Last page, with last item in tree selected -> selects second last item
fn previous_selects_expected_item(
    #[case] visible_height: usize,
    #[case] visible_offset: usize,
    #[case] table_selected_index: usize,
    #[case] expected_table_selected_index: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) =
        make_test_view_state_with_height(visible_height, visible_offset, 0f32)?;
    view_state.table_selected_index = table_selected_index;

    // Act
    view_state.previous(1);
    view_state.update_visible_rows();

    // Assert
    assert_eq!(
        expected_table_selected_index,
        view_state.table_selected_index
    );
    assert_selected_item_name_eq(expected_selected_item_name, &view_state);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
fn select_item_with_out_of_bounds_row_index_selects_first_item() {
    // Arrange
    let mut view_state = ViewState::default();

    // Act
    view_state.select_item(1);

    // Assert
    assert_eq!(0, view_state.table_selected_index);
}

#[rstest]
fn subtract_item_tree_size_subtracts_value_from_self_and_ancestors() -> anyhow::Result<()> {
    // Arrange
    let value_to_subtract = 10000u64;
    let (view_state, temp_dir_path) = make_test_view_state_with_height(10, 0, 0f32)?;

    // Act
    subtract_item_tree_size(&view_state.visible_row_items[4], value_to_subtract);

    // Assert
    // Check that self has been updated
    assert_eq!(
        13000,
        view_state.visible_row_items[4].borrow().size.get_value()
    );
    // Check that ancestors have been updated
    assert_eq!(
        170000,
        view_state.visible_row_items[1].borrow().size.get_value()
    );
    assert_eq!(
        170000,
        view_state.visible_row_items[0].borrow().size.get_value()
    );
    // Spot check that siblings were not affected
    assert_eq!(
        24000,
        view_state.visible_row_items[3].borrow().size.get_value()
    );
    assert_eq!(
        25000,
        view_state.visible_row_items[2].borrow().size.get_value()
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case("1.3", 25, 157000)] // Delete a directory with 3 sub-items
#[case("1.4", 28, 158000)] // Delete a single file
#[case("1.5", 18, 162000)] // Delete a directory with 10 sub-items
#[case("1.5.3.5", 28, 180000)] // Delete an empty directory
#[case("1.11", 28, 180000)] // Deletes symbolic link to dir, but not the tree it points to
fn delete_selected_item_deletes_only_the_selected_item_or_tree_and_updates_sizes(
    #[case] selected_item_name: &str,
    #[case] expected_final_item_count: usize,
    #[case] expected_total_size: u64,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    select_item_by_name(selected_item_name, &mut view_state)?;

    // Act
    view_state.delete_selected_item();
    view_state.update_visible_rows();

    // Assert
    assert_eq!(None, get_row_index_by_name(selected_item_name, &view_state));
    assert_eq!(expected_final_item_count, view_state.total_items_in_tree);
    assert_eq!(
        expected_total_size,
        view_state.item_tree[0].borrow().size.get_value()
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case(7, 0, 0, "")] // first item selected -> first item selected
#[case(7, 6, 5, "1.5.2")] // 1.5.2 selected -> first item selected
#[case(7, 18, 6, "1.10")] // last item selected -> first item selected
fn first_selects_first_item(
    #[case] visible_height: usize,
    #[case] visible_offset: usize,
    #[case] table_selected_index: usize,
    #[case] expected_preselected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) =
        make_test_view_state_with_height(visible_height, visible_offset, 0f32)?;
    view_state.table_selected_index = table_selected_index;
    assert_selected_item_name_eq(expected_preselected_item_name, &view_state);

    // Act
    view_state.first();

    // Assert
    assert_eq!(0, view_state.visible_offset);
    assert_eq!(0, view_state.table_selected_index);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case(7, 0, 0f32, 0, "")] // first item selected -> last item selected
#[case(7, 6, 0f32, 5, "1.5.2")] // 1.5.2 selected -> last item selected
#[case(7, 18, 0f32, 6, "1.10")] // last item selected -> last item selected
#[case(4, 0, 1.1f32, 0, "")] // No visible items
fn last_selects_last_item(
    #[case] visible_height: usize,
    #[case] visible_offset: usize,
    #[case] view_size_threshold_fraction: f32,
    #[case] table_selected_index: usize,
    #[case] expected_preselected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state_with_height(
        visible_height,
        visible_offset,
        view_size_threshold_fraction,
    )?;
    view_state.table_selected_index = table_selected_index;
    if view_state.displayable_item_count > 0 {
        assert_selected_item_name_eq(expected_preselected_item_name, &view_state);
    }

    // Act
    view_state.last();

    // Assert
    if view_state.displayable_item_count >= view_state.visible_height {
        assert_eq!(
            view_state.displayable_item_count - view_state.visible_height,
            view_state.visible_offset
        );
    }

    if view_state.visible_height >= view_state.displayable_item_count
        && view_state.displayable_item_count > 0
    {
        assert_eq!(
            view_state.visible_height - 1,
            view_state.table_selected_index
        );
    }

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case("", 2)] // select first item -> only first item and its only child visible
#[case("1", 14)] // select 1 -> first item, 1 and 1.1 - 1.10 visible
#[case("1.5.3", TEST_DIRECTORY_TREE_ITEM_COUNT - 5)] // select 1.5.3 -> collapsed up to 1.5.3
#[case("1.5.3.5", TEST_DIRECTORY_TREE_ITEM_COUNT - 5)] // select 1.5.3.5 -> collapsed up to 1.5.3
fn collapse_selected_children_filters_correctly(
    #[case] selected_item_name: &str,
    #[case] expected_displayable_item_count: usize,
) -> anyhow::Result<()> {
    // Arrange
    // It does not matter what visible height value we use, as we are selecting by name.
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
#[case("", 2)] // select first item + collapse its children + expand its children -> all displayable
#[case("1", 14)] // select 1 + collapse its children + expand its children -> all displayable
fn expand_selected_children(
    #[case] selected_item_name: &str,
    #[case] expected_displayable_item_count: usize,
) -> anyhow::Result<()> {
    // Arrange
    // It does not matter what visible height value we use, as we are selecting by name.
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

#[test]
fn read_and_write_config_file_succeeds() -> anyhow::Result<()> {
    // Arrange
    let mut view_state = ViewState::default();
    view_state.accepted_license_terms = true;
    view_state.config_file_path =
        Some(std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4())));

    // Act
    view_state.write_config_file()?; // Write true
    view_state.accepted_license_terms = false;
    view_state.read_config_file()?;

    // Assert
    assert!(view_state.accepted_license_terms);

    Ok(())
}

#[test]
fn accept_license_terms_updates_view_state_and_writes_config_file() -> anyhow::Result<()> {
    // Arrange
    let mut view_state = ViewState::default();
    view_state.accepted_license_terms = false;
    view_state.config_file_path =
        Some(std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4())));

    // Act
    view_state.accept_license_terms();

    // Assert
    assert!(view_state.accepted_license_terms);
    view_state.accepted_license_terms = false;
    view_state.read_config_file()?;
    assert!(view_state.accepted_license_terms);

    Ok(())
}
