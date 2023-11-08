use super::VERSION;
use super::*;
use crate::cli::view_state::ViewState;
use crate::cli::view_state_test_utils::{
    assert_selected_item_name_eq, make_test_view_state, TEST_DIRECTORY_TREE_ITEM_COUNT,
};
use crate::{
    cli::{
        input_event_source::InputEventSource, view_state_test_utils::make_test_view_state_from_path,
    },
    test_directory_utils::delete_test_directory_tree,
    test_utils::TestOut,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use rstest::rstest;
use std::collections::VecDeque;

struct TestInputEventSource {
    events: VecDeque<Event>,
}

impl TestInputEventSource {
    fn new(events: Vec<Event>) -> Self {
        TestInputEventSource {
            events: VecDeque::from(events),
        }
    }
}

impl InputEventSource for TestInputEventSource {
    fn read_event(&mut self) -> std::io::Result<crossterm::event::Event> {
        if self.events.len() > 0 {
            Ok(self.events.pop_front().unwrap())
        } else {
            Ok(Event::Resize(80, 40))
        }
    }
}

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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    let regex_safe_path = view_state.visible_row_items[0]
        .borrow()
        .path_segment
        .as_str()
        .replace("\\", "\\\\");
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
                    // Any key except 'a' can be used to close the dialog.
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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    output.expect("Before you are able to delete")?;
    // Also confirm that nothing was deleted.
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
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    // Set accepted_license_terms to true, otherwise the license accept dialog would be shown, not the delete dialog.
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
                    // Any key except 'y' can be used to cancel, but we are also guarding against
                    // double-tapping 'd' used for deletion, which would be dangerous.
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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    assert_selected_item_name_eq(expected_selected_item_name, &view_state);
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
#[case(0f32, 3, "1.2", 24)] // File
#[ignore]
#[case(0f32, 4, "1.3", 21)] // Directory
#[ignore]
#[case(0.1f32, 4, "1.3", 21)] // Directory, with 10% size threshold fraction.
fn render_with_confirmed_delete_deletes_selected_item(
    #[case] size_threshold_fraction: f32,
    #[case] down_count: usize,
    #[case] expected_deleted_item_name: &str,
    #[case] expected_final_item_count: usize,
) -> anyhow::Result<()> {
    // Arrange
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(size_threshold_fraction)?;
    // Set accepted_license_terms to true, otherwise the license accept dialog would be shown, not the delete dialog.
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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    // The collected item and all children should be gone.
    assert_eq!(expected_final_item_count, view_state.total_items_in_tree);
    view_state.item_tree.iter().for_each(|item| {
        let item_ref = item.borrow();
        assert!(item_ref.path_segment != expected_deleted_item_name);
        assert!(!item_ref
            .path_segment
            .starts_with(format!("{}.", expected_deleted_item_name).as_str()));
    });

    // Also confirm that it has really been deleted from the file system.
    let view_state = make_test_view_state_from_path(&temp_dir_path, 0, 0, size_threshold_fraction)?;
    assert_eq!(expected_final_item_count, view_state.total_items_in_tree);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

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
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT - 1, 0, "1.10")] // down number of items in tree -> 1.10 selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT, 0, "1.10")] // down more than number of items in tree -> 1.10 selected
fn render_with_navigation_input_selects_correct_item(
    #[case] down_count: usize,
    #[case] up_count: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    assert_selected_item_name_eq(expected_selected_item_name, &view_state);

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
#[case(11, 0, TEST_DIRECTORY_TREE_ITEM_COUNT, 0, "1.10")] // down to 1.5.2 + page down xTEST_DIRECTORY_TREE_ITEM_COUNT -> 1.10 selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT - 2, 0, 1, 0, "1.10")] // down to 1.9 + page down -> 1.10 selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT - 1, 0, 1, 0, "1.10")] // down to 1.10 + page down -> 1.10 selected
fn render_with_page_up_or_down_navigation_input_selects_correct_item(
    #[case] down_count: usize,
    #[case] up_count: usize,
    #[case] page_down_count: usize,
    #[case] page_up_count: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    assert_selected_item_name_eq(expected_selected_item_name, &view_state);

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
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    assert_selected_item_name_eq(expected_selected_item_name, &view_state);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[rstest]
#[ignore]
#[case(0, "1.10")] // first given no selection -> last item selected
#[ignore]
#[case(10, "1.10")] // down to 1.5 + last -> last item selected
#[ignore]
#[case(TEST_DIRECTORY_TREE_ITEM_COUNT - 1, "1.10")] // down to last item + last -> last item selected
fn render_with_last_navigation_input_selects_last_item(
    #[case] down_count: usize,
    #[case] expected_selected_item_name: &str,
) -> anyhow::Result<()> {
    // Arrange
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
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

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    assert_selected_item_name_eq(expected_selected_item_name, &view_state);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[test]
#[ignore]
fn render_with_help_key_input_outputs_help() -> anyhow::Result<()> {
    // Arrange
    let mut view_state = ViewState::default();
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Char(HELP_KEY), KeyModifiers::NONE)), // Toggle display help
        Event::Key(KeyEvent::new(KeyCode::Char(HELP_KEY), KeyModifiers::NONE)), // Toggle hide help (can be any key)
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    output.matches(Regex::new(
        format!("^.*\\s+.*{}\\s+.*{}.*\\s+", QUIT_KEY_1, QUIT_KEY_2_SYMBOL).as_str(),
    )?)?;
    // TODO: Improve the output handling so that all help text can be checked, regardless of the height of
    //       the terminal.

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
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(size_threshold_fraction)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(
            KeyCode::Char(view_size_threshold_key),
            KeyModifiers::NONE,
        )),
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
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
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)), // Selects 1
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), // Collapses 1
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
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
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)), // Selects 1
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), // Collapses 1
        Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)), // Expands 1
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
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
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)), // Collapses children
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
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
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)), // Select 1
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), // Collapses 1
        Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)), // Expands children
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    assert_eq!(12, view_state.displayable_item_count);

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
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)), // Select 1
        Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)), // Collapses 1
        Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)), // Expands direct children
        Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)), // Expands all children
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    assert_eq!(
        TEST_DIRECTORY_TREE_ITEM_COUNT,
        view_state.displayable_item_count
    );

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}

