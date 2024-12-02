use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Span,
    widgets::Widget,
};

#[derive(Debug, Clone)]
pub struct Header {
    show_fps: bool,
}

impl Header {
    pub fn new(show_fps: bool) -> Self {
        Self { show_fps }
    }
}

impl Widget for Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let fps_txt = if self.show_fps { "FPS (soon)" } else { "" };
        let fps_span = Span::raw(fps_txt);
        let fps_width = fps_span.width().try_into().unwrap_or(0);
        let [h1, h2] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(fps_width)]).areas(area);

        Span::raw("tim:r").render(h1, buf);
        fps_span.render(h2, buf);
    }
}
