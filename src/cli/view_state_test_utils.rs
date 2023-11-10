use super::{
    environment::MockEnvServiceTrait, row_item::RowItem, skin::Skin, view_state::ViewState,
};
use crate::{cli::view_command::ViewCommand, test_directory_utils::create_test_directory_tree};
use space_rs::SizeDisplayFormat;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub(crate) const TEST_DIRECTORY_TREE_ITEM_COUNT: usize = 29;
pub(crate) const TEST_DIRECTORY_TREE_TOTAL_SIZE: u64 = 180000;

pub(crate) fn make_test_view_state(
    size_threshold_fraction: f32,
) -> Result<(ViewState, PathBuf), anyhow::Error> {
    let temp_dir = create_test_directory_tree()?;
    // The visible height and offset is not important for the test calling this, so just use any valid values.
    Ok((
        make_test_view_state_from_path(&temp_dir, 7, 0, size_threshold_fraction)?,
        temp_dir,
    ))
}

pub(crate) fn make_test_view_state_with_height(
    visible_height: usize,
    visible_offset: usize,
    size_threshold_fraction: f32,
) -> Result<(ViewState, PathBuf), anyhow::Error> {
    let temp_dir = create_test_directory_tree()?;
    Ok((
        make_test_view_state_from_path(
            &temp_dir,
            visible_height,
            visible_offset,
            size_threshold_fraction,
        )?,
        temp_dir,
    ))
}

pub(crate) fn make_test_view_state_from_path(
    path: &PathBuf,
    visible_height: usize,
    visible_offset: usize,
    size_threshold_fraction: f32,
) -> Result<ViewState, anyhow::Error> {
    let size_display_format = SizeDisplayFormat::Metric;
    let env_service_mock = MockEnvServiceTrait::new();
    let mut view_command = ViewCommand::new(
        Some(vec![path.clone()]),
        Some(size_display_format.clone()),
        (size_threshold_fraction * 100f32) as u8,
        Box::new(env_service_mock),
    );
    let items = view_command.get_directory_items();
    let items = view_command.get_row_items(items, 0f32);

    let mut view_state = ViewState::new(
        items,
        size_display_format,
        size_threshold_fraction,
        &Skin::default(),
    );
    view_state.visible_height = visible_height;
    view_state.visible_offset = visible_offset;
    view_state.table_width = 80;
    view_state.update_visible_rows();

    Ok(view_state)
}

pub(crate) fn assert_selected_item_name_eq(
    expected_selected_item_name: &str,
    view_state: &ViewState,
) {
    let mut expected = expected_selected_item_name.to_string();
    if view_state.visible_offset == 0 && view_state.table_selected_index == 0 {
        // Verify that we expected the first item to be selected.
        assert_eq!(
            "", expected,
            "Did not expect the first item to be selected."
        );
        // The first item is selected, which has a randomly generated name. We use that in the check instead
        // of the specified value.
        expected = view_state.visible_row_items[0]
            .borrow()
            .path_segment
            .clone();
    }

    let selected_item = view_state.get_selected_item().unwrap();
    let selected_item = selected_item.borrow();
    assert_eq!(expected, selected_item.path_segment);
}

pub(crate) fn select_item_by_name(name: &str, view_state: &mut ViewState) -> anyhow::Result<()> {
    // If not the first item, specified with empty string, then select the item by name.
    if name != "" {
        let row_index = get_row_index_by_name(name, &view_state).unwrap();
        view_state.select_item(row_index);
        assert_selected_item_name_eq(name, &view_state);
        return Ok(());
    } else if !view_state.item_tree.is_empty() {
        view_state.select_item(0);
        return Ok(());
    }

    anyhow::bail!("Could not find and select an item with name: {}", name);
}

pub(crate) fn get_row_index_by_name(name: &str, view_state: &ViewState) -> Option<usize> {
    for item in &view_state.item_tree {
        if let Some(item_row_index) = get_item_row_index(name, item) {
            return Some(item_row_index);
        }
    }

    None
}

fn get_item_row_index(name: &str, item: &Rc<RefCell<RowItem>>) -> Option<usize> {
    let item = item.borrow();
    if item.path_segment == name {
        return Some(item.row_index);
    } else if item.has_children {
        for child in &item.children {
            if let Some(item_row_index) = get_item_row_index(name, child) {
                return Some(item_row_index);
            }
        }
    }

    None
}
