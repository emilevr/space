mod dialogs;
mod help;
pub(crate) mod key_handlers;
pub(crate) mod rendering;
mod scan_drain;

use super::{input_event_source::InputEventSource, scan_worker, skin::Skin, view_state::ViewState};
#[cfg(not(test))]
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
#[cfg(not(test))]
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
};
use ratatui::{prelude::*, Terminal};
use std::{
    io::Write,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

pub(crate) use rendering::table_column_constraints;

#[cfg(test)]
#[path = "scan_test.rs"]
mod scan_test;

#[cfg(test)]
#[path = "scan_drain_edge_test.rs"]
mod scan_drain_edge_test;

#[cfg(test)]
#[path = "scan_render_loop_test.rs"]
mod scan_render_loop_test;

#[cfg(test)]
#[path = "render_test.rs"]
mod render_test;

#[cfg(test)]
#[path = "nav_test.rs"]
mod nav_test;

#[cfg(test)]
#[path = "filter_test.rs"]
mod filter_test;

#[cfg(test)]
#[path = "dialog_test.rs"]
mod dialog_test;

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
pub(crate) const RESCAN_KEY: char = 'r';
pub(crate) const ACCEPT_LICENSE_TERMS_KEY: char = 'a';
pub(crate) const FILTER_KEY: char = '/';

pub(crate) const QUIT_KEY_2_SYMBOL: &str = "Esc";
pub(crate) const SELECT_PREV_KEY_SYMBOL: char = '↑';
pub(crate) const SELECT_NEXT_KEY_SYMBOL: char = '↓';
pub(crate) const COLLAPSE_KEY_SYMBOL: char = '←';
pub(crate) const EXPAND_KEY_SYMBOL: char = '→';
pub(crate) const COLLAPSE_CHILDREN_KEY_SYMBOL: char = '-';
pub(crate) const EXPAND_CHILDREN_KEY_SYMBOL: char = '+';
pub(crate) const SELECT_PREV_PAGE_KEY_SYMBOL: &str = "Page Up";
pub(crate) const SELECT_NEXT_PAGE_KEY_SYMBOL: &str = "Page Down";
pub(crate) const SELECT_FIRST_KEY_SYMBOL: &str = "Home";
pub(crate) const SELECT_LAST_KEY_SYMBOL: &str = "End";

pub(crate) fn render<W: Write, I: InputEventSource>(
    view_state: &mut ViewState,
    writer: &mut W,
    input_event_source: &mut I,
    skin: &Skin,
    should_exit: Arc<AtomicBool>,
    scan_sender: scan_worker::ScanSender,
    scan_receiver: scan_worker::ScanReceiver,
) -> anyhow::Result<()> {
    #[cfg(not(test))]
    enable_raw_mode()?;

    execute!(writer, enter_terminal_command(), EnableMouseCapture)?;
    let backend = CrosstermBackend::new(writer);

    #[cfg(not(test))]
    let mut terminal = Terminal::new(backend)?;
    #[cfg(test)]
    let mut terminal = {
        let area = Rect::new(0, 0, 120, 24);
        Terminal::with_options(
            backend,
            ratatui::TerminalOptions {
                viewport: ratatui::Viewport::Fixed(area),
            },
        )?
    };

    let result = render_loop(
        &mut terminal,
        view_state,
        input_event_source,
        skin,
        should_exit,
        scan_sender,
        scan_receiver,
    );

    #[cfg(not(test))]
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

const POLL_TIMEOUT: Duration = Duration::from_millis(50);

/// Maximum time to spend draining scan messages per frame.  Prevents the drain
/// loop from starving the render loop when scan threads produce faster than
/// the UI can render.
const DRAIN_BUDGET: Duration = Duration::from_millis(100);

fn render_loop<B: Backend, I: InputEventSource>(
    terminal: &mut Terminal<B>,
    view_state: &mut ViewState,
    input_event_source: &mut I,
    skin: &Skin,
    should_exit: Arc<AtomicBool>,
    scan_sender: scan_worker::ScanSender,
    scan_receiver: scan_worker::ScanReceiver,
) -> anyhow::Result<()> {
    // Track how many scan/rescan threads are outstanding.  Each sends
    // Complete when done; final cleanup runs when the count reaches 0.
    let mut active_scan_count: usize = if view_state.is_scanning { 1 } else { 0 };

    loop {
        view_state.spinner_tick = view_state.spinner_tick.wrapping_add(1);
        if view_state.is_scanning {
            view_state.derive_scanning_state();
        }
        terminal.draw(|f| rendering::create_frame(f, view_state, skin))?;

        if should_exit.load(std::sync::atomic::Ordering::Relaxed) {
            anyhow::bail!("Cancelled.");
        }

        // Check for async deletion completion before processing key events,
        // so the dialog closes before the next key is handled.
        view_state.check_deletion_complete();

        let event = input_event_source.poll_event(POLL_TIMEOUT)?;

        if let Some(Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        })) = event
        {
            if code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL {
                anyhow::bail!("Cancelled.");
            }

            if key_handlers::handle_key_input(view_state, code) {
                return Ok(());
            }
        }

        // Process any rescan requests from key handlers or deletion recovery.
        if let Some((path, ancestor_segments)) = view_state.rescan_request.take() {
            active_scan_count += 1;
            scan_worker::spawn_rescan(
                ancestor_segments,
                path,
                should_exit.clone(),
                scan_sender.clone(),
            );
        }

        if active_scan_count > 0 {
            let deadline = Instant::now() + DRAIN_BUDGET;
            let completed = scan_drain::drain_scan_channel(
                &scan_receiver,
                view_state,
                deadline,
                &mut active_scan_count,
            );
            if !completed {
                view_state.try_auto_expand_to_fill();
            }
        }
    }
}
