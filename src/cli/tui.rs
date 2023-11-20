use super::{
    input_event_source::InputEventSource,
    row_item::RowItemType,
    skin::Skin,
    view_state::{
        get_excl_percentage_column_width, ViewState, APPARENT_SIZE_COLUMN_WIDTH,
        EXPAND_INDICATOR_COLUMN_WIDTH, INCL_PERCENTAGE_COLUMN_WIDTH,
    },
};
#[cfg(not(test))]
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
    style::{Modifier, Style},
    widgets::{
        Block, Borders, Cell, Clear, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Widget, Wrap,
    },
    Frame, Terminal,
};
use std::{
    cmp::{max, min},
    io::Write,
};

#[cfg(test)]
#[path = "./tui_test.rs"]
mod tui_test;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const HELP_KEY: char = '?';
pub(crate) const QUIT_KEY_1: char = 'q';
pub(crate) const COLLAPSE_SELECTED_CHILDREN_KEY: char = '-';
pub(crate) const COLLAPSE_SELECTED_CHILDREN_KEY_ALT: char = '_';
pub(crate) const EXPAND_SELECTED_CHILDREN_KEY: char = '+';
pub(crate) const EXPAND_SELECTED_CHILDREN_KEY_ALT: char = '=';
pub(crate) const VIEW_SIZE_THRESHOLD_0_PERCENT_KEY: char = '0';
pub(crate) const VIEW_SIZE_THRESHOLD_10_PERCENT_KEY: char = '1';
pub(crate) const VIEW_SIZE_THRESHOLD_20_PERCENT_KEY: char = '2';
pub(crate) const VIEW_SIZE_THRESHOLD_30_PERCENT_KEY: char = '3';
pub(crate) const VIEW_SIZE_THRESHOLD_40_PERCENT_KEY: char = '4';
pub(crate) const VIEW_SIZE_THRESHOLD_50_PERCENT_KEY: char = '5';
pub(crate) const VIEW_SIZE_THRESHOLD_60_PERCENT_KEY: char = '6';
pub(crate) const VIEW_SIZE_THRESHOLD_70_PERCENT_KEY: char = '7';
pub(crate) const VIEW_SIZE_THRESHOLD_80_PERCENT_KEY: char = '8';
pub(crate) const VIEW_SIZE_THRESHOLD_90_PERCENT_KEY: char = '9';
pub(crate) const DELETE_KEY: char = 'd';
pub(crate) const CONFIRM_DELETE_KEY: char = 'y';
pub(crate) const ACCEPT_LICENSE_TERMS_KEY: char = 'a';

pub(crate) const QUIT_KEY_2_SYMBOL: &str = "Esc";
pub(crate) const SELECT_PREV_KEY_SYMBOL: char = '↑';
pub(crate) const SELECT_NEXT_KEY_SYMBOL: char = '↓';
pub(crate) const COLLAPSE_KEY_SYMBOL: char = '←';
pub(crate) const EXPAND_KEY_SYMBOL: char = '→';
pub(crate) const SELECT_PREV_PAGE_KEY_SYMBOL: &str = "Page Up";
pub(crate) const SELECT_NEXT_PAGE_KEY_SYMBOL: &str = "Page Down";
pub(crate) const SELECT_FIRST_KEY_SYMBOL: &str = "Home";
pub(crate) const SELECT_LAST_KEY_SYMBOL: &str = "End";

