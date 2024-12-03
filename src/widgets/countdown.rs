use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    clock::{self, Clock},
    events::{Event, EventHandler},
};

#[derive(Debug, Clone)]
pub struct Countdown {
    headline: String,
    clock: Clock<clock::Countdown>,
}

impl Countdown {
    pub const fn new(headline: String, clock: Clock<clock::Countdown>) -> Self {
        Self { headline, clock }
    }
}

impl EventHandler for Countdown {
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

pub struct CountdownWidget;

impl StatefulWidget for &CountdownWidget {
    type State = Countdown;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let h = Paragraph::new(state.headline.clone()).centered();
        let c = Paragraph::new(state.clock.format()).centered();
        let [v1, v2] = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

        h.render(v1, buf);
        c.render(v2, buf);
    }
}
