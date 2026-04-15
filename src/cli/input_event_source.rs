use crossterm::event::Event;
use std::{io, time::Duration};

pub trait InputEventSource {
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>>;
}
