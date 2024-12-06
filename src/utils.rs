use ratatui::layout::{Constraint, Flex, Layout, Rect};

/// Helper to center an area horizontally by given `Constraint`
/// based on [Center a Rect](https://ratatui.rs/recipes/layout/center-a-rect)
pub fn center_horizontal(base_area: Rect, horizontal: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(base_area);
    area
}

/// Helper to center an area vertically by given `Constraint`
/// based on [Center a Rect](https://ratatui.rs/recipes/layout/center-a-rect)
pub fn center_vertical(base_area: Rect, vertical: Constraint) -> Rect {
    let [area] = Layout::vertical([vertical])
        .flex(Flex::Center)
        .areas(base_area);
    area
}
/// Helper to center an area by given `Constraint`'s
/// based on [Center a Rect](https://ratatui.rs/recipes/layout/center-a-rect)
pub fn center(base_area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let area = center_horizontal(base_area, horizontal);
    center_vertical(area, vertical)
}

#[cfg(test)]
mod tests {

    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect, text::Span, widgets::Widget};

    #[test]
    fn test_center() {
        let l = Span::raw("hello!");
        let mut b = Buffer::empty(Rect::new(0, 0, 10, 3));
        let area = center(
            b.area,
            Constraint::Length(l.width() as u16),
            Constraint::Length(1),
        );
        l.render(area, &mut b);
        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "          ",
            "  hello!  ",
            "          ",
        ]);
        assert_eq!(b, expected);
    }

    #[test]
    fn test_center_horizontal() {
        let l = Span::raw("hello!");
        let mut b = Buffer::empty(Rect::new(0, 0, 10, 1));
        let area = center_horizontal(b.area, Constraint::Length(l.width() as u16));
        l.render(area, &mut b);
        let expected = Buffer::with_lines(["  hello!  "]);
        assert_eq!(b, expected);
    }

    #[test]
    fn test_center_vertical() {
        let l = Span::raw("hello vertical");
        let mut b = Buffer::empty(Rect::new(0, 0, 20, 3));
        let area = center_vertical(b.area, Constraint::Length(1));
        l.render(area, &mut b);
        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "                    ",
            "hello vertical      ",
            "                    ",
        ]);
        assert_eq!(b, expected);
    }
}
