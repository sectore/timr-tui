use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::Widget,
};

use crate::utils::center;

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
        let headline = Line::raw(self.headline.clone());

        let area = center(
            area,
            Constraint::Length(headline.width() as u16),
            Constraint::Length(3),
        );

        let [v1, _, v2] = Layout::vertical(Constraint::from_lengths([1, 1, 1])).areas(area);

        headline.render(v2, buf);
        Line::raw("SOON").centered().italic().render(v1, buf);
    }
}
