use super::{render_row, render_rows};
use crate::{
    cli::{
        cli_command::CliCommand,
        environment::MockEnvServiceTrait,
        row_item::{RowItem, RowItemType},
        skin::Skin,
        view_command::ViewCommand,
        view_state_test_utils::{make_test_view_state, TEST_DIRECTORY_TREE_TOTAL_SIZE},
    },
    test_directory_utils::{create_test_directory_tree, delete_test_directory_tree},
    test_utils::{env_service_mock_without_env_vars, TestOut},
};
use mockall::predicate::eq;
use ratatui::prelude::{Constraint, CrosstermBackend};
use rstest::rstest;
use space_rs::{
    size::{Size, SizeDisplayFormat},
    DirectoryItem, DirectoryItemType,
};
use std::{cell::RefCell, env::VarError, rc::Rc};
use uuid::Uuid;

#[test]
fn single_target_path_that_does_not_exist_should_fail() -> anyhow::Result<()> {
    // Arrange
    let env_service_mock = MockEnvServiceTrait::new();
    let mut view_command = ViewCommand {
        target_paths: Some(vec![std::env::temp_dir().join(Uuid::new_v4().to_string())]),
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        env_service: Box::new(env_service_mock),
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
    let env_service_mock = MockEnvServiceTrait::new();
    let view_command = ViewCommand {
        target_paths: None,
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 1000000,
        env_service: Box::new(env_service_mock),
    };
    let item = DirectoryItem {
        path_segment: "/some/path".to_string(),
        size_in_bytes: Size::default(),
        children: vec![],
        descendant_count: 0,
        item_type: DirectoryItemType::Unknown,
    };
    let mut rows = vec![];

    // Act
    view_command.add_row_item(item, 0.1f32, &mut rows);

    // Assert
    assert!(rows.is_empty());
}

#[test]
fn analyze_space_given_no_target_paths_traces_current_dir() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let mut env_service_mock = MockEnvServiceTrait::new();
    let temp_dir_copy = temp_dir.clone();
    env_service_mock
        .expect_current_dir()
        .returning(move || Ok(temp_dir_copy.clone()));

    let mut view_command = ViewCommand {
        target_paths: None,
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        env_service: Box::new(env_service_mock),
    };

    // Act
    let items = view_command.analyze_space();

    // Assert
    assert_eq!(1, items.len());
    assert_eq!(
        temp_dir.display().to_string().as_str(),
        items[0].path_segment
    );
    assert_eq!(
        TEST_DIRECTORY_TREE_TOTAL_SIZE,
        items[0].size_in_bytes.get_value()
    );
    assert_eq!(1, items[0].children.len());
    assert_eq!("1", items[0].children[0].path_segment);

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
        incl_fraction: 0.1f32,
        path_segment: "/some/path".to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        row_index: 1,
    }));

    // Act
    let rendered_count = render_row(
        &item,
        1f32,
        &constraints,
        80,
        SizeDisplayFormat::Metric,
        &mut backend,
        &Skin::default(),
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
    let rendered_count = render_rows(
        view_state,
        size_threshold_fraction,
        &mut output,
        &Skin::default(),
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
// Ignore this test by default as it needs to be run in a real terminal, similar to the TUI tests, which does
// not work on build agents for some operating systems. It will be included with those on the build agent
// using an appropriate terminal multiplexer.
#[ignore]
fn run_given_no_size_display_format_defaults_to_metric() -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir = create_test_directory_tree()?;
    let env_service_mock = env_service_mock_without_env_vars();
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        env_service: Box::new(env_service_mock),
    };

    // Act
    view_command.run(&mut output)?;

    // Assert
    output.expect("180 KB")?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

// Ignore this test by default as it needs to be run in a real terminal, similar to the TUI tests, which does
// not work on build agents for some operating systems. It will be included with those on the build agent
// using an appropriate terminal multiplexer.
#[rstest]
#[ignore]
#[case(SizeDisplayFormat::Metric, "180 KB")]
#[ignore]
#[case(SizeDisplayFormat::Binary, "175 KiB")]
fn run_with_size_display_format_uses_that_format(
    #[case] size_display_format: SizeDisplayFormat,
    #[case] expected_output: &str,
) -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir = create_test_directory_tree()?;
    let env_service_mock = env_service_mock_without_env_vars();
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: Some(size_display_format),
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        env_service: Box::new(env_service_mock),
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
    let env_service_mock = env_service_mock_without_env_vars();
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: None,
        size_threshold_percentage: 100,
        total_size_in_bytes: 0,
        env_service: Box::new(env_service_mock),
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
    let env_service_mock = env_service_mock_without_env_vars();
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir1.clone(), temp_dir2.clone()]),
        size_display_format: None,
        size_threshold_percentage: 100,
        total_size_in_bytes: 0,
        env_service: Box::new(env_service_mock),
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

