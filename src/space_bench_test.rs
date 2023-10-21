use super::run;
use crate::test_directory_utils::{create_test_directory_tree, delete_test_directory_tree};

const BINARY_PATH: &str = "./space-bench";

#[test]
fn run_without_target_path_fails() {
    // Arrange
    let args = vec![BINARY_PATH.to_string()];

    // Act
    let result = run(&args);

    // Assert
    assert!(result.is_err());
}

#[test]
fn run_with_target_path_succeeds() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = create_test_directory_tree()?;
    let args = vec![
        BINARY_PATH.to_string(),
        temp_dir.display().to_string(),
        "--warmup-seconds".to_string(),
        "1".to_string(),
        "--measurement-seconds".to_string(),
        "1".to_string(),
    ];

    // Act
    run(&args)?;

    // Assert
    delete_test_directory_tree(&temp_dir);

    Ok(())
}
