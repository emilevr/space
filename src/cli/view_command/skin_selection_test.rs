use crate::cli::{environment::MockEnvServiceTrait, view_command::ViewCommand};
use mockall::predicate::eq;
use rstest::rstest;
use std::{
    env::VarError,
    sync::{atomic::AtomicBool, Arc},
};

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
    let should_exit = Arc::new(AtomicBool::new(false));

    let view_command = ViewCommand {
        target_paths: None,
        size_display_format: None,
        size_threshold_percentage: 1,
        filter_regex: None,
        total_size_in_bytes: 0,
        env_service: Box::new(env_service_mock),
        should_exit,
    };

    // Act
    let color_count = view_command.get_color_count();

    // Assert
    assert_eq!(expected_color_count, color_count);
}