#[test]
#[ignore]
fn render_with_accept_license_input_updates_view_state_and_writes_config_file() -> anyhow::Result<()>
{
    // Arrange
    // NOTE: The real terminal height will be used when this test runs, so make sure the test is independent
    //       of the terminal height.
    let (mut view_state, temp_dir_path) = make_test_view_state(0f32)?;
    view_state.accepted_license_terms = false;
    view_state.config_file_path =
        Some(std::env::temp_dir().join(format!("space_test_{}", uuid::Uuid::new_v4())));
    let mut output = TestOut::new();
    let mut input_event_source = TestInputEventSource::new(vec![
        Event::Key(KeyEvent::new(KeyCode::Char(DELETE_KEY), KeyModifiers::NONE)), // Shows delete dialog
        Event::Key(KeyEvent::new(
            KeyCode::Char(ACCEPT_LICENSE_TERMS_KEY),
            KeyModifiers::NONE,
        )), // Accepts license terms
        Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)), // Cancel delete dialog
        Event::Key(KeyEvent::new(KeyCode::Char(QUIT_KEY_1), KeyModifiers::NONE)),
    ]);

    // Act
    render(
        &mut view_state,
        &mut output,
        &mut input_event_source,
        &Skin::default(),
    )?;

    // Assert
    assert!(view_state.accepted_license_terms);
    view_state.accepted_license_terms = false;
    view_state.read_config_file()?;
    assert!(view_state.accepted_license_terms);

    delete_test_directory_tree(&temp_dir_path);

    Ok(())
}
