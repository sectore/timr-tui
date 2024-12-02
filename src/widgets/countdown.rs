use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};

use crate::clock::{self, Clock};

#[derive(Debug)]
pub struct Countdown {
    headline: String,
    clock: Clock<clock::Countdown>,
}

impl Countdown {
    pub const fn new(headline: String, clock: Clock<clock::Countdown>) -> Self {
        Self { headline, clock }
    }
}

impl Widget for Countdown {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let h = Paragraph::new(self.headline).centered();
        let c = Paragraph::new(self.clock.format()).centered();
        let [v1, v2] = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

        h.render(v1, buf);
        c.render(v2, buf)
    }
}
