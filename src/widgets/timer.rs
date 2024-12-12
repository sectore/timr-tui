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
    headline: String,
    clock: Clock<clock::Timer>,
}

impl Timer {
    pub const fn new(headline: String, clock: Clock<clock::Timer>) -> Self {
        Self { headline, clock }
    }
}

impl EventHandler for Timer {
    fn update(&mut self, event: Event) {
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
            _ => {}
        }
    }
}

pub struct TimerWidget;

impl StatefulWidget for &TimerWidget {
    type State = Timer;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock = ClockWidget::new();
        let headline = Line::raw(state.headline.to_uppercase());

        let area = center(
            area,
            Constraint::Length(max(clock.get_width(), headline.width() as u16)),
            Constraint::Length(clock.get_height() + 1 /* height of headline */),
        );
        let [v1, v2] =
            Layout::vertical(Constraint::from_lengths([clock.get_height(), 1])).areas(area);

        clock.render(v1, buf, &mut state.clock);
        headline.render(v2, buf);
    }
}
