use crate::cli::view_state::ViewState;
use crate::cli::view_state_test_utils::{
    assert_selected_item_name_eq, make_test_view_state_with_height,
};
use crate::test_directory_utils::delete_test_directory_tree;
use rstest::rstest;

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
#[case(3, 14, 15, 14)]
#[case(3, 14, 12, 12)]
#[case(3, 14, 17, 15)]
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
#[case(4, 0, 0f32, 0, 1, "1")]
#[case(4, 0, 0f32, 3, 3, "1.3")]
#[case(10, 0, 0.09f32, 7, 7, "1.5")]
#[case(4, 25, 0f32, 3, 3, "1.12.1")]
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
        assert_selected_item_name_eq(expected_selected_item_name, &view_state, None);
    }

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case(5, 0, 0, 0, "")]
#[case(5, 5, 0, 0, "1.3")]
#[case(5, 20, 4, 3, "1.9")]
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
    assert_selected_item_name_eq(expected_selected_item_name, &view_state, None);

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
#[case(7, 0, 0, "")]
#[case(7, 6, 5, "1.5.2")]
#[case(7, 18, 6, "1.10")]
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
    assert_selected_item_name_eq(expected_preselected_item_name, &view_state, None);

    // Act
    view_state.first();

    // Assert
    assert_eq!(0, view_state.visible_offset);
    assert_eq!(0, view_state.table_selected_index);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[case(7, 0, 0f32, 0, "")]
#[case(7, 6, 0f32, 5, "1.5.2")]
#[case(7, 18, 0f32, 6, "1.10")]
#[case(4, 0, 1.1f32, 0, "")]
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
        assert_selected_item_name_eq(expected_preselected_item_name, &view_state, None);
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
