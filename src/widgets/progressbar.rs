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
        let label = Span::raw(format!(" {}% ", self.percentage));
        let [h0, area] = Layout::horizontal([
            Constraint::Length(label.width() as u16),
            Constraint::Percentage(100),
        ])
        .areas(area);
        let [h1, h2] =
            Layout::horizontal([Constraint::Percentage(self.percentage), Constraint::Fill(0)])
                .areas(area);

        // label
        label.render(h0, buf);
        // done
        Span::from(line::THICK_HORIZONTAL.repeat(h1.width as usize)).render(h1, buf);
        // rest
        Span::from(line::HORIZONTAL.repeat(h2.width as usize)).render(h2, buf);
    }
}
