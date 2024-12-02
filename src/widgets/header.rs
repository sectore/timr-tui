use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Span,
    widgets::Widget,
};

use crate::utils::format_ms;

#[derive(Debug, Clone)]
pub struct Header {
    tick: u128,
}

impl Header {
    pub fn new(tick: u128) -> Self {
        Self { tick }
    }
}

impl Widget for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let time_string = format_ms(self.tick * 100, true);
        let tick_span = Span::raw(time_string);
        let tick_width = tick_span.width().try_into().unwrap_or(0);
        let [h1, h2] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(tick_width)]).areas(area);

        Span::raw("tim:r").render(h1, buf);
        tick_span.render(h2, buf);
    }
}
