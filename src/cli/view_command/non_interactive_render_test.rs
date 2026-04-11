use super::{render_row, render_rows};
use crate::cli::{
    row_item::{RowItem, RowItemType},
    skin::Skin,
    view_state_test_utils::make_test_view_state,
};
use crate::test_directory_utils::delete_test_directory_tree;
use crate::test_utils::TestOut;
use ratatui::prelude::{Constraint, CrosstermBackend};
use rstest::rstest;
use space_rs::size::{Size, SizeDisplayFormat};
use std::{cell::RefCell, rc::Rc};

#[test]
fn render_row_with_size_smaller_than_threshold_does_not_output_anything() -> anyhow::Result<()> {
    // Arrange
    let constraints = vec![
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Length(5),
    ];
    let writer = TestOut::new();
    let mut backend = CrosstermBackend::new(writer);
    let item = Rc::new(RefCell::new(RowItem {
        size: Size::default(),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: 0.1f32,
        peer_fraction: 0.0,
        path_segment: "/some/path".to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 1,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }));

    // Act
    let should_exit = std::sync::atomic::AtomicBool::new(false);
    let mut rows_since_check = 0;
    let rendered_count = render_row(
        &item,
        1f32,
        &constraints,
        80,
        SizeDisplayFormat::Metric,
        &mut backend,
        &Skin::default(),
        &should_exit,
        &mut rows_since_check,
    )?;

    // Assert
    assert_eq!(0, rendered_count);

    Ok(())
}

#[rstest]
#[case(0f32, 29)]
#[case(0.01f32, 20)]
#[case(0.02f32, 17)]
#[case(0.03f32, 15)]
#[case(0.04f32, 14)]
#[case(0.08f32, 10)]
#[case(0.09f32, 8)]
#[case(0.1f32, 8)]
#[case(0.11f32, 7)]
#[case(0.14f32, 2)]
#[case(0.15f32, 2)]
#[case(0.2f32, 2)]
#[case(1f32, 2)]
fn render_rows_given_size_threshold_should_render_correct_rows(
    #[case] size_threshold_fraction: f32,
    #[case] expected_rendered_count: usize,
) -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let (view_state, temp_dir_path) = make_test_view_state(0f32)?;

    // Act
    let should_exit = std::sync::atomic::AtomicBool::new(false);
    let rendered_count = render_rows(
        view_state,
        size_threshold_fraction,
        &mut output,
        &Skin::default(),
        &should_exit,
    )?;

    // Assert
    if expected_rendered_count != rendered_count {
        println!("{}", output);
    }
    assert_eq!(expected_rendered_count, rendered_count);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[test]
fn render_row_with_regex_hidden_item_does_not_render() -> anyhow::Result<()> {
    // Arrange - item passes size threshold but is hidden by regex filter
    let constraints = vec![
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Length(5),
    ];
    let writer = TestOut::new();
    let mut backend = CrosstermBackend::new(writer);
    let item = Rc::new(RefCell::new(RowItem {
        size: Size::default(),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: 1.0f32,
        peer_fraction: 0.0,
        path_segment: "/some/path".to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: false,
    }));

    // Act
    let should_exit = std::sync::atomic::AtomicBool::new(false);
    let mut rows_since_check = 0;
    let rendered_count = render_row(
        &item,
        0f32,
        &constraints,
        80,
        SizeDisplayFormat::Metric,
        &mut backend,
        &Skin::default(),
        &should_exit,
        &mut rows_since_check,
    )?;

    // Assert
    assert_eq!(0, rendered_count);

    Ok(())
}
