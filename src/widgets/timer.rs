use crate::{
    events::{Event, EventHandler},
    utils::center,
    widgets::clock::{self, Clock, ClockWidget},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use std::cmp::max;

#[derive(Debug, Clone)]
pub struct Timer {
    clock: Clock<clock::Timer>,
}

impl Timer {
    pub const fn new(clock: Clock<clock::Timer>) -> Self {
        Self { clock }
    }
}

impl EventHandler for Timer {
    fn update(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::Tick => {
                self.clock.tick();
            }
            Event::Key(key) if key.code == KeyCode::Char('s') => {
                self.clock.toggle_pause();
            }
            Event::Key(key) if key.code == KeyCode::Char('r') => {
                self.clock.reset();
            }
            _ => return Some(event),
        }
        None
    }
}

pub struct TimerWidget;

impl StatefulWidget for &TimerWidget {
    type State = Timer;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock = ClockWidget::new();
        let headline = "Timer".to_uppercase();
        let label = Line::raw((format!("{} {}", headline, state.clock.get_mode())).to_uppercase());

        let area = center(
            area,
            Constraint::Length(max(
                clock.get_width(&state.clock.get_format()),
                label.width() as u16,
            )),
            Constraint::Length(clock.get_height() + 1 /* height of label */),
        );
        let [v1, v2] =
            Layout::vertical(Constraint::from_lengths([clock.get_height(), 1])).areas(area);

        clock.render(v1, buf, &mut state.clock);
        label.centered().render(v2, buf);
    }
}
