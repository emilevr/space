use crate::cli::row_item::{RowItem, RowItemType};
use crate::cli::skin::Skin;
use crate::cli::view_state::table_rows::get_row_cell_content_plain;
use space_rs::{Size, SizeDisplayFormat};
use std::{cell::RefCell, rc::Rc};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn make_dir_row_item(is_scanning: bool) -> Rc<RefCell<RowItem>> {
    Rc::new(RefCell::new(RowItem {
        size: Size::new(1000),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::Directory,
        incl_fraction: 1.0,
        peer_fraction: 0.0,
        path_segment: "test_dir".to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }))
}

fn make_file_row_item() -> Rc<RefCell<RowItem>> {
    Rc::new(RefCell::new(RowItem {
        size: Size::new(500),
        has_children: false,
        expanded: false,
        tree_prefix: String::default(),
        item_type: RowItemType::File,
        incl_fraction: 0.5,
        peer_fraction: 0.0,
        path_segment: "test_file.txt".to_string(),
        children: vec![],
        parent: None,
        descendant_count: 0,
        depth: 0,
        max_child_size: 0,
        row_index: 0,
        is_scanning: false,
        scanning_child_count: 0,
        access_denied: false,
        regex_visible: true,
    }))
}

/// Returns the path cell from `get_row_cell_content_plain` (index 1 in plain mode).
fn path_cell(item: &Rc<RefCell<RowItem>>, spinner_tick: usize) -> String {
    let cells = get_row_cell_content_plain(
        item,
        SizeDisplayFormat::Metric,
        &Skin::default(),
        spinner_tick,
    );
    // cells: [size, path, bars, percentage]
    cells[1].clone()
}

// ─── Spinner character rendering ──────────────────────────────────────────────

#[test]
fn format_path_cell_shows_spinner_char_when_directory_is_scanning() {
    let item = make_dir_row_item(true);
    // tick=0 -> first frame '⠋'
    let path = path_cell(&item, 0);
    assert!(
        path.contains('⠋'),
        "expected spinner frame '⠋' in path cell, got: {path:?}"
    );
}

#[test]
fn format_path_cell_shows_directory_icon_when_not_scanning() {
    let item = make_dir_row_item(false);
    let skin = Skin::default();
    let path = path_cell(&item, 0);
    assert!(
        path.contains(skin.item_type_directory_symbol),
        "expected directory icon '{}' in path cell, got: {path:?}",
        skin.item_type_directory_symbol
    );
}

#[test]
fn format_path_cell_does_not_show_directory_icon_when_scanning() {
    let item = make_dir_row_item(true);
    let skin = Skin::default();
    let path = path_cell(&item, 0);
    assert!(
        !path.contains(skin.item_type_directory_symbol),
        "did not expect directory icon when is_scanning=true, got: {path:?}"
    );
}

#[test]
fn format_path_cell_cycles_through_all_spinner_frames() {
    // SPINNER_FRAMES has 10 entries.
    let expected_frames: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let item = make_dir_row_item(true);

    for (tick, &expected_frame) in expected_frames.iter().enumerate() {
        let path = path_cell(&item, tick);
        assert!(
            path.contains(expected_frame),
            "tick={tick}: expected frame '{expected_frame}' in path cell, got: {path:?}"
        );
    }
}

#[test]
fn format_path_cell_spinner_wraps_after_all_frames() {
    let item = make_dir_row_item(true);
    // tick=10 wraps back to frame 0 ('⠋').
    let path_at_0 = path_cell(&item, 0);
    let path_at_10 = path_cell(&item, 10);
    assert_eq!(
        path_at_0, path_at_10,
        "expected tick=10 to wrap back to same frame as tick=0"
    );
}

#[test]
fn format_path_cell_file_not_affected_by_spinner_tick() {
    let item = make_file_row_item();
    let skin = Skin::default();
    // File items should always show the file icon, regardless of spinner_tick.
    for tick in [0, 5, 9, 10] {
        let path = path_cell(&item, tick);
        assert!(
            path.contains(skin.item_type_file_symbol),
            "tick={tick}: expected file icon in path cell, got: {path:?}"
        );
        // No spinner character should appear in a file's path cell.
        for frame in &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'] {
            assert!(
                !path.contains(*frame),
                "tick={tick}: unexpected spinner char '{frame}' in file path cell"
            );
        }
    }
}

// ─── spinner_tick field on ViewState ─────────────────────────────────────────

#[test]
fn view_state_default_spinner_tick_is_zero() {
    use crate::cli::view_state::ViewState;
    let view_state = ViewState::default();
    assert_eq!(0, view_state.spinner_tick);
}
