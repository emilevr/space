use super::input_event_source::InputEventSource;
use crossterm::event::{self, Event};
use std::io;

pub struct CrosstermInputEventSource {}

impl CrosstermInputEventSource {
    pub fn new() -> Self {
        CrosstermInputEventSource {}
    }
}

impl InputEventSource for CrosstermInputEventSource {
    fn read_event(&mut self) -> io::Result<Event> {
        event::read()
    }
}
