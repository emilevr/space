use super::*;
use crate::cli::tui_test_utils::TestInputEventSource;
use crate::cli::view_state::ViewState;
use crate::test_utils::TestOut;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[test]
#[ignore]
fn render_loop_exits_on_ctrl_c() {
    let mut view_state = ViewState::default();
    let mut output = TestOut::new();
    let ctrl_c = Event::Key(KeyEvent {
        code: KeyCode::Char('c'),
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::CONTROL,
        state: crossterm::event::KeyEventState::NONE,
    });
    let mut input_event_source = TestInputEventSource::new(vec![ctrl_c]);
    let should_exit = Arc::new(AtomicBool::new(false));
    let (scan_sender, scan_receiver) = crossfire::mpsc::unbounded_blocking();

    let result = render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
        should_exit,
        scan_sender,
        scan_receiver,
    );

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Cancelled"),
        "Expected 'Cancelled' in error, got: {err_msg}"
    );
}

#[test]
#[ignore]
fn render_loop_exits_when_should_exit_flag_is_set() {
    let mut view_state = ViewState::default();
    let mut output = TestOut::new();
    // Provide no key events; the should_exit flag will cause the loop to bail.
    let mut input_event_source = TestInputEventSource::new(vec![]);
    let should_exit = Arc::new(AtomicBool::new(true));
    let (scan_sender, scan_receiver) = crossfire::mpsc::unbounded_blocking();

    let result = render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
        should_exit,
        scan_sender,
        scan_receiver,
    );

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Cancelled"),
        "Expected 'Cancelled' in error, got: {err_msg}"
    );
}