pub(crate) fn render<W: Write, I: InputEventSource>(
    view_state: &mut ViewState,
    writer: &mut W,
    input_event_source: &mut I,
    skin: &Skin,
) -> anyhow::Result<()> {
    enable_raw_mode()?;

    execute!(writer, enter_terminal_command(), EnableMouseCapture)?;
    let backend = CrosstermBackend::new(writer);
    let mut terminal = Terminal::new(backend)?;

    let result = render_loop(&mut terminal, view_state, input_event_source, skin);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        exit_terminal_command(),
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

#[cfg(not(test))]
fn enter_terminal_command() -> EnterAlternateScreen {
    EnterAlternateScreen
}

#[cfg(not(test))]
fn exit_terminal_command() -> LeaveAlternateScreen {
    LeaveAlternateScreen
}

#[cfg(test)]
fn enter_terminal_command() -> crossterm::terminal::Clear {
    crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
}

#[cfg(test)]
fn exit_terminal_command() -> DisableMouseCapture {
    DisableMouseCapture
}

fn render_loop<B: Backend, I: InputEventSource>(
    terminal: &mut Terminal<B>,
    view_state: &mut ViewState,
    input_event_source: &mut I,
    skin: &Skin,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| create_frame(f, view_state, skin))?;

        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = input_event_source.read_event()?
        {
            if view_state.show_help {
                view_state.show_help = false;
            } else if view_state.show_delete_dialog {
                if view_state.accepted_license_terms {
                    match code {
                        KeyCode::Char(CONFIRM_DELETE_KEY) => {
                            view_state.delete_selected_item();
                            view_state.show_delete_dialog = false;
                        }
                        _ => view_state.show_delete_dialog = false,
                    }
                } else {
                    match code {
                        KeyCode::Char(ACCEPT_LICENSE_TERMS_KEY) => {
                            view_state.accept_license_terms();
                        }
                        _ => view_state.show_delete_dialog = false,
                    }
                }
            } else {
                match code {
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_0_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_10_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.01f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_20_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.02f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_30_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.03f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_40_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.04f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_50_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.05f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_60_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.06f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_70_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.07f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_80_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.08f32)
                    }
                    KeyCode::Char(VIEW_SIZE_THRESHOLD_90_PERCENT_KEY) => {
                        view_state.set_size_threshold_fraction(0.09f32)
                    }
                    KeyCode::Char(HELP_KEY) => view_state.show_help = true,
                    KeyCode::Char(DELETE_KEY) => view_state.show_delete_dialog = true,
                    KeyCode::Char(QUIT_KEY_1) | KeyCode::Esc => return Ok(()),
                    KeyCode::Left => view_state.collapse_selected_item(),
                    KeyCode::Right => view_state.expand_selected_item(),
                    KeyCode::Up => view_state.previous(1),
                    KeyCode::Down => view_state.next(1),
                    KeyCode::PageUp => view_state.previous(view_state.visible_height),
                    KeyCode::PageDown => view_state.next(view_state.visible_height),
                    KeyCode::Home => view_state.first(),
                    KeyCode::End => view_state.last(),
                    KeyCode::Char(COLLAPSE_SELECTED_CHILDREN_KEY)
                    | KeyCode::Char(COLLAPSE_SELECTED_CHILDREN_KEY_ALT) => {
                        view_state.collapse_selected_children()
                    }
                    KeyCode::Char(EXPAND_SELECTED_CHILDREN_KEY)
                    | KeyCode::Char(EXPAND_SELECTED_CHILDREN_KEY_ALT) => {
                        view_state.expand_selected_children()
                    }
                    _ => {}
                }
            }
        }
    }
}

fn create_frame<B: Backend>(f: &mut Frame<B>, view_state: &mut ViewState, skin: &Skin) {
    let view_height = f.size().height;
    let table_height = view_height - 1; // Subract one for the help header.

    view_state.visible_height = table_height as usize - 1; // Subtrace one for the column headers.

    let vertical_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(table_height)].as_ref())
        .split(f.size());
    let width = vertical_rects[0].width;
    let horizontal_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(width - 1), Constraint::Length(1)].as_ref())
        .split(vertical_rects[1]);

    render_title_bar(f, view_state, &vertical_rects[0], skin);
    render_table(f, view_state, &horizontal_rects[0], skin);
    render_vertical_scrollbar(f, view_state, &horizontal_rects[1], skin);

    if view_state.show_help {
        render_help(f, skin);
    } else if view_state.show_delete_dialog {
        if view_state.accepted_license_terms {
            render_delete_dialog(f, view_state, skin);
        } else {
            render_accept_license_terms_dialog(f, skin);
        }
    }
}

