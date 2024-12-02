use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pomodoro {
    headline: String,
}

impl Pomodoro {
    pub const fn new(headline: String) -> Self {
        Self { headline }
    }
}

impl Widget for Pomodoro {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let h = Paragraph::new(self.headline).centered();
        h.render(area, buf);
    }
}
