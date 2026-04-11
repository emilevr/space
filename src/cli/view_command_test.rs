use crate::cli::{
    cli_command::CliCommand, environment::MockEnvServiceTrait, view_command::ViewCommand,
};
use crate::test_directory_utils::{create_test_directory_tree, delete_test_directory_tree};
use crate::test_utils::{env_service_mock_without_env_vars, TestOut};
use rstest::rstest;
use space_rs::{
    size::{Size, SizeDisplayFormat},
    DirectoryItem, DirectoryItemType,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use uuid::Uuid;

#[test]
fn single_target_path_that_does_not_exist_should_fail() -> anyhow::Result<()> {
    // Arrange
    let env_service_mock = MockEnvServiceTrait::new();
    let should_exit = Arc::new(AtomicBool::new(false));
    let mut view_command = ViewCommand {
        target_paths: Some(vec![std::env::temp_dir().join(Uuid::new_v4().to_string())]),
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        filter_regex: None,
        env_service: Box::new(env_service_mock),
        should_exit,
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
    let should_exit = Arc::new(AtomicBool::new(false));
    let view_command = ViewCommand {
        target_paths: None,
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 1000000,
        filter_regex: None,
        env_service: Box::new(env_service_mock),
        should_exit,
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
    let should_exit = Arc::new(AtomicBool::new(false));
    let mut view_command = ViewCommand {
        target_paths: None,
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        filter_regex: None,
        env_service: Box::new(env_service_mock),
        should_exit,
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
        crate::cli::view_state_test_utils::TEST_DIRECTORY_TREE_TOTAL_SIZE,
        items[0].size_in_bytes.get_value()
    );
    assert_eq!(1, items[0].children.len());
    assert_eq!("1", items[0].children[0].path_segment);

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

#[test]
#[ignore]
fn run_given_no_size_display_format_defaults_to_metric() -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir = create_test_directory_tree()?;
    let env_service_mock = env_service_mock_without_env_vars();
    let should_exit = Arc::new(AtomicBool::new(false));
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: None,
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        filter_regex: None,
        env_service: Box::new(env_service_mock),
        should_exit,
    };

    // Act
    view_command.run(&mut output)?;

    // Assert
    output.expect("180 KB")?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

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
    let should_exit = Arc::new(AtomicBool::new(false));
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: Some(size_display_format),
        size_threshold_percentage: 1,
        total_size_in_bytes: 0,
        filter_regex: None,
        env_service: Box::new(env_service_mock),
        should_exit,
    };

    // Act
    view_command.run(&mut output)?;

    // Assert
    output.expect(expected_output)?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

#[test]
#[ignore]
fn run_given_single_target_path_shows_expected_duration_prompt() -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir = create_test_directory_tree()?;
    let env_service_mock = env_service_mock_without_env_vars();
    let should_exit = Arc::new(AtomicBool::new(false));
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: None,
        size_threshold_percentage: 100,
        total_size_in_bytes: 0,
        filter_regex: None,
        env_service: Box::new(env_service_mock),
        should_exit,
    };

    // Act
    view_command.run(&mut output)?;

    // Assert
    output.expect("Analyzing path ")?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}

#[test]
#[ignore]
fn run_given_multiple_target_paths_shows_expected_duration_prompt() -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir1 = create_test_directory_tree()?;
    let temp_dir2 = create_test_directory_tree()?;
    let env_service_mock = env_service_mock_without_env_vars();
    let should_exit = Arc::new(AtomicBool::new(false));
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir1.clone(), temp_dir2.clone()]),
        size_display_format: None,
        size_threshold_percentage: 100,
        total_size_in_bytes: 0,
        filter_regex: None,
        env_service: Box::new(env_service_mock),
        should_exit,
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

#[test]
#[ignore]
fn run_with_should_exit_equal_to_true_exits() -> anyhow::Result<()> {
    // Arrange
    let mut output = TestOut::new();
    let temp_dir = create_test_directory_tree()?;
    let env_service_mock = env_service_mock_without_env_vars();
    let should_exit = Arc::new(AtomicBool::new(false));
    let mut view_command = ViewCommand {
        target_paths: Some(vec![temp_dir.clone()]),
        size_display_format: None,
        size_threshold_percentage: 100,
        total_size_in_bytes: 0,
        filter_regex: None,
        env_service: Box::new(env_service_mock),
        should_exit: should_exit.clone(),
    };

    // Act
    should_exit.store(true, Ordering::SeqCst);
    let result = view_command.run(&mut output);

    // Assert
    assert!(result.is_err());

    delete_test_directory_tree(&temp_dir);

    Ok(())
}
