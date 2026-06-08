use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::line,
    widgets::{Fill, Widget},
};

#[derive(Debug, Clone)]
pub struct Progressbar {
    pub percentage: u16,
}

impl Progressbar {
    pub fn new(percentage: u16) -> Self {
        Self { percentage }
    }
}

impl Widget for Progressbar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [h1, h2] =
            Layout::horizontal([Constraint::Percentage(self.percentage), Constraint::Fill(0)])
                .areas(area);
        Fill::new(line::THICK_HORIZONTAL).render(h1, buf);
        Fill::new(line::HORIZONTAL).render(h2, buf);
    }
}
