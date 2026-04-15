use super::*;
use crate::cli::tui_test_utils::TestInputEventSource;
use crate::cli::view_state::ViewState;
use crate::cli::view_state_test_utils::{make_test_view_state, TEST_DIRECTORY_TREE_ITEM_COUNT};
use crate::test_directory_utils::delete_test_directory_tree;
use crate::test_utils::TestOut;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use rstest::rstest;
use std::sync::{atomic::AtomicBool, Arc};

#[test]
#[ignore]
fn render_with_help_key_input_outputs_help() -> anyhow::Result<()> {
    // Arrange
    let mut view_state = ViewState::default();
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Char(HELP_KEY), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char(HELP_KEY), KeyModifiers::NONE)),
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
    output.matches(Regex::new(
        format!("^.*\\s+.*{}\\s+.*{}.*\\s+", QUIT_KEY_1, QUIT_KEY_2_SYMBOL).as_str(),
    )?)?;

    Ok(())
}

#[rstest]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_0_PERCENT_KEY, 0f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_10_PERCENT_KEY, 0.01f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_20_PERCENT_KEY, 0.02f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_30_PERCENT_KEY, 0.03f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_40_PERCENT_KEY, 0.04f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_50_PERCENT_KEY, 0.05f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_60_PERCENT_KEY, 0.06f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_70_PERCENT_KEY, 0.07f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_80_PERCENT_KEY, 0.08f32)]
#[ignore]
#[case(VIEW_SIZE_THRESHOLD_90_PERCENT_KEY, 0.09f32)]
fn render_with_view_size_threshold_input_shows_percentage_on_top_line(
    #[case] view_size_threshold_key: char,
    #[case] size_threshold_fraction: f32,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(size_threshold_fraction)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(
            KeyCode::Char(view_size_threshold_key),
            KeyModifiers::NONE,
        )),
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
    output.matches(Regex::new(
        format!(
            "Space\\s+.*≥\\s+.*{:.0}%\\s*",
            size_threshold_fraction * 100f32
        )
        .as_str(),
    )?)?;

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[test]
#[ignore]
fn render_given_collapse_input_for_expanded_directory_item_collapses_it() -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    // Down to "1" (expanded dir), Down to its first child, Left navigates
    // to parent "1", Left again collapses "1".
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
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
    let item = view_state.visible_row_items[1].borrow();
    assert_eq!("1", item.path_segment);
    assert!(!item.expanded);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[test]
#[ignore]
fn render_given_expand_input_for_collapsed_directory_item_expands_it() -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
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
    let item = view_state.visible_row_items[1].borrow();
    assert_eq!("1", item.path_segment);
    assert!(item.expanded);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(COLLAPSE_SELECTED_CHILDREN_KEY)]
#[ignore]
#[case(COLLAPSE_SELECTED_CHILDREN_KEY_ALT)]
fn render_given_collapse_children_input_for_directory_item_collapses_all_children(
    #[case] key: char,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)),
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
    let item = view_state.visible_row_items[1].borrow();
    assert_eq!("1", item.path_segment);
    assert!(!item.expanded);
    assert_eq!(2, view_state.displayable_item_count);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(EXPAND_SELECTED_CHILDREN_KEY)]
#[ignore]
#[case(EXPAND_SELECTED_CHILDREN_KEY_ALT)]
fn render_given_expand_children_input_for_collapsed_directory_item_expands_all_children(
    #[case] key: char,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)),
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
    assert_eq!(14, view_state.displayable_item_count);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(EXPAND_SELECTED_CHILDREN_KEY)]
#[ignore]
#[case(EXPAND_SELECTED_CHILDREN_KEY_ALT)]
fn render_given_expand_children_input_for_expanded_directory_item_expands_all_children(
    #[case] key: char,
) -> anyhow::Result<()> {
    // Arrange
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)),
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
    assert_eq!(
        TEST_DIRECTORY_TREE_ITEM_COUNT,
        view_state.displayable_item_count
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}
