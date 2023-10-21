use std::env;

use crate::{
    parse_args, prepare_command, run,
    test_directory_utils::{create_test_directory_tree, delete_test_directory_tree},
    test_utils::TestOut,
};

const BINARY_PATH: &str = "./space";

#[test]
fn parse_args_without_args_returns_args_with_no_target_paths() -> anyhow::Result<()> {
    // Arrange
    let args = vec![BINARY_PATH.to_string()];

    // Act
    let cli_args = parse_args(&args)?;

    // Assert
    assert_eq!(None, cli_args.target_paths);

    Ok(())
}

#[test]
fn parse_args_given_non_interactive_returns_correct_args() -> anyhow::Result<()> {
    // Arrange
    let args = vec![
        BINARY_PATH.to_string(),
        "view".to_string(),
        "--non-interactive".to_string(),
    ];

    // Act
    let cli_args = parse_args(&args)?;

    // Assert
    assert_eq!(true, cli_args.non_interactive);

    Ok(())
}

#[test]
fn prepare_command_when_no_target_paths_specified_uses_current_dir() -> anyhow::Result<()> {
    // Arrange
    let args = vec![BINARY_PATH.to_string()];
    let args = parse_args(&args)?;

    // Act
    let command = prepare_command(args)?;

    // Assert
    assert_eq!(
        Some(vec!(env::current_dir().unwrap())),
        command.target_paths
    );

    Ok(())
}

#[test]
fn run_given_non_existent_path_fails() {
    // Arrange
    let temp_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let args = vec![BINARY_PATH.to_string(), temp_dir.display().to_string()];

    // Act
    let result = run(&args, &mut TestOut::new(), Some(log_dir));

    // Assert
    if let Err(err) = result {
        assert!(err.to_string().contains("does not exist"));
    } else {
        assert!(false, "Expected the result to be an error.");
    }
}

// Ignore this test by default as it needs to be run in a real terminal, similar to the TUI tests, which does
// not work on build agents for some operating systems. It will be included with those on the build agent
// using an appropriate terminal multiplexer.
#[test]
#[ignore]
fn run_given_valid_path_succeeds() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let file_path = temp_dir.join("1").join("1.2");
    let args = vec![BINARY_PATH.to_string(), file_path.display().to_string()];
    let mut test_out = TestOut::new();

    // Act
    let result = run(&args, &mut test_out, Some(log_dir));

    // Assert
    result.expect("run() is expected to succeed!");
    test_out.expect(
        format!(
            "{}1{}1.2",
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        )
        .as_str(),
    )?;

    delete_test_directory_tree(&temp_dir);

    Ok(())
}
