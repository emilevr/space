#[cfg(not(test))]
use super::crossterm_input_event_source::CrosstermInputEventSource;
use super::{
    cli_command::CliCommand,
    environment::EnvServiceTrait,
    row_item::RowItem,
    skin::Skin,
    tui,
    view_state::{self, ViewState},
};
use anyhow::{bail, Context};
use crossterm::{style::Print, QueueableCommand};
use ratatui::prelude::*;
use space_rs::{DirectoryItem, SizeDisplayFormat};
use std::{
    cell::RefCell,
    io::Write,
    path::PathBuf,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc},
};
use unicode_segmentation::UnicodeSegmentation;

#[cfg(test)]
#[path = "./view_command_test.rs"]
mod view_command_test;

const COLORTERM_ENV_VAR: &str = "COLORTERM";
const TERM_ENV_VAR: &str = "TERM";

pub(crate) struct ViewCommand {
    target_paths: Option<Vec<PathBuf>>,
    size_display_format: Option<SizeDisplayFormat>,
    size_threshold_percentage: u8,
    #[cfg(not(test))]
    non_interactive: bool,
    total_size_in_bytes: u64,
    env_service: Box<dyn EnvServiceTrait>,
    should_exit: Arc<AtomicBool>,
}

impl CliCommand for ViewCommand {
    fn prepare(&mut self) -> anyhow::Result<&mut Self> {
        let has_target_paths = match &self.target_paths {
            Some(target_paths) => !target_paths.is_empty(),
            None => false,
        };

        if has_target_paths {
            // Clean and validate
            let mut target_paths = self.target_paths.clone().unwrap();
            target_paths.sort();
            target_paths.dedup();
            for target_path in &target_paths {
                if !target_path.exists() {
                    bail!("{} does not exist!", target_path.display());
                }
            }
            // Update args with cleaned target paths.
            self.target_paths = Some(target_paths);
        } else {
            self.target_paths = Some(vec![self.env_service.current_dir()?]);
        }

        Ok(self)
    }

    fn run<W: Write>(&mut self, writer: &mut W) -> anyhow::Result<()> {
        if let Some(target_paths) = &self.target_paths {
            if target_paths.len() == 1 {
                writeln!(writer, "Analyzing path {}", target_paths[0].display())?;
            } else {
                writeln!(writer, "Analyzing the following paths:")?;
                target_paths.iter().try_for_each(|path| {
                    writeln!(writer, "  - {}", path.display())?;
                    anyhow::Ok(())
                })?;
            }
        }
        writeln!(
            writer,
            "This could take a while depending on the size of the tree ..."
        )?;

        let items = self.get_directory_items();

        let size_display_format = match &self.size_display_format {
            Some(size_display_format) => *size_display_format,
            _ => SizeDisplayFormat::Metric,
        };

        let size_threshold_fraction = self.size_threshold_percentage as f32 / 100f32;
        let items = self.get_row_items(items, size_threshold_fraction);

        let skin = self.select_skin();

        let mut view_state =
            ViewState::new(items, size_display_format, size_threshold_fraction, &skin);

        // TODO: Push any error into some sort of error stream and expose in UI.
        let _ = view_state.read_config_file();

        #[cfg(not(test))]
        if self.is_interactive() {
            tui::render(
                &mut view_state,
                writer,
                &mut CrosstermInputEventSource::new(),
                &skin,
                self.should_exit.clone(),
            )?;
            writeln!(writer, "Done.")?;
            return Ok(());
        }

        render_rows(view_state, size_threshold_fraction, writer, &skin)?;
        writer.flush()?;

        Ok(())
    }
}

