use super::rendering::expand_area;
use super::{ACCEPT_LICENSE_TERMS_KEY, CONFIRM_DELETE_KEY};
use crate::cli::{
    skin::Skin,
    view_state::{table_rows::SPINNER_FRAMES, DeletionState, ViewState},
};
use ratatui::{
    buffer,
    layout::{Alignment, Constraint, Layout},
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap},
    Frame,
};
use std::cmp::{max, min};

macro_rules! get_dialog_area {
    ($dialog:expr, $width_percent:expr, $ideal_width:expr, $available_area:expr) => {
        {
            // We use a preliminary area with only a width specified. The height will be determined by the
            // available area.
            let area = centered_rect($width_percent, $ideal_width, 0, $available_area);

            // Measure the required height.
            // - We subtract 2 from the width for the vertical borders.
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

pub(in crate::cli) fn render_delete_dialog<B: Backend>(
    f: &mut Frame<B>,
    view_state: &mut ViewState,
    skin: &Skin,
) {
    match view_state.deletion_state {
        DeletionState::InProgress => {
            let spinner = SPINNER_FRAMES[view_state.spinner_tick % SPINNER_FRAMES.len()];
            let lines = vec![
                Line::from(vec![Span::raw(format!("{spinner} Deleting... "))]),
                Line::default(),
                Line::from("Press Escape to cancel."),
            ];
            render_centered_dialog(f, lines, Alignment::Center);
        }
        DeletionState::Cancelling => {
            let spinner = SPINNER_FRAMES[view_state.spinner_tick % SPINNER_FRAMES.len()];
            let lines = vec![Line::from(format!("{spinner} Cancelling deletion..."))];
            render_centered_dialog(f, lines, Alignment::Center);
        }
        DeletionState::Idle => {
            let Some(selected_item) = view_state.get_selected_item() else {
                return;
            };

            let value_style = skin.value_style();
            let selected_item_ref = selected_item.borrow();
            let is_dir =
                selected_item_ref.item_type == crate::cli::row_item::RowItemType::Directory;

            let lines = build_delete_dialog_lines(
                &selected_item_ref,
                is_dir,
                view_state,
                &value_style,
                skin,
            );

            render_centered_dialog(f, lines, Alignment::Center);
        }
    }
}

fn build_delete_dialog_lines<'a>(
    selected_item_ref: &std::cell::Ref<'a, crate::cli::row_item::RowItem>,
    is_dir: bool,
    view_state: &ViewState,
    value_style: &Style,
    skin: &Skin,
) -> Vec<Line<'a>> {
    vec![
        Line::from(vec![
            Span::raw("Delete "),
            Span::raw(if is_dir { "directory " } else { "file " }),
            Span::styled(selected_item_ref.path_segment.clone(), *value_style),
            Span::styled(
                if is_dir {
                    " - including all files and sub-directories"
                } else {
                    ""
                },
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ?"),
        ]),
        Line::default(),
        Line::from(vec![
            Span::raw("This will free up at least "),
            Span::styled(
                selected_item_ref
                    .size
                    .to_string(view_state.size_display_format),
                *value_style,
            ),
        ]),
        Line::default(),
        build_delete_warning_line(is_dir, view_state, skin),
        Line::default(),
        Line::from(vec![
            Span::raw("Press ["),
            Span::styled(CONFIRM_DELETE_KEY.to_string(), *value_style),
            Span::raw("]es to confirm. Any other key to cancel."),
        ]),
    ]
}

fn build_delete_warning_line<'a>(is_dir: bool, view_state: &ViewState, skin: &Skin) -> Line<'a> {
    Line::from(Span::styled(
        format!(
            "WARNING: Deletion is permanent in order to free up space. {}Proceed with caution!",
            if is_dir && view_state.size_threshold_fraction > 0f32 {
                format!(
                    "The current view size threshold percentage is {:0}%, i.e. not all files that will be deleted may be visible in the UI. ",
                    view_state.size_threshold_fraction * 100f32
                )
            } else {
                "".into()
            }
        ),
        Style::default().fg(skin.delete_warning_text_fg_color),
    ))
}

pub(in crate::cli) fn render_accept_license_terms_dialog<B: Backend>(
    f: &mut Frame<B>,
    skin: &Skin,
) {
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

    render_centered_dialog(f, lines, Alignment::Left);
}

fn render_centered_dialog<B: Backend>(
    f: &mut Frame<B>,
    lines: Vec<Line<'_>>,
    alignment: Alignment,
) {
    // Create the dialog content, without a block so we can measure the contents only.
    let content = Paragraph::new(lines.clone())
        .alignment(alignment)
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
        .alignment(alignment)
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