#[rstest]
#[case("truecolor", "", Some(16_777_216))]
#[case("24bit", "", Some(16_777_216))]
#[case("24-bit", "", Some(16_777_216))]
#[case("kitty", "", Some(256))]
#[case("kitty-256color", "", Some(256))]
#[case("konsole", "", Some(256))]
#[case("rxvt-unicode-256color", "", Some(256))]
#[case("screen-256color", "", Some(256))]
#[case("tmux-256color", "", Some(256))]
#[case("xterm-256color", "", Some(256))]
#[case("xterm256", "", Some(256))]
#[case("ansi", "", Some(16))]
#[case("screen", "", Some(16))]
#[case("tmux", "", Some(16))]
#[case("xterm", "", Some(16))]
#[case("rxvt-unicode", "", Some(8))]
#[case("dumb", "", None)]
#[case("monochrome", "", None)]
#[case("", "truecolor", Some(16_777_216))]
#[case("", "24bit", Some(16_777_216))]
#[case("", "24-bit", Some(16_777_216))]
#[case("", "kitty", Some(256))]
#[case("", "kitty-256color", Some(256))]
#[case("", "konsole", Some(256))]
#[case("", "rxvt-unicode-256color", Some(256))]
#[case("", "screen-256color", Some(256))]
#[case("", "tmux-256color", Some(256))]
#[case("", "xterm-256color", Some(256))]
#[case("", "xterm256", Some(256))]
#[case("", "ansi", Some(16))]
#[case("", "screen", Some(16))]
#[case("", "tmux", Some(16))]
#[case("", "xterm", Some(16))]
#[case("", "rxvt-unicode", Some(8))]
#[case("", "dumb", None)]
#[case("", "monochrome", None)]
fn get_color_count_return_expected_color_count(
    #[case] colorterm_value: &str,
    #[case] term_value: &str,
    #[case] expected_color_count: Option<u32>,
) {
    // Arrange
    let mut env_service_mock = MockEnvServiceTrait::new();
    let colorterm_value = colorterm_value.to_string();
    let term_value = term_value.to_string();
    if !colorterm_value.is_empty() {
        env_service_mock
            .expect_var()
            .with(eq(crate::cli::view_command::COLORTERM_ENV_VAR))
            .returning(move |_| Ok(colorterm_value.clone()));
    } else {
        env_service_mock
            .expect_var()
            .with(eq(crate::cli::view_command::COLORTERM_ENV_VAR))
            .returning(move |_| Err(VarError::NotPresent));
    }

    if !term_value.is_empty() {
        env_service_mock
            .expect_var()
            .with(eq(crate::cli::view_command::TERM_ENV_VAR))
            .returning(move |_| Ok(term_value.clone()));
    } else {
        env_service_mock
            .expect_var()
            .with(eq(crate::cli::view_command::TERM_ENV_VAR))
            .returning(move |_| Err(VarError::NotPresent));
    }

    let view_command = ViewCommand {
        target_paths: None,
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        env_service: Box::new(env_service_mock),
    };

    // Act
    let color_count = view_command.get_color_count();

    // Assert
    assert_eq!(expected_color_count, color_count);
}