impl ViewCommand {
    pub fn new(
        target_paths: Option<Vec<PathBuf>>,
        size_display_format: Option<SizeDisplayFormat>,
        size_threshold_percentage: u8,
        #[cfg(not(test))] non_interactive: bool,
        env_service: Box<dyn EnvServiceTrait>,
        should_exit: Arc<AtomicBool>,
    ) -> Self {
        ViewCommand {
            target_paths,
            size_display_format,
            size_threshold_percentage,
            #[cfg(not(test))]
            non_interactive,
            total_size_in_bytes: 0,
            env_service,
            should_exit,
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn target_paths(&self) -> &Option<Vec<PathBuf>> {
        &self.target_paths
    }

    #[inline(always)]
    pub fn get_directory_items(&mut self) -> Vec<DirectoryItem> {
        self.analyze_space()
    }

    #[cfg(not(test))]
    fn is_interactive(&self) -> bool {
        use crossterm::tty::IsTty;
        use std::io;
        if cfg!(debug_assertions) {
            !self.non_interactive
        } else {
            !self.non_interactive && io::stdout().is_tty()
        }
    }

    pub(crate) fn get_row_items(
        &mut self,
        dir_items: Vec<DirectoryItem>,
        size_threshold_fraction: f32,
    ) -> Vec<Rc<RefCell<RowItem>>> {
        let mut rows = vec![];
        for dir_item in dir_items {
            if let Some(row_item) = self.add_row_item(dir_item, size_threshold_fraction, &mut rows)
            {
                row_item
                    .borrow_mut()
                    .update_tree_prefix(&String::default(), false);
            }
        }
        rows
    }

    fn add_row_item(
        &self,
        item: DirectoryItem,
        size_threshold_fraction: f32,
        rows: &mut Vec<Rc<RefCell<RowItem>>>,
    ) -> Option<Rc<RefCell<RowItem>>> {
        if item.get_fraction(self.total_size_in_bytes) < size_threshold_fraction {
            return None;
        }
        let value = RowItem::from_directory_item(&item, self.total_size_in_bytes, None, 0);
        rows.push(value.clone());
        Some(value)
    }

    fn analyze_space(&mut self) -> Vec<DirectoryItem> {
        let mut sanitized_paths = vec![];
        if let Some(target_paths) = &self.target_paths {
            for path in target_paths {
                if !sanitized_paths.contains(path) && path.exists() {
                    sanitized_paths.push(path.clone());
                }
            }
            sanitized_paths.sort();
        } else if let Ok(current_dir) = self.env_service.current_dir() {
            sanitized_paths.push(current_dir);
        }

        let items = DirectoryItem::build(sanitized_paths);

        // TODO: Do this inline
        self.total_size_in_bytes = items.iter().map(|t| t.size_in_bytes.get_value()).sum();

        items
    }

    fn select_skin(&self) -> Skin {
        let low_color = Skin {
            title_fg_color: Color::White,
            title_bg_color: Color::Blue,
            version_fg_color: Color::Gray,
            table_header_bg_color: Color::DarkGray,
            table_header_fg_color: Color::White,
            value_fg_color: None,
            value_style_reversed: true,
            delete_warning_text_fg_color: Color::LightRed,
            key_help_danger_bg_color: Color::LightRed,
            key_help_key_fg_color: Color::Gray,
            ..Default::default()
        };

        if let Some(color_count) = self.get_color_count() {
            match color_count {
                ..=256 => low_color,
                _ => Skin::default(),
            }
        } else {
            low_color
        }
    }

    fn get_color_count(&self) -> Option<u32> {
        if let Some(colorterm) = self
            .env_service
            .var(COLORTERM_ENV_VAR)
            .ok()
            .filter(|colorterm| !colorterm.is_empty())
            .or_else(|| {
                self.env_service
                    .var(TERM_ENV_VAR)
                    .ok()
                    .filter(|term| !term.is_empty())
            })
        {
            match colorterm.to_lowercase().as_str() {
                "truecolor" | "24bit" | "24-bit" => Some(16_777_216), // 24-bit color
                "kitty" | "kitty-256color" => Some(256),              // 256 colors
                "konsole" => Some(256),                               // 256 colors
                "rxvt-unicode-256color" => Some(256),                 // 256 colors
                "screen-256color" => Some(256),                       // 256 colors
                "tmux-256color" => Some(256),                         // 256 colors
                "xterm-256color" | "xterm256" => Some(256),           // 256 colors
                "ansi" => Some(16),                                   // 16 colors
                "screen" => Some(16),                                 // 16 colors
                "tmux" => Some(16),                                   // 16 colors
                "xterm" => Some(16),                                  // 16 colors
                "rxvt-unicode" => Some(8),                            // 8 colors (customizable)
                "dumb" => None,                                       // No color support
                "monochrome" => None,                                 // No color support
                _ => None,                                            // Unknown or custom value
            }
        } else {
            None
        }
    }
}

fn render_rows<W: Write>(
    view_state: ViewState,
    size_threshold_fraction: f32,
    writer: &mut W,
    skin: &Skin,
) -> anyhow::Result<usize> {
    let mut rendered_count = 0;
    let backend = CrosstermBackend::new(writer);
    let mut terminal = Terminal::new(backend)?;

    let Rect { width, .. } = terminal.get_frame().size();

    // Exclude the expand column.
    let constraints: Vec<Constraint> = tui::table_column_contraints(width)
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
                terminal.backend_mut(),
                skin,
            )?;
            anyhow::Ok(())
        })
        .context("An error occurred while rendering a row!")?;

    terminal.flush()?;

    Ok(rendered_count)
}

fn render_row<W: Write>(
    item: &Rc<RefCell<RowItem>>,
    size_threshold_fraction: f32,
    constraints: &Vec<Constraint>,
    terminal_width: u16,
    size_display_format: SizeDisplayFormat,
    backend: &mut CrosstermBackend<W>,
    skin: &Skin,
) -> anyhow::Result<usize> {
    let mut rendered_count = 0;
    let item_ref = item.borrow();

    if item_ref.incl_fraction < size_threshold_fraction {
        return Ok(rendered_count);
    }

    let cells =
        view_state::get_row_cell_content(item, size_display_format, terminal_width, true, skin);

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
            )
            .unwrap();
        }
    }

    Ok(rendered_count)
}
