use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Widget},
};

use crate::widgets::progressbar::Progressbar;

#[derive(Debug, Clone)]
pub struct Header {
    pub percentage: Option<u16>,
}

impl Widget for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(percentage) = self.percentage {
            Progressbar::new(percentage).render(area, buf);
        } else {
            Block::new().borders(Borders::TOP).render(area, buf);
        }
    }
}
