use super::configure_logger;
use crate::cli::environment::MockEnvServiceTrait;
use log::LevelFilter;
use mockall::predicate::eq;
use std::env::VarError;

#[test]
fn configure_logging_defaults_to_level_warn() {
    // Arrange
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let mut env_service_mock = MockEnvServiceTrait::new();
    env_service_mock
        .expect_var()
        .with(eq(super::SPACE_LOG_LEVEL_ENV_VAR_NAME))
        .returning(|_| Err(VarError::NotPresent));

    // Act
    let level = configure_logger(Some(log_dir), &env_service_mock);

    // Assert
    assert_eq!(LevelFilter::Warn, level);
}

#[test]
fn configure_logging_with_env_level_set_uses_env_level() {
    // Arrange
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let mut env_service_mock = MockEnvServiceTrait::new();
    env_service_mock
        .expect_var()
        .with(eq(super::SPACE_LOG_LEVEL_ENV_VAR_NAME))
        .returning(|_| Ok("error".into()));

    // Act
    let level = configure_logger(Some(log_dir), &env_service_mock);

    // Assert
    assert_eq!(LevelFilter::Error, level);
}

#[test]
fn configure_logging_with_env_level_set_to_invalid_value_defaults_to_warn() {
    // Arrange
    let log_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    let mut env_service_mock = MockEnvServiceTrait::new();
    env_service_mock
        .expect_var()
        .with(eq(super::SPACE_LOG_LEVEL_ENV_VAR_NAME))
        .returning(|_| Ok("some_invalid_level".into()));

    // Act
    let level = configure_logger(Some(log_dir), &env_service_mock);

    // Assert
    assert_eq!(LevelFilter::Warn, level);
}
