use crate::widgets::clock_elements::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

const D_RECT: Rect = Rect::new(0, 0, DIGIT_WIDTH, DIGIT_HEIGHT);

fn b() -> Buffer {
    Buffer::empty(D_RECT)
}

#[test]
fn test_d0() {
    let mut b = b();
    Digit::new(0, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██ ██",
        "██ ██",
        "██ ██",
        "█████",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(0, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██ ██",
        "██ ██",
        "██ ██",
        "█████",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_d1() {
    let mut b = b();
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
    let mut b = b();
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
fn test_d3() {
    let mut b = b();
    Digit::new(3, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "   ██",
        "█████",
        "   ██",
        "█████",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(3, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "   ██",
        "█████",
        "   ██",
        "█████",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_d4() {
    let mut b = b();
    Digit::new(4, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "██ ██",
        "██ ██",
        "█████",
        "   ██",
        "   ██",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(4, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "██ ██",
        "██ ██",
        "█████",
        "   ██",
        "   ██",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_d5() {
    let mut b = b();
    Digit::new(5, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██   ",
        "█████",
        "   ██",
        "█████",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(5, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██   ",
        "█████",
        "   ██",
        "█████",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_d6() {
    let mut b = b();
    Digit::new(6, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██   ",
        "█████",
        "██ ██",
        "█████",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(6, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██   ",
        "█████",
        "██ ██",
        "█████",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_d7() {
    let mut b = b();
    Digit::new(7, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "   ██",
        "   ██",
        "   ██",
        "   ██",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(7, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "   ██",
        "   ██",
        "   ██",
        "   ██",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_d8() {
    let mut b = b();
    Digit::new(8, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██ ██",
        "█████",
        "██ ██",
        "█████",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(8, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██ ██",
        "█████",
        "██ ██",
        "█████",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_d9() {
    let mut b = b();
    Digit::new(9, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██ ██",
        "█████",
        "   ██",
        "█████",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(9, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██ ██",
        "█████",
        "   ██",
        "█████",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_char_e() {
    let mut b = b();
    Digit::new(10, false, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██   ",
        "████ ",
        "██   ",
        "█████",
        "     ",
    ]);
    assert_eq!(b, expected, "w/o border");

    Digit::new(10, true, "█").render(D_RECT, &mut b);
    #[rustfmt::skip]
    let expected = Buffer::with_lines([
        "█████",
        "██   ",
        "████ ",
        "██   ",
        "█████",
        "─────",
    ]);
    assert_eq!(b, expected, "w/ border");
}

#[test]
fn test_dot() {
    let mut b = b();
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
    let mut b = b();
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
