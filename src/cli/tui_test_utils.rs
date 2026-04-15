use crate::cli::input_event_source::InputEventSource;
use crossterm::event::Event;
use std::collections::VecDeque;

pub(crate) struct TestInputEventSource {
    events: VecDeque<Event>,
}

impl TestInputEventSource {
    pub(crate) fn new(events: Vec<Event>) -> Self {
        TestInputEventSource {
            events: VecDeque::from(events),
        }
    }
}

impl InputEventSource for TestInputEventSource {
    fn poll_event(
        &mut self,
        _timeout: std::time::Duration,
    ) -> std::io::Result<Option<crossterm::event::Event>> {
        if !self.events.is_empty() {
            Ok(Some(self.events.pop_front().unwrap()))
        } else {
            Ok(Some(Event::Resize(80, 40)))
        }
    }
}
