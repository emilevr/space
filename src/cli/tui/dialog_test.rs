use super::*;
use crate::cli::tui_test_utils::TestInputEventSource;
use crate::cli::view_state_test_utils::make_test_view_state;
use crate::test_directory_utils::delete_test_directory_tree;
use crate::test_utils::TestOut;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::sync::{atomic::AtomicBool, Arc};

#[test]
#[ignore]
fn render_with_accept_license_input_updates_view_state_and_writes_config_file() -> anyhow::Result<()>
{
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    view_state.accepted_license_terms = false;
    view_state.config_file_path =
        Some(std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4())));
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Char(DELETE_KEY), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(
            KeyCode::Char(ACCEPT_LICENSE_TERMS_KEY),
            KeyModifiers::NONE,
        )),
        Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);
    let should_exit = Arc::new(AtomicBool::new(false));

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
        should_exit,
        crossfire::mpsc::unbounded_blocking().0,
        crossfire::mpsc::unbounded_blocking().1,
    )?;

    // Assert
    assert!(view_state.accepted_license_terms);
    view_state.accepted_license_terms = false;
    view_state.read_config_file()?;
    assert!(view_state.accepted_license_terms);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}
