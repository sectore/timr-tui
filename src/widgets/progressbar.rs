use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::line,
    text::Span,
    widgets::Widget,
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
        let [h0, h1] =
            Layout::horizontal([Constraint::Percentage(self.percentage), Constraint::Fill(0)])
                .areas(area);

        // done
        Span::from(line::THICK_HORIZONTAL.repeat(h0.width as usize)).render(h0, buf);
        // rest
        Span::from(line::HORIZONTAL.repeat(h1.width as usize)).render(h1, buf);
    }
}
