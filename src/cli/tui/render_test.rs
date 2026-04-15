use super::*;
use crate::cli::tui_test_utils::TestInputEventSource;
use crate::cli::view_state_test_utils::{
    assert_selected_item_name_eq, make_test_view_state, make_test_view_state_from_path,
    TEST_DIRECTORY_TREE_ITEM_COUNT,
};
use crate::test_directory_utils::delete_test_directory_tree;
use crate::test_utils::TestOut;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use rstest::rstest;
use std::sync::{atomic::AtomicBool, Arc};

#[test]
#[ignore]
fn render_outputs_crate_version_number() -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![Event::Key(KeyEvent::new(
        KeyCode::Char(QUIT_KEY_1),
        KeyModifiers::NONE,
    ))]);
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
    output.matches(Regex::new(format!("Space\\s+.*v{}", VERSION).as_str())?)?;

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[test]
#[ignore]
fn render_outputs_full_path_as_first_row_with_100_percent_size() -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![Event::Key(KeyEvent::new(
        KeyCode::Char(QUIT_KEY_1),
        KeyModifiers::NONE,
    ))]);
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
    let regex_safe_path = view_state.visible_row_items[0]
        .borrow()
        .path_segment
        .as_str()
        .replace('\\', "\\\\");
    output.matches(Regex::new(
        format!("^.*{}.*100%.*$", regex_safe_path).as_str(),
    )?)?;

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
fn render_with_delete_input_without_license_terms_accepted_shows_license_dialog(
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    view_state.accepted_license_terms = false;
    let mut output = TestOut::new();
    let input_events: Vec<Event> =
        std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .take(1) // select "1"
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char('d'),
                    KeyModifiers::NONE,
                )))
                .take(1),
            )
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char('d'),
                    KeyModifiers::NONE,
                )))
                .take(1),
            ) // Cancel
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char(QUIT_KEY_1),
                    KeyModifiers::NONE,
                )))
                .take(1),
            )
            .collect();
    let mut input_event_source = TestInputEventSource::new(input_events);
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
    output.expect("Before you are able to delete")?;
    assert_eq!(
        TEST_DIRECTORY_TREE_ITEM_COUNT,
        view_state.total_items_in_tree
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(3, "1.2")] // File
#[ignore]
#[case(4, "1.3")] // Directory
fn render_with_cancelled_delete_does_not_delete_item(
    #[case] down_count: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    view_state.accepted_license_terms = true;
    let mut output = TestOut::new();
    let input_events: Vec<Event> =
        std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .take(down_count)
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char('d'),
                    KeyModifiers::NONE,
                )))
                .take(1),
            ) // Show delete dialog
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char('d'),
                    KeyModifiers::NONE,
                )))
                .take(1),
            ) // Cancel deletion
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char(QUIT_KEY_1),
                    KeyModifiers::NONE,
                )))
                .take(1),
            )
            .collect();
    let mut input_event_source = TestInputEventSource::new(input_events);
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
    assert_selected_item_name_eq(expected_selected_item_name, &view_state, Some(&output));
    output.expect("This will free up at least")?;
    assert_eq!(
        TEST_DIRECTORY_TREE_ITEM_COUNT,
        view_state.total_items_in_tree
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(0f32, 0, "", 0)] // First item
#[ignore]
#[case(0f32, 3, "1.2", 28)] // File
#[ignore]
#[case(0f32, 4, "1.3", 25)] // Directory
#[ignore]
#[case(0.1f32, 4, "1.3", 25)] // Directory, with 10% size threshold fraction.
fn render_with_confirmed_delete_deletes_selected_item(
    #[case] size_threshold_fraction: f32,
    #[case] down_count: usize,
    #[case] expected_deleted_item_name: &str,
    #[case] expected_final_item_count: usize,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(size_threshold_fraction)?;
    view_state.accepted_license_terms = true;
    let mut output = TestOut::new();
    let input_events: Vec<Event> =
        std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .take(down_count)
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char('d'),
                    KeyModifiers::NONE,
                )))
                .take(1),
            ) // Show delete dialog
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char('y'),
                    KeyModifiers::NONE,
                )))
                .take(1),
            ) // Cancel deletion
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::Char(QUIT_KEY_1),
                    KeyModifiers::NONE,
                )))
                .take(1),
            )
            .collect();
    let mut input_event_source = TestInputEventSource::new(input_events);
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
    assert_eq!(expected_final_item_count, view_state.total_items_in_tree);
    view_state.item_tree.iter().for_each(|item| {
        let item_ref = item.borrow();
        assert!(item_ref.path_segment != expected_deleted_item_name);
        assert!(!item_ref
            .path_segment
            .starts_with(format!("{}.", expected_deleted_item_name).as_str()));
    });

    let view_state = make_test_view_state_from_path(&temp_dir_path, 0, 0, size_threshold_fraction)?;
    assert_eq!(expected_final_item_count, view_state.total_items_in_tree);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}
