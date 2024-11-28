use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Timer<'a> {
    value: u64,
    headline: Text<'a>,
}

impl<'a> Timer<'a> {
    pub const fn new(value: u64, headline: Text<'a>) -> Self {
        Self { value, headline }
    }
}

impl Widget for Timer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let h = Paragraph::new(self.headline).centered();
        h.render(area, buf);
    }
}
