use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::widgets::header::Header;

const W: u16 = 10;
const RECT: Rect = Rect::new(0, 0, W, 1);

#[test]
fn test_header_none() {
    let mut b = Buffer::empty(RECT);
    Header { percentage: None }.render(RECT, &mut b);
    assert_eq!(b, Buffer::with_lines(["──────────"]));
}

#[test]
fn test_header_progress() {
    let mut b = Buffer::empty(RECT);
    Header { percentage: Some(50) }.render(RECT, &mut b);
    assert_eq!(b, Buffer::with_lines(["━━━━━─────"]));
}

#[test]
fn test_header_progress_full() {
    let mut b = Buffer::empty(RECT);
    Header { percentage: Some(100) }.render(RECT, &mut b);
    assert_eq!(b, Buffer::with_lines(["━━━━━━━━━━"]));
}
