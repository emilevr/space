use crate::{
    cli::{
        cli_command::CliCommand,
        row_item::{RowItem, RowItemType},
        view_command::ViewCommand,
        view_state_test_utils::make_test_view_state,
    },
    test_directory_utils::{create_test_directory_tree, delete_test_directory_tree},
    test_utils::TestOut,
};
use ratatui::prelude::{Constraint, CrosstermBackend};
use rstest::rstest;
use space_rs::{
    size::{Size, SizeDisplayFormat},
    DirectoryItem, DirectoryItemType,
};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};
use uuid::Uuid;

use super::{render_row, render_rows};

#[test]
fn single_target_path_that_does_not_exist_should_fail() -> anyhow::Result<()> {
    // Arrange
    let mut view_command = ViewCommand {
        target_paths: Some(vec![std::env::temp_dir().join(Uuid::new_v4().to_string())]),
        size_display_format: None,
        size_threshold_percentage: 1,
        non_interactive: true,
        total_size_in_bytes: 0,
    };

    // Act
    let result = view_command.prepare();

    // Assert
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("does not exist"));

    Ok(())
}

#[test]
fn add_row_item_given_item_of_size_below_threshold_does_not_add_item() {
    // Arrange
    let view_command = ViewCommand {
        target_paths: None,
        size_display_format: None,
        size_threshold_percentage: 1,
        non_interactive: false,
        total_size_in_bytes: 1000000,
    };
    let item = DirectoryItem {
        path: Arc::new(PathBuf::from("/some/path")),
        size_in_bytes: Size::default(),
        children: vec![],
        item_type: DirectoryItemType::Unknown,
    };
    let mut rows = vec![];

    // Act
    view_command.add_row_item(item, 0.1f32, &mut rows);

    // Assert
    assert!(rows.is_empty());
}

#[test]
fn trace_space_given_no_target_paths_traces_current_dir() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_dir)?;

    let mut view_command = ViewCommand {
        target_paths: None,
        size_display_format: None,
        size_threshold_percentage: 1,
        non_interactive: true,
        total_size_in_bytes: 0,
    };

    // Act
    let items = view_command.trace_space();

    // Assert
    assert_eq!(1, items.len());
    assert!(items[0]
        .path
        .display()
        .to_string()
        .contains(temp_dir.display().to_string().as_str()));
    assert_eq!(170000, items[0].size_in_bytes.get_value());
    assert_eq!(1, items[0].children.len());
    assert!(items[0].children[0]
        .path
        .display()
        .to_string()
        .ends_with("1"));

    // Restore the current dir.
    std::env::set_current_dir(current_dir)?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

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
        name: "some name".to_string(),
        incl_fraction: 0.1f32,
        path: Arc::new(PathBuf::from("/some/path")),
        children: vec![],
        parent: None,
        row_index: 1,
    }));

    // Act
    let rendered_count = render_row(
        &item,
        1f32,
        &constraints,
        80,
        &SizeDisplayFormat::Metric,
        &mut backend,
    )?;

    // Assert
    assert_eq!(0, rendered_count);

    Ok(())
}

#[rstest]
#[case(0f32, 25)]
#[case(0.01f32, 18)]
#[case(0.02f32, 15)]
#[case(0.03f32, 13)]
#[case(0.04f32, 12)]
#[case(0.08f32, 11)]
#[case(0.09f32, 9)]
#[case(0.1f32, 8)]
#[case(0.11f32, 7)]
#[case(0.14f32, 4)]
#[case(0.15f32, 2)]
#[case(0.2f32, 2)]
fn render_rows_given_size_threshold_should_render_correct_rows(
    #[case] size_threshold_fraction: f32,
    #[case] expected_rendered_count: usize,
) -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let (view_state, temp_dir_path) = make_test_view_state(0f32)?;

    // Act
    let rendered_count = render_rows(view_state, size_threshold_fraction, &mut output)?;

    // Assert
    if expected_rendered_count != rendered_count {
        println!("{}", output);
    }
    assert_eq!(expected_rendered_count, rendered_count);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[test]
// Ignore this test by default as it needs to be run in a real terminal, similar to the TUI tests, which does
// not work on build agents for some operating systems. It will be included with those on the build agent
// using an appropriate terminal multiplexer.
#[ignore]
fn run_given_no_size_display_format_defaults_to_metric() -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir = create_test_directory_tree()?;
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: None,
        size_threshold_percentage: 1,
        non_interactive: true,
        total_size_in_bytes: 0,
    };

    // Act
    view_command.run(&mut output)?;

    // Assert
    output.expect("170 KB")?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

// Ignore this test by default as it needs to be run in a real terminal, similar to the TUI tests, which does
// not work on build agents for some operating systems. It will be included with those on the build agent
// using an appropriate terminal multiplexer.
#[rstest]
#[ignore]
#[case(SizeDisplayFormat::Metric, "170 KB")]
#[ignore]
#[case(SizeDisplayFormat::Binary, "166 KiB")]
fn run_with_size_display_format_uses_that_format(
    #[case] size_display_format: SizeDisplayFormat,
    #[case] expected_output: &str,
) -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir = create_test_directory_tree()?;
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: Some(size_display_format),
        size_threshold_percentage: 1,
        non_interactive: true,
        total_size_in_bytes: 0,
    };

    // Act
    view_command.run(&mut output)?;

    // Assert
    output.expect(expected_output)?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

#[test]
// Ignore this test by default as it needs to be run in a real terminal, similar to the TUI tests, which does
// not work on build agents for some operating systems. It will be included with those on the build agent
// using an appropriate terminal multiplexer.
#[ignore]
fn run_given_single_target_path_shows_expected_duration_prompt() -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir = create_test_directory_tree()?;
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: None,
        size_threshold_percentage: 100,
        non_interactive: true,
        total_size_in_bytes: 0,
    };

    // Act
    view_command.run(&mut output)?;

    // Assert
    output.expect("Analyzing path ")?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

#[test]
// Ignore this test by default as it needs to be run in a real terminal, similar to the TUI tests, which does
// not work on build agents for some operating systems. It will be included with those on the build agent
// using an appropriate terminal multiplexer.
#[ignore]
fn run_given_multiple_target_paths_shows_expected_duration_prompt() -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir1 = create_test_directory_tree()?;
    let temp_dir2 = create_test_directory_tree()?;
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir1.clone(), temp_dir2.clone()]),
        size_display_format: None,
        size_threshold_percentage: 100,
        non_interactive: true,
        total_size_in_bytes: 0,
    };

    // Act
    view_command.run(&mut output)?;

    // Assert
    output.expect("Analyzing the following paths:")?;
    output.expect(temp_dir1.to_str().expect("Unable to get temp dir 1 path"))?;
    output.expect(temp_dir2.to_str().expect("Unable to get temp dir 2 path"))?;

    delete_test_directory_tree(&temp_dir1);
    delete_test_directory_tree(&temp_dir2);

    Ok(())
}
