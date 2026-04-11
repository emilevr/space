use super::ViewCommand;
use crate::cli::{
    crossterm_input_event_source::CrosstermInputEventSource, scan_worker, skin::Skin, tui,
    view_state::ViewState,
};
use space_rs::SizeDisplayFormat;
use std::io::Write;

impl ViewCommand {
    pub(super) fn run_interactive<W: Write>(
        &self,
        writer: &mut W,
        size_display_format: SizeDisplayFormat,
        size_threshold_fraction: f32,
        skin: &Skin,
    ) -> anyhow::Result<()> {
        let paths = self.get_sanitized_paths();
        let (scan_sender, scan_receiver) = crossfire::mpsc::unbounded_blocking();
        scan_worker::spawn_scan(paths, self.should_exit.clone(), scan_sender.clone());

        let mut view_state = ViewState::new(
            vec![],
            size_display_format,
            size_threshold_fraction,
            self.filter_regex.clone(),
            skin,
        );
        view_state.is_scanning = true;

        // TODO: Push any error into some sort of error stream and expose in UI.
        let _ = view_state.read_config_file();

        tui::render(
            &mut view_state,
            writer,
            &mut CrosstermInputEventSource::new(),
            skin,
            self.should_exit.clone(),
            scan_sender,
            scan_receiver,
        )?;

        writeln!(writer, "Done.")?;
        writer.flush()?;

        Ok(())
    }
}