fn render_title_bar<B: Backend>(f: &mut Frame<B>, data: &ViewState, area: &Rect, skin: &Skin) {
    let title_style = Style::default()
        .fg(skin.title_fg_color)
        .bg(skin.title_bg_color);
    let version_style = Style::default()
        .fg(skin.version_fg_color)
        .bg(skin.title_bg_color);

    let title = "Space";
    let version_display = format!("v{VERSION}");
    let size_filter_display = format!("≥ {:.0}%", data.size_threshold_fraction * 100f32);

    let available_width = area.width as i32
        - (title.len() + version_display.len() + size_filter_display.len()) as i32
        - 3  // Subtract 3 for the column separators
        - 1; // Subtract 1 to account for table scrollbar.

    let (key_help, available_width) = get_key_help(available_width, skin);

    let key_help_len = key_help.width();
    let title_cells = [
        Cell::from(title),
        Cell::from(version_display.as_str()).set_style(version_style),
        Cell::from(""), // Spacer
        Cell::from(key_help),
        Cell::from(""), // Spacer
        Cell::from(size_filter_display.as_str()),
    ];

    let title_row = Row::new(title_cells)
        .style(title_style)
        .height(1)
        .bottom_margin(0);

    let spacer_length = max(0, available_width / 2) as u16;

    let title_widths = [
        Constraint::Length(title.len() as u16),
        Constraint::Length(version_display.len() as u16),
        Constraint::Length(spacer_length),
        Constraint::Length(key_help_len as u16),
        Constraint::Length(spacer_length),
        Constraint::Length(size_filter_display.len() as u16),
    ];
    let title_table = Table::new([])
        .header(title_row)
        .block(Block::default().borders(Borders::NONE))
        .widths(&title_widths);
    f.render_widget(title_table, *area);
}

/// Add only as many key help entries as we have space for. We add the more important ones first.
fn get_key_help<'a>(available_width: i32, skin: &Skin) -> (Line<'a>, i32) {
    let mut available_width = available_width;
    let key_style = Style::default()
        .bg(skin.title_bg_color)
        .fg(skin.key_help_key_fg_color);
    let key_help_style = Style::default()
        .bg(skin.title_bg_color)
        .fg(skin.title_fg_color);

    #[rustfmt::skip]
    let all_key_help = vec![
        Span::styled(format!(" {HELP_KEY}"), key_style), Span::styled(" Help ", key_help_style),
        Span::styled(format!(" {QUIT_KEY_1}/{QUIT_KEY_2_SYMBOL}"), key_style), Span::styled(" Quit ", key_help_style),
        Span::styled(format!(" {DELETE_KEY}"), key_style), Span::styled(" Delete ", key_help_style),
        Span::styled(format!(" {SELECT_PREV_KEY_SYMBOL}{SELECT_NEXT_KEY_SYMBOL}"), key_style), Span::styled(" Selection ", key_help_style),
        Span::styled(format!(" {COLLAPSE_KEY_SYMBOL}{EXPAND_KEY_SYMBOL}"), key_style), Span::styled(" Collapse/Expand ", key_help_style),
    ];
    let mut key_help = vec![];
    for i in (0..all_key_help.len()).step_by(2) {
        let shortcut = &all_key_help[i];
        let shortcut_width = shortcut.width();
        let help = &all_key_help[i + 1];
        let help_width = help.width();
        if shortcut_width as i32 + help_width as i32 <= available_width {
            key_help.push(shortcut.clone());
            key_help.push(help.clone());
            available_width -= shortcut_width as i32 + help_width as i32;
        } else {
            break;
        }
    }

    (Line::from(key_help), available_width)
}

fn render_table<B: Backend>(
    f: &mut Frame<B>,
    view_state: &mut ViewState,
    area: &Rect,
    skin: &Skin,
) {
    let table_header_style = Style::default()
        .bg(skin.table_header_bg_color)
        .fg(skin.table_header_fg_color);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let table_selected_index = view_state.table_selected_index;

    let header_cells = ["Size", "", "Path", "", "Incl"]
        .iter()
        .map(|h| Cell::from(*h));
    let header = Row::new(header_cells)
        .style(table_header_style)
        .height(1)
        .bottom_margin(0);

    // NB! Update the table width with the current available width.
    view_state.table_width = area.width;
    // Only then update the visible rows, as we need the available width here.
    let rows = view_state.update_visible_rows();

    let constraints = table_column_contraints(area.width);
    let table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(selected_style)
        .highlight_symbol("")
        .widths(&constraints);
    f.render_stateful_widget(
        table,
        *area,
        &mut TableState::default().with_selected(Some(table_selected_index)),
    );
}

fn render_vertical_scrollbar<B: Backend>(
    f: &mut Frame<B>,
    data: &mut ViewState,
    area: &Rect,
    skin: &Skin,
) {
    // The content length must be the total number of rows minus the number of visible rows.
    let content_length = if data.displayable_item_count > data.visible_height {
        data.displayable_item_count - data.visible_height
    } else {
        data.displayable_item_count
    };

    let mut vertical_scroll_state = ScrollbarState::default()
        .position(data.visible_offset as u16)
        .content_length(content_length as u16)
        .viewport_content_length(data.visible_height as u16);

    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(symbols::scrollbar::Set {
                track: symbols::line::VERTICAL,
                thumb: symbols::block::FULL,
                begin: "▲",
                end: "▼",
            })
            .style(
                Style::default()
                    .bg(skin.title_bg_color)
                    .fg(skin.title_fg_color),
            ),
        *area,
        &mut vertical_scroll_state,
    );
}

