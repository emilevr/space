use super::ViewState;
use crate::cli::view_state_test_utils::{
    assert_selected_item_name_eq, make_test_view_state, select_item_by_name,
};
use crate::test_directory_utils::delete_test_directory_tree;
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
    assert_selected_item_name_eq(item_name, &view_state, None);

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
    assert_selected_item_name_eq(item_name, &view_state, None);
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
    assert_selected_item_name_eq(item_name, &view_state, None);
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
    let mut view_state = ViewState::default();

    // Act
    view_state.expand_selected_item();

    // Assert
    assert!(view_state.get_selected_item().is_none());
}

#[rstest]
#[case("1.1", "1")]
#[case("1.5.2", "1.5")]
#[case("1.11", "1")]
fn collapse_selected_item_given_leaf_item_navigates_to_parent(
    #[case] selected_item_name: &str,
    #[case] parent_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    select_item_by_name(selected_item_name, &mut view_state)?;

    // Act
    view_state.collapse_selected_item();

    // Assert - navigates to parent without collapsing it.
    assert_selected_item_name_eq(parent_item_name, &view_state, None);

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
    assert_selected_item_name_eq(selected_item_name, &view_state, None);
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
