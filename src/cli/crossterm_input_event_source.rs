use super::input_event_source::InputEventSource;
use crossterm::event::{self, Event};
use std::{io, time::Duration};

pub struct CrosstermInputEventSource {}

impl CrosstermInputEventSource {
    pub fn new() -> Self {
        CrosstermInputEventSource {}
    }
}

impl InputEventSource for CrosstermInputEventSource {
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        if event::poll(timeout)? {
            event::read().map(Some)
        } else {
            Ok(None)
        }
    }
}
