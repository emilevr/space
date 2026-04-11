use super::*;
use crate::cli::tui_test_utils::TestInputEventSource;
use crate::cli::view_state_test_utils::{
    assert_selected_item_name_eq, make_test_view_state, TEST_DIRECTORY_TREE_ITEM_COUNT,
};
use crate::test_directory_utils::delete_test_directory_tree;
use crate::test_utils::TestOut;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rstest::rstest;
use std::sync::{atomic::AtomicBool, Arc};

#[rstest]
#[ignore]
#[case(0, 1, "")] // up -> first item selected
#[ignore]
#[case(1, 2, "")] // down + 2x up -> first item selected
#[ignore]
#[case(23, 23, "")] // down x23 to 1.9 + 23x up -> first item selected
#[ignore]
#[case(23, 22, "1")] // down x23 to 1.9 + 22x up -> 1 selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT - 1, 0, "1.12.1")] // down number of items in tree -> 1.12.1 selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT, 0, "1.12.1")] // down more than number of items in tree -> 1.12.1 selected
fn render_with_navigation_input_selects_correct_item(
    #[case] down_count: usize,
    #[case] up_count: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let input_events: Vec<Event> =
        std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .take(down_count)
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)))
                    .take(up_count),
            )
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

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(0, 0, 0, 1, "")] // page up -> first item selected
#[ignore]
#[case(5, 0, 0, 1, "")] // down to last item on 1st page + page up -> first item selected
#[ignore]
#[case(6, 0, 0, 1, "")] // down to 1st item on 2nd page + page up -> first item selected
#[ignore]
#[case(11, 0, TEST_DIRECTORY_TREE_ITEM_COUNT, 0, "1.12.1")] // down to 1.5.2 + page down xTEST_DIRECTORY_TREE_ITEM_COUNT -> 1.12.1 selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT - 2, 0, 1, 0, "1.12.1")] // down to 1.9 + page down -> 1.12.1 selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT - 1, 0, 1, 0, "1.12.1")] // down to 1.12.1 + page down -> 1.12.1 selected
fn render_with_page_up_or_down_navigation_input_selects_correct_item(
    #[case] down_count: usize,
    #[case] up_count: usize,
    #[case] page_down_count: usize,
    #[case] page_up_count: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let input_events: Vec<Event> =
        std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .take(down_count)
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)))
                    .take(up_count),
            )
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::PageDown,
                    KeyModifiers::NONE,
                )))
                .take(page_down_count),
            )
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(
                    KeyCode::PageUp,
                    KeyModifiers::NONE,
                )))
                .take(page_up_count),
            )
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

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(0, "")] // first given no selection -> first item selected
#[ignore]
#[case(10, "")] // down to 1.5 + first -> first item selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT, "")] // down to last item + first -> first item selected
fn render_with_first_navigation_input_selects_first_item(
    #[case] down_count: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let input_events: Vec<Event> =
        std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .take(down_count)
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)))
                    .take(1),
            )
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

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(0, "1.12.1")] // first given no selection -> last item selected
#[ignore]
#[case(10, "1.12.1")] // down to 1.5 + last -> last item selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT - 1, "1.12.1")] // down to last item + last -> last item selected
fn render_with_last_navigation_input_selects_last_item(
    #[case] down_count: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let input_events: Vec<Event> =
        std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .take(down_count)
            .chain(
                std::iter::repeat(Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)))
                    .take(1),
            )
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

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}
