mod non_interactive;
pub(crate) mod non_interactive_render;
mod skin_selection;

#[cfg(not(test))]
mod interactive;

use super::{cli_command::CliCommand, environment::EnvServiceTrait, row_item::RowItem};
use space_rs::{DirectoryItem, SizeDisplayFormat};
use std::{
    cell::RefCell,
    io::Write,
    path::PathBuf,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc},
};

#[cfg(test)]
#[path = "../view_command_test.rs"]
mod view_command_test;

pub(crate) const COLORTERM_ENV_VAR: &str = "COLORTERM";
pub(crate) const TERM_ENV_VAR: &str = "TERM";

pub(crate) struct ViewCommand {
    target_paths: Option<Vec<PathBuf>>,
    size_display_format: Option<SizeDisplayFormat>,
    size_threshold_percentage: u8,
    #[cfg(not(test))]
    non_interactive: bool,
    filter_regex: Option<regex::Regex>,
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
                    anyhow::bail!("{} does not exist!", target_path.display());
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

        let size_display_format = match &self.size_display_format {
            Some(size_display_format) => *size_display_format,
            _ => SizeDisplayFormat::Metric,
        };

        let size_threshold_fraction = self.size_threshold_percentage as f32 / 100f32;
        let skin = self.select_skin();

        #[cfg(not(test))]
        if self.is_interactive() {
            return self.run_interactive(
                writer,
                size_display_format,
                size_threshold_fraction,
                &skin,
            );
        }

        self.run_non_interactive(writer, size_display_format, size_threshold_fraction, &skin)
    }
}

impl ViewCommand {
    pub fn new(
        target_paths: Option<Vec<PathBuf>>,
        size_display_format: Option<SizeDisplayFormat>,
        size_threshold_percentage: u8,
        #[cfg(not(test))] non_interactive: bool,
        filter_regex: Option<regex::Regex>,
        env_service: Box<dyn EnvServiceTrait>,
        should_exit: Arc<AtomicBool>,
    ) -> Self {
        ViewCommand {
            target_paths,
            size_display_format,
            size_threshold_percentage,
            #[cfg(not(test))]
            non_interactive,
            filter_regex,
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
        // from_directory_item creates all nodes collapsed; expand the full tree
        // for the non-scanning (pre-built) view.
        Self::expand_tree(&value);
        rows.push(value.clone());
        Some(value)
    }

    /// Recursively expands all directory nodes in the tree.
    fn expand_tree(item: &Rc<RefCell<RowItem>>) {
        let mut item_ref = item.borrow_mut();
        if item_ref.has_children {
            item_ref.expanded = true;
            for child in &item_ref.children {
                Self::expand_tree(child);
            }
        }
    }

    fn get_sanitized_paths(&self) -> Vec<PathBuf> {
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
        sanitized_paths
    }

    fn analyze_space(&mut self) -> Vec<DirectoryItem> {
        let sanitized_paths = self.get_sanitized_paths();

        let items = DirectoryItem::build(sanitized_paths, &self.should_exit);

        // TODO: Do this inline
        self.total_size_in_bytes = items.iter().map(|t| t.size_in_bytes.get_value()).sum();

        items
    }
}
