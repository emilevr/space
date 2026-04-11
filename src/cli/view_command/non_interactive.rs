use super::{non_interactive_render, ViewCommand};
use crate::cli::{skin::Skin, view_state::ViewState};
use space_rs::SizeDisplayFormat;
use std::{
    io::Write,
    sync::{atomic::AtomicBool, Arc},
};

impl ViewCommand {
    pub(super) fn run_non_interactive<W: Write>(
        &mut self,
        writer: &mut W,
        size_display_format: SizeDisplayFormat,
        size_threshold_fraction: f32,
        skin: &Skin,
    ) -> anyhow::Result<()> {
        writeln!(
            writer,
            "This could take a while, depending on the size of the tree ...\n\
            Press Ctrl/Cmd+C to cancel (or the appropriate override for your terminal)"
        )?;

        // Enable raw mode so Ctrl+C is delivered as a key event rather than
        // relying on the OS console signal, which is unreliable across Windows
        // terminal environments.
        #[cfg(not(test))]
        crossterm::terminal::enable_raw_mode()?;

        // Spawn a background thread that polls for Ctrl+C.
        let cancel_thread = spawn_cancel_thread(self.should_exit.clone());

        let items = self.get_directory_items();
        let items = self.get_row_items(items, size_threshold_fraction);

        let mut view_state = ViewState::new(
            items,
            size_display_format,
            size_threshold_fraction,
            self.filter_regex.take(),
            skin,
        );

        // TODO: Push any error into some sort of error stream and expose in UI.
        let _ = view_state.read_config_file();

        view_state.apply_regex_filter();

        let filter_regex = view_state.filter_regex.clone();

        non_interactive_render::render_rows(
            view_state,
            size_threshold_fraction,
            writer,
            skin,
            &self.should_exit,
        )?;

        writer.flush()?;

        // Stop the cancel-polling thread.
        cancel_thread.stop();

        #[cfg(not(test))]
        crossterm::terminal::disable_raw_mode()?;

        if self.should_exit.load(std::sync::atomic::Ordering::Relaxed) {
            anyhow::bail!("Cancelled.");
        }

        let mut filter_message = String::new();
        if size_threshold_fraction > 0f32 || filter_regex.is_some() {
            filter_message.push_str("^ Only showing items that");
            if size_threshold_fraction > 0f32 {
                filter_message.push_str(&format!(
                    " are at least {}% of the total size",
                    size_threshold_fraction * 100.0f32,
                ));
            }
            if let Some(ex) = filter_regex {
                if !filter_message.is_empty() {
                    filter_message.push_str(" and");
                }
                filter_message.push_str(&format!(" that match regex \"{}\"", ex));
            }
            writeln!(writer, "{}", filter_message)?;
        }

        writeln!(writer, "Done.")?;

        Ok(())
    }
}

struct CancelThread {
    stop_flag: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl CancelThread {
    fn stop(mut self) {
        self.stop_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn spawn_cancel_thread(should_exit: Arc<AtomicBool>) -> CancelThread {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_polling = stop_flag.clone();
    let handle = std::thread::spawn(move || {
        use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
        use std::time::Duration;
        while !stop_polling.load(std::sync::atomic::Ordering::Relaxed) {
            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                if let Ok(Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                })) = event::read()
                {
                    should_exit.store(true, std::sync::atomic::Ordering::SeqCst);
                    eprintln!("Cancelling...");
                    break;
                }
            }
        }
    });
    CancelThread {
        stop_flag,
        handle: Some(handle),
    }
}