fn render_help<B: Backend>(f: &mut Frame<B>, skin: &Skin) {
    let block = Block::default().title("Help").borders(Borders::ALL);
    let mut area = f.size();
    f.render_widget(Clear, area); // Clear out the background
    f.render_widget(block, area);

    contract_area(&mut area, 2, 2);

    let key_style = Style::default()
        .fg(skin.title_fg_color)
        .bg(skin.table_header_bg_color);
    let danger_key_style = Style::default()
        .fg(skin.title_fg_color)
        .bg(skin.key_help_danger_bg_color);
    let section_header_style = Style::default().add_modifier(Modifier::BOLD);

    let key_column_size: usize = 11;
    let column_constraints = [
        Constraint::Length(key_column_size as u16),
        Constraint::Length(key_column_size as u16),
        Constraint::Length(area.width - key_column_size as u16 * 2),
    ];

    render_help_section(
        "General",
        vec![
            Row::new(vec![
                Cell::from(format!("{QUIT_KEY_1:^key_column_size$}")).style(key_style),
                Cell::from(format!("{QUIT_KEY_2_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from("Quit the application"),
            ]),
            Row::new(Vec::<Cell>::with_capacity(0)),
            Row::new(vec![
                Cell::from(format!("{DELETE_KEY:^key_column_size$}")).style(danger_key_style),
                Cell::from(""),
                Cell::from("Delete selected item"),
            ]),
            Row::new(vec![
                Cell::from(""),
                Cell::from(format!("{CONFIRM_DELETE_KEY:^key_column_size$}"))
                    .style(danger_key_style),
                Cell::from("Confirm delete (on delete dialog)"),
            ]),
        ],
        section_header_style,
        column_constraints,
        f,
        &mut area,
    );

    render_help_section(
        "Navigation",
        vec![
            Row::new(vec![
                Cell::from(format!("{SELECT_PREV_KEY_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Select previous item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_NEXT_KEY_SYMBOL:^key_column_size$}",)).style(key_style),
                Cell::from(""),
                Cell::from("Select next item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_PREV_PAGE_KEY_SYMBOL:^key_column_size$}"))
                    .style(key_style),
                Cell::from(""),
                Cell::from("Select corresponding item on previous page"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_NEXT_PAGE_KEY_SYMBOL:^key_column_size$}"))
                    .style(key_style),
                Cell::from(""),
                Cell::from("Select corresponding item on next page item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_FIRST_KEY_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Select first item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{SELECT_LAST_KEY_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Select last item"),
            ]),
        ],
        section_header_style,
        column_constraints,
        f,
        &mut area,
    );

    render_help_section(
        "Filtering",
        vec![
            Row::new(vec![
                Cell::from(format!("{COLLAPSE_KEY_SYMBOL:^key_column_size$}",)).style(key_style),
                Cell::from(""),
                Cell::from("Collapse selected directory item"),
            ]),
            Row::new(vec![
                Cell::from(format!("{EXPAND_KEY_SYMBOL:^key_column_size$}")).style(key_style),
                Cell::from(""),
                Cell::from("Expand selected directory item"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{COLLAPSE_SELECTED_CHILDREN_KEY:^key_column_size$}"
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("If all children are collapsed, then collapse the selected item\nIf one or more children are expanded, then collapse all children of the selected item"),
            ])
            .height(2),
            Row::new(vec![
                Cell::from(format!("{EXPAND_SELECTED_CHILDREN_KEY:^key_column_size$}"))
                    .style(key_style),
                Cell::from(""),
                Cell::from("If selected item is collapsed, then expand and show collapsed children\nIf selected directory item is expanded, then expand all descendants"),
            ])
            .height(2),
            Row::new(Vec::<Cell>::with_capacity(0)),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_0_PERCENT_KEY:^key_column_size$}"
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to  0%, i.e. do not hide any items"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_10_PERCENT_KEY:^key_column_size$}"
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 10%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_20_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 20%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_30_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 30%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_40_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 40%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_50_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 50%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_60_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 60%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_70_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 70%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_80_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 80%"),
            ]),
            Row::new(vec![
                Cell::from(format!(
                    "{VIEW_SIZE_THRESHOLD_90_PERCENT_KEY:^key_column_size$}",
                ))
                .style(key_style),
                Cell::from(""),
                Cell::from("Set view size threshold to 90%"),
            ]),
        ],
        section_header_style,
        column_constraints,
        f,
        &mut area,
    );
}

