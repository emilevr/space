use crossterm::event::Event;
use std::io;

pub trait InputEventSource {
    fn read_event(&mut self) -> io::Result<Event>;
}
