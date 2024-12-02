use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Timer {
    value: u64,
    headline: String,
}

impl Timer {
    pub const fn new(value: u64, headline: String) -> Self {
        Self { value, headline }
    }
}

impl Widget for Timer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let h = Paragraph::new(self.headline).centered();
        h.render(area, buf);
    }
}
