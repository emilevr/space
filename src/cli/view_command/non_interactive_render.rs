#[cfg(test)]
#[path = "non_interactive_render_test.rs"]
mod non_interactive_render_test;

use crate::cli::{
    row_item::RowItem,
    skin::Skin,
    tui,
    view_state::{table_rows, ViewState},
};
use anyhow::Context;
use crossterm::{style::Print, QueueableCommand};
use ratatui::prelude::*;
use space_rs::SizeDisplayFormat;
use std::{
    cell::RefCell,
    io::Write,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};
use unicode_segmentation::UnicodeSegmentation;

/// How many rows to render between checks of the cancellation flag.
const CANCEL_CHECK_INTERVAL: usize = 100;

pub(crate) fn render_rows<W: Write>(
    view_state: ViewState,
    size_threshold_fraction: f32,
    writer: &mut W,
    skin: &Skin,
    should_exit: &AtomicBool,
) -> anyhow::Result<usize> {
    let mut rendered_count = 0;
    let mut rows_since_check: usize = 0;
    let mut backend = CrosstermBackend::new(writer);

    let width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(120);

    // Exclude the expand column.  Use half the terminal width for the path
    // column since non-interactive mode doesn't have visible rows to measure.
    let path_width = width / 2;
    let constraints: Vec<Constraint> = tui::table_column_constraints(width, path_width)
        .into_iter()
        .enumerate()
        .filter_map(
            |(index, element)| {
                if index != 1 {
                    Some(element)
                } else {
                    None
                }
            },
        )
        .collect();

    view_state
        .item_tree
        .iter()
        .try_for_each(|item| {
            rendered_count += render_row(
                item,
                size_threshold_fraction,
                &constraints,
                width,
                view_state.size_display_format,
                &mut backend,
                skin,
                should_exit,
                &mut rows_since_check,
            )?;
            anyhow::Ok(())
        })
        .context("An error occurred while rendering a row!")?;

    std::io::Write::flush(&mut backend)?;

    Ok(rendered_count)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_row<W: Write>(
    item: &Rc<RefCell<RowItem>>,
    size_threshold_fraction: f32,
    constraints: &Vec<Constraint>,
    terminal_width: u16,
    size_display_format: SizeDisplayFormat,
    backend: &mut CrosstermBackend<W>,
    skin: &Skin,
    should_exit: &AtomicBool,
    rows_since_check: &mut usize,
) -> anyhow::Result<usize> {
    let mut rendered_count = 0;
    let item_ref = item.borrow();

    if item_ref.incl_fraction < size_threshold_fraction {
        return Ok(rendered_count);
    }
    if !item_ref.regex_visible {
        return Ok(rendered_count);
    }

    // Periodically check for cancellation.
    *rows_since_check += 1;
    if *rows_since_check >= CANCEL_CHECK_INTERVAL {
        *rows_since_check = 0;
        if should_exit.load(Ordering::Relaxed) {
            anyhow::bail!("Cancelled.");
        }
    }

    let cells = table_rows::get_row_cell_content_plain(item, size_display_format, skin, 0);

    let column_count = constraints.len();
    for col_index in 0..column_count {
        let mut max_len = constraints[col_index].apply(terminal_width) as usize;
        if col_index == 1 {
            // We subtract one extra char as we are using graphemes for the item type icons, which are
            // typically wider than a single normal char.
            max_len -= 1;
        }

        // Get the potentially truncated content, making sure we truncate at valid character boundaries.
        let content = cells[col_index]
            .graphemes(true)
            .take(max_len)
            .collect::<String>();

        backend.queue(Print(format!("{content:max_len$}")))?;

        if col_index < column_count - 1 {
            backend.queue(Print(' '))?;
        } else {
            backend.queue(Print('\n'))?;
        }
    }

    rendered_count += 1;

    if item_ref.has_children {
        for child in &item_ref.children {
            rendered_count += render_row(
                child,
                size_threshold_fraction,
                constraints,
                terminal_width,
                size_display_format,
                backend,
                skin,
                should_exit,
                rows_since_check,
            )?;
        }
    }

    Ok(rendered_count)
}
