use crate::{
    cli::{cli_command::CliCommand, view_command::ViewCommand},
    parse_args, resolve_command, run,
    test_directory_utils::{create_test_directory_tree, delete_test_directory_tree},
    test_utils::TestOut,
    CommandArgs,
};
use std::env;

const BINARY_PATH: &str = "./space";

#[test]
fn parse_args_when_no_sub_command_specified_defaults_to_view_command() -> anyhow::Result<()> {
    // Arrange
    let args = vec![BINARY_PATH.to_string()];

    // Act
    let cli_args = parse_args(args)?;

    // Assert
    match cli_args.command {
        CommandArgs::View {
            target_paths,
            size_threshold_percentage,
            non_interactive,
        } => {
            assert_eq!(None, target_paths);
            assert_eq!(
                1,
                size_threshold_percentage,
                "Expected size_threshold_percentage to be the default of {}.",
                crate::DEFAULT_SIZE_THRESHOLD_PERCENTAGE
            );
            assert_eq!(
                false, non_interactive,
                "Expected the non-interactive filter to be None."
            );
        }
    };

    Ok(())
}

#[test]
fn parse_args_given_view_command_without_args_returns_view_command() -> anyhow::Result<()> {
    // Arrange
    let args = vec![BINARY_PATH, "view"];

    // Act
    let cli_args = parse_args(args.iter().map(|arg| arg.to_string()).collect())?;

    // Assert
    match cli_args.command {
        CommandArgs::View { .. } => {}
    };

    Ok(())
}

#[test]
fn parse_args_given_view_command_non_interactive_returns_correct_view_command() -> anyhow::Result<()>
{
    // Arrange
    let args = vec![BINARY_PATH, "view", "--non-interactive"];

    // Act
    let cli_args = parse_args(args.iter().map(|arg| arg.to_string()).collect())?;

    // Assert
    match cli_args.command {
        CommandArgs::View {
            non_interactive, ..
        } => {
            assert_eq!(true, non_interactive);
        }
    };

    Ok(())
}

#[test]
fn resolve_command_when_no_target_paths_specified_uses_current_dir() -> anyhow::Result<()> {
    // Arrange
    let args = vec![BINARY_PATH.to_string()];

    // Act
    let mut cli_command = resolve_command(args)?;
    cli_command.prepare()?;

    // Assert
    let view_command = cli_command
        .as_any()
        .downcast_ref::<ViewCommand>()
        .expect("Not a ViewCommand!");
    assert_eq!(
        Some(vec!(env::current_dir().unwrap())),
        view_command.target_paths
    );

    Ok(())
}

#[test]
fn run_given_invalid_args_fails() {
    // Arrange

    // Act
    let result = run(vec![], &mut TestOut::new());

    // Assert
    if let Err(err) = result {
        assert!(err.to_string().contains("Arguments are invalid!"));
    } else {
        assert!(false, "Expected the result to be an error.");
    }
}

#[test]
fn run_given_non_existent_path_fails() {
    // Arrange
    let temp_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let args = vec![BINARY_PATH.to_string(), temp_dir.display().to_string()];

    // Act
    let result = run(args, &mut TestOut::new());

    // Assert
    if let Err(err) = result {
        assert!(err.to_string().contains("does not exist"));
    } else {
        assert!(false, "Expected the result to be an error.");
    }
}

#[test]
// Ignore this test by default as it needs to be run in a real terminal, similar to the TUI tests. It will be
// included with those on the build agent.
#[ignore]
fn run_given_valid_path_succeeds() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let file_path = temp_dir.join("1").join("1.2");
    let args = vec![BINARY_PATH.to_string(), file_path.display().to_string()];
    let mut test_out = TestOut::new();

    // Act
    let result = run(args, &mut test_out);

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