fn render_help_section<B: Backend>(
    title: &str,
    rows: Vec<Row<'_>>,
    title_style: Style,
    column_constraints: [Constraint; 3],
    f: &mut Frame<'_, B>,
    area: &mut Rect,
) {
    let row_count = rows.len() as u16;
    let table = Table::new(rows)
        .block(Block::default().title(title).title_style(title_style))
        .widths(&column_constraints)
        .column_spacing(1);
    f.render_widget(table, *area);
    remove_area_top(area, row_count + 2);
}

fn expand_area(area: &mut Rect, cx: u16, cy: u16, area_limit: Rect) {
    if area.x >= cx {
        area.x -= cx;
    } else {
        area.x = 0;
    }

    if area.y >= cy {
        area.y -= cy;
    } else {
        area.y = 0;
    }

    area.width += cx * 2;
    area.height += cy * 2;

    *area = area.intersection(area_limit);
}

fn contract_area(rect: &mut Rect, cx: u16, cy: u16) {
    rect.x += cx;
    rect.y += cy;

    if rect.width >= cx * 2 {
        rect.width -= cx * 2;
    } else {
        rect.width = 0;
    }

    if rect.height >= cy * 2 {
        rect.height -= cy * 2;
    } else {
        rect.height = 0;
    }
}

fn remove_area_top(rect: &mut Rect, by_y: u16) {
    rect.y += by_y;
    if rect.height >= by_y {
        rect.height -= by_y;
    } else {
        rect.height = 0;
    }
}

macro_rules! get_dialog_area {
    ($dialog:expr, $width_percent:expr, $ideal_width:expr, $available_area:expr) => {
        {
            // We use a preliminary area with only a width specified. The height will be determined by the
            // available area.
            let area = centered_rect($width_percent, $ideal_width, 0, $available_area);

            // Measure the required height.
            // - We subract 2 from the width for the vertical borders.
            // - We add 2 to the height to account for the horizontal borders.
            let content_height = measure_height!($dialog, area.width - 2, area.height) + 2;

            // Now return a rectangle with exact measured height, or the area height if the measured height exceeded that.
            centered_rect(80, area.width, content_height, area)
        }
    };
}

macro_rules! measure_height {
    ($dialog:expr, $width:expr, $max_height:expr) => {{
        let area = Rect {
            x: 0,
            y: 0,
            width: $width,
            height: $max_height,
        };
        let mut buffer = Buffer::empty(area);

        $dialog.render(area, &mut buffer);

        // Find the first non-empty line from the bottom up.
        let buffer_content = buffer.content();
        let mut non_empty_cell_index = 0;
        let default_cell = buffer::Cell::default();
        for i in (0..buffer_content.len()).rev() {
            if !default_cell.eq(&buffer_content[i]) {
                non_empty_cell_index = i as u16;
                break;
            }
        }

        // Return the last row number that had content
        (non_empty_cell_index + ($width) - 1) / ($width)
    }};
}

