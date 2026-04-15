use super::scan_drain::drain_scan_channel;
use crate::cli::scan_worker::ScanMessage;
use crate::cli::scan_worker::ScanReceiver;
use crate::cli::tui_test_utils::TestInputEventSource;
use crate::cli::view_state::ViewState;
use crossfire::mpsc as cf_mpsc;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use space_rs::{DirectoryItem, DirectoryItemType, Size};
use std::time::{Duration, Instant};

fn test_deadline() -> Instant {
    Instant::now() + Duration::from_secs(1)
}

fn test_drain(receiver: &ScanReceiver, view_state: &mut ViewState) -> bool {
    let mut count = 1usize;
    drain_scan_channel(receiver, view_state, test_deadline(), &mut count)
}

// ─── Unit tests for ViewState::is_scanning ────────────────────────────────────

#[test]
fn view_state_default_is_scanning_is_false() {
    let view_state = ViewState::default();
    assert!(!view_state.is_scanning);
}

#[test]
fn view_state_is_scanning_can_be_set_to_true() {
    let view_state = ViewState {
        is_scanning: true,
        ..Default::default()
    };
    assert!(view_state.is_scanning);
}

// ─── Unit tests for poll_event ────────────────────────────────────────────────

#[test]
fn test_input_event_source_returns_events_in_order() {
    use crate::cli::input_event_source::InputEventSource;

    let events = vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
    ];
    let mut source = TestInputEventSource::new(events);

    let first = source
        .poll_event(std::time::Duration::from_millis(0))
        .unwrap();
    let second = source
        .poll_event(std::time::Duration::from_millis(0))
        .unwrap();

    assert_eq!(
        first,
        Some(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
    );
    assert_eq!(
        second,
        Some(Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)))
    );
}

#[test]
fn test_input_event_source_returns_resize_when_queue_is_empty() {
    use crate::cli::input_event_source::InputEventSource;

    let mut source = TestInputEventSource::new(vec![]);

    let event = source
        .poll_event(std::time::Duration::from_millis(0))
        .unwrap();

    // When empty, returns a Resize event to keep the render loop alive without blocking.
    assert!(matches!(event, Some(Event::Resize(_, _))));
}

// ─── Unit tests for drain_scan_channel ────────────────────────────────────────

fn make_file_dir_item(path_segment: &str, size: u64) -> DirectoryItem {
    DirectoryItem {
        path_segment: path_segment.to_string(),
        item_type: DirectoryItemType::File,
        size_in_bytes: Size::new(size),
        descendant_count: 0,
        children: vec![],
    }
}

#[test]
fn drain_scan_channel_processes_item_and_adds_to_view_state() {
    let mut view_state = ViewState {
        is_scanning: true,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();
    sender
        .send(ScanMessage::Item(make_file_dir_item("a", 1000)))
        .unwrap();

    let complete = test_drain(&receiver, &mut view_state);

    assert!(
        !complete,
        "Should not signal completion - no Complete message"
    );
    assert_eq!(1, view_state.item_tree.len());
    assert_eq!(1000, view_state.total_size_in_bytes);
    // Still scanning - Complete not received yet.
    assert!(view_state.is_scanning);
}

#[test]
fn drain_scan_channel_processes_child_item_and_adds_to_root() {
    let mut view_state = ViewState {
        is_scanning: true,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();

    // Send a root item, then a child item.
    sender
        .send(ScanMessage::Item(DirectoryItem {
            path_segment: "/root".to_string(),
            item_type: DirectoryItemType::Directory,
            size_in_bytes: Size::new(0),
            descendant_count: 0,
            children: vec![],
        }))
        .unwrap();
    sender
        .send(ScanMessage::ChildItem(make_file_dir_item("child", 500)))
        .unwrap();

    test_drain(&receiver, &mut view_state);

    assert_eq!(1, view_state.item_tree.len());
    let root = view_state.item_tree[0].borrow();
    assert_eq!(1, root.children.len());
    assert_eq!(500, root.size.get_value());
    assert_eq!(500, view_state.total_size_in_bytes);
}

#[test]
fn drain_scan_channel_on_complete_sets_is_scanning_false_and_returns_true() {
    let mut view_state = ViewState {
        is_scanning: true,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();
    sender.send(ScanMessage::Complete).unwrap();

    let complete = test_drain(&receiver, &mut view_state);

    assert!(complete, "Should signal completion on Complete message");
    assert!(!view_state.is_scanning);
}

#[test]
fn drain_scan_channel_on_complete_recalculates_fractions() {
    let mut view_state = ViewState {
        is_scanning: true,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking();
    // Send two items with different sizes, then complete.
    sender
        .send(ScanMessage::Item(make_file_dir_item("a", 1000)))
        .unwrap();
    sender
        .send(ScanMessage::Item(make_file_dir_item("b", 3000)))
        .unwrap();
    sender.send(ScanMessage::Complete).unwrap();

    test_drain(&receiver, &mut view_state);

    assert!(!view_state.is_scanning);
    // After recalculation with total 4000: a = 0.25, b = 0.75.
    let fraction_a = view_state.item_tree[0].borrow().incl_fraction;
    let fraction_b = view_state.item_tree[1].borrow().incl_fraction;
    assert!(
        (fraction_a - 0.25f32).abs() < 0.001f32,
        "Expected fraction_a ≈ 0.25, got {fraction_a}"
    );
    assert!(
        (fraction_b - 0.75f32).abs() < 0.001f32,
        "Expected fraction_b ≈ 0.75, got {fraction_b}"
    );
}

#[test]
fn drain_scan_channel_on_disconnected_sender_sets_is_scanning_false_and_returns_true() {
    let mut view_state = ViewState {
        is_scanning: true,
        ..Default::default()
    };
    let (sender, receiver) = cf_mpsc::unbounded_blocking::<ScanMessage>();
    drop(sender); // Disconnect without sending Complete.

    let complete = test_drain(&receiver, &mut view_state);

    assert!(complete, "Should signal completion on channel disconnect");
    assert!(!view_state.is_scanning);
}

#[test]
fn drain_scan_channel_with_empty_channel_does_not_change_is_scanning() {
    let mut view_state = ViewState {
        is_scanning: true,
        ..Default::default()
    };
    let (_sender, receiver) = cf_mpsc::unbounded_blocking::<ScanMessage>();
    // Channel is empty - no messages.

    let complete = test_drain(&receiver, &mut view_state);

    assert!(!complete, "Should not signal completion on empty channel");
    // is_scanning stays true - we're still waiting for scan results.
    assert!(view_state.is_scanning);
}
