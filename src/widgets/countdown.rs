use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Countdown {
    headline: String,
}

impl Countdown {
    pub const fn new(headline: String) -> Self {
        Self { headline }
    }
}

impl Widget for Countdown {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let h = Paragraph::new(self.headline).centered();
        h.render(area, buf);
    }
}
