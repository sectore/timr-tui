use crate::widgets::clock_elements::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

const D_RECT: Rect = Rect::new(0, 0, DIGIT_WIDTH, DIGIT_HEIGHT);

#[test]
fn test_d1() {
    let mut b = Buffer::empty(D_RECT);
    Digit::new(1, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "   ██",
        "   ██",
        "   ██",
        "   ██",
        "   ██",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(1, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "   ██",
        "   ██",
        "   ██",
        "   ██",
        "   ██",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_d2() {
    let mut b = Buffer::empty(D_RECT);
    Digit::new(2, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "   ██",
        "█████",
        "██   ",
        "█████",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(2, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "   ██",
        "█████",
        "██   ",
        "█████",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_dot() {
    let mut b = Buffer::empty(D_RECT);
    Dot::new("█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "     ",
        "     ",
        "     ",
        "     ",
        " ██  ",
        "     ",
    ]);
    assert_eq!(b, expected);
}

#[test]
fn test_colon() {
    let mut b = Buffer::empty(D_RECT);
    Colon::new("█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "     ",
        " ██  ",
        "     ",
        " ██  ",
        "     ",
        "     ",
    ]);
    assert_eq!(b, expected);
}
