use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Countdown<'a> {
    headline: Text<'a>,
}

impl<'a> Countdown<'a> {
    pub const fn new(headline: Text<'a>) -> Self {
        Self { headline }
    }
}

impl Widget for Countdown<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let h = Paragraph::new(self.headline).centered();
        h.render(area, buf);
    }
}
