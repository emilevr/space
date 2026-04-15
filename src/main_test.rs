use std::{
    env::{self, VarError},
    sync::{atomic::AtomicBool, Arc},
};

use crate::{
    cli::environment::MockEnvServiceTrait,
    logging::SPACE_LOG_LEVEL_ENV_VAR_NAME,
    parse_args, prepare_command, run,
    test_directory_utils::{create_test_directory_tree, delete_test_directory_tree},
    test_utils::{env_service_mock_without_env_vars, TestOut},
};

const BINARY_PATH: &str = "./space";

#[test]
fn parse_args_default_size_threshold_percentage_is_none() -> anyhow::Result<()> {
    let args = vec![BINARY_PATH.to_string()];
    let cli_args = parse_args(&args)?;
    assert_eq!(
        None, cli_args.size_threshold_percentage,
        "Default size threshold should be None (resolved later based on interactive mode)"
    );
    Ok(())
}

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
    assert!(cli_args.non_interactive);

    Ok(())
}

#[test]
fn prepare_command_when_no_target_paths_specified_uses_current_dir() -> anyhow::Result<()> {
    // Arrange
    let args = vec![BINARY_PATH.to_string()];
    let args = parse_args(&args)?;
    let mut env_service_mock = MockEnvServiceTrait::new();
    env_service_mock
        .expect_current_dir()
        .returning(env::current_dir);
    let should_exit = Arc::new(AtomicBool::new(false));

    // Act
    let command = prepare_command(args, Box::new(env_service_mock), should_exit)?;

    // Assert
    assert_eq!(
        Some(vec!(env::current_dir().unwrap())),
        *command.target_paths()
    );

    Ok(())
}

#[test]
fn run_given_non_existent_path_fails() {
    // Arrange
    let temp_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let args = vec![BINARY_PATH.to_string(), temp_dir.display().to_string()];
    let mut env_service_mock = MockEnvServiceTrait::new();
    env_service_mock
        .expect_var()
        .with(mockall::predicate::eq(SPACE_LOG_LEVEL_ENV_VAR_NAME))
        .returning(|_| Err(VarError::NotPresent));
    let should_exit = Arc::new(AtomicBool::new(false));

    // Act
    let result = run(
        &args,
        &mut TestOut::new(),
        Some(log_dir),
        Box::new(env_service_mock),
        should_exit,
    );

    // Assert
    if let Err(err) = result {
        assert!(err.to_string().contains("does not exist"));
    } else {
        unreachable!("Expected the result to be an error.");
    }
}

// ─── --filter-regex arg tests ────────────────────────────────────────────────

#[test]
fn parse_args_filter_regex_is_none_by_default() -> anyhow::Result<()> {
    let args = vec![BINARY_PATH.to_string()];
    let cli_args = parse_args(&args)?;
    assert_eq!(None, cli_args.filter_regex);
    Ok(())
}

#[test]
fn parse_args_filter_regex_short_flag() -> anyhow::Result<()> {
    let args = vec![
        BINARY_PATH.to_string(),
        "-r".to_string(),
        "test".to_string(),
    ];
    let cli_args = parse_args(&args)?;
    assert_eq!(Some("test".to_string()), cli_args.filter_regex);
    Ok(())
}

#[test]
fn parse_args_filter_regex_long_flag() -> anyhow::Result<()> {
    let args = vec![
        BINARY_PATH.to_string(),
        "--filter-regex".to_string(),
        "src.*rs".to_string(),
    ];
    let cli_args = parse_args(&args)?;
    assert_eq!(Some("src.*rs".to_string()), cli_args.filter_regex);
    Ok(())
}

#[test]
fn prepare_command_with_invalid_regex_returns_error() {
    let args = vec![
        BINARY_PATH.to_string(),
        "--filter-regex".to_string(),
        "[invalid".to_string(),
    ];
    let cli_args = parse_args(&args).unwrap();
    let mut env_service_mock = MockEnvServiceTrait::new();
    env_service_mock
        .expect_current_dir()
        .returning(env::current_dir);
    let should_exit = Arc::new(AtomicBool::new(false));
    let result = prepare_command(cli_args, Box::new(env_service_mock), should_exit);
    assert!(result.is_err(), "Expected an error for invalid regex");
    let msg = format!("{}", result.err().unwrap());
    assert!(
        msg.contains("filter-regex") || msg.contains("[invalid"),
        "Error message should mention the bad pattern, got: {msg}"
    );
}

#[test]
fn prepare_command_with_valid_regex_succeeds() -> anyhow::Result<()> {
    let args = vec![
        BINARY_PATH.to_string(),
        "--filter-regex".to_string(),
        "src.*\\.rs".to_string(),
    ];
    let cli_args = parse_args(&args)?;
    let mut env_service_mock = MockEnvServiceTrait::new();
    env_service_mock
        .expect_current_dir()
        .returning(env::current_dir);
    let should_exit = Arc::new(AtomicBool::new(false));
    let result = prepare_command(cli_args, Box::new(env_service_mock), should_exit);
    assert!(
        result.is_ok(),
        "A valid regex pattern should not cause an error"
    );
    Ok(())
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
    let env_service_mock = env_service_mock_without_env_vars();
    let should_exit = Arc::new(AtomicBool::new(false));

    // Act
    let result = run(
        &args,
        &mut test_out,
        Some(log_dir),
        Box::new(env_service_mock),
        should_exit,
    );

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