fn render_delete_dialog<B: Backend>(f: &mut Frame<B>, view_state: &mut ViewState, skin: &Skin) {
    if let Some(selected_item) = view_state.get_selected_item() {
        let value_style = skin.value_style();
        let selected_item_ref = selected_item.borrow();
        let is_dir = selected_item_ref.item_type == RowItemType::Directory;

        let lines = vec![
            Line::from(vec![
                Span::raw("Delete "),
                Span::raw(if is_dir {
                    "directory "
                } else {
                    "file "
                }),
                Span::styled(&selected_item_ref.path_segment, value_style),
                Span::styled(if is_dir {
                    " - including all files and sub-directories"
                } else {
                    ""
                }, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" ?"),
            ]),
            Line::default(),
            Line::from(vec![
                Span::raw("This will free up at least "),
                Span::styled(
                    selected_item_ref.size.to_string(view_state.size_display_format),
                    value_style,
                ),
            ]),
            Line::default(),
            Line::from(Span::styled(
                format!("WARNING: Deletion is permanent in order to free up space. {}Proceed with caution!",
                    if is_dir && view_state.size_threshold_fraction > 0f32 {
                        format!(
                            "The current view size threshold percentage is {:0}%, i.e. not all files that will be deleted may be visible in the UI. ",
                            view_state.size_threshold_fraction * 100f32
                        )
                    } else {
                        "".into()
                    }),
                Style::default().fg(skin.delete_warning_text_fg_color),
            )),
            Line::default(),
            Line::from(vec![
                Span::raw("Press ["),
                Span::styled(CONFIRM_DELETE_KEY.to_string(), value_style),
                Span::raw("]es to confirm. Any other key to cancel."),
            ]),
        ];

        // Create the dialog content, without a block so we can measure the contents only.
        let content = Paragraph::new(lines.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        let mut area = get_dialog_area!(content, 80, 30, f.size());
        expand_area(&mut area, 1, 1, f.size());

        // Recreate content with styling etc, for rendering.
        let content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::uniform(1)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        // Clear the background for the dialog.
        f.render_widget(Clear, area);

        f.render_widget(content, area);
    }
}

fn render_accept_license_terms_dialog<B: Backend>(f: &mut Frame<B>, skin: &Skin) {
    let value_style = skin.value_style();
    let bold_style = Style::default().add_modifier(Modifier::BOLD);

    #[rustfmt::skip]
    let lines = vec![
        Line::from(Span::styled(
r#"Before you are able to delete you have to accept the license terms once. Don't worry, it's free, but there
 are some conditions to take note of. Bottom line: use at your own risk."#,
            value_style,
        )),
        Line::default(),
        Line::from(Span::styled("MIT License",Style::default().add_modifier(Modifier::BOLD))),
        Line::default(),
        Line::from(vec![
            Span::raw(
r#"THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. "#),
            Span::styled(
r#"IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE."#, bold_style)]),
        Line::default(),
        Line::from(vec![Span::raw(
r#"By accepting, you agree to be bound by the terms of the MIT License. You can review the full license text
 at https://github.com/emilevr/space/blob/main/LICENSE"#)]),
        Line::default(),
        Line::from(vec![
            Span::raw("Press ["),
            Span::styled(ACCEPT_LICENSE_TERMS_KEY.to_string(), value_style),
            Span::raw("] to accept the license terms. Any other key to cancel."),
        ]),
    ];

    // Create the dialog content, without a block so we can measure the contents only.
    let content = Paragraph::new(lines.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    let mut area = get_dialog_area!(content, 80, 30, f.size());
    expand_area(&mut area, 1, 1, f.size());

    // Recreate content with styling etc, for rendering.
    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .padding(Padding::uniform(1)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    // Clear the background for the dialog.
    f.render_widget(Clear, area);

    f.render_widget(content, area);
}

fn centered_rect(width_percent: u16, ideal_width: u16, height: u16, area: Rect) -> Rect {
    let rect_height = match height {
        0 => area.height,
        _ => min(height, area.height),
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((area.height - rect_height) / 2),
                Constraint::Length(rect_height),
                Constraint::Length((area.height - rect_height) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    let rect_width = min(
        max((area.width * width_percent) / 100, ideal_width),
        area.width,
    );
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((area.width - rect_width) / 2),
                Constraint::Length(rect_width),
                Constraint::Length((area.width - rect_width) / 2),
            ]
            .as_ref(),
        )
        .split(layout[1])[1]
}

pub(crate) fn table_column_contraints(width: u16) -> [Constraint; 5] {
    let excl_percentage_column_width = get_excl_percentage_column_width(width);
    let remaining_width = width as i32
        - APPARENT_SIZE_COLUMN_WIDTH as i32
        - EXPAND_INDICATOR_COLUMN_WIDTH as i32
        - excl_percentage_column_width as i32
        - INCL_PERCENTAGE_COLUMN_WIDTH as i32
        - 4;

    let remaining_width = if remaining_width < 0 {
        0
    } else {
        remaining_width as u16
    };

    [
        Constraint::Length(APPARENT_SIZE_COLUMN_WIDTH),
        Constraint::Length(EXPAND_INDICATOR_COLUMN_WIDTH),
        Constraint::Length(remaining_width),
        Constraint::Length(excl_percentage_column_width),
        Constraint::Length(INCL_PERCENTAGE_COLUMN_WIDTH),
    ]
}
