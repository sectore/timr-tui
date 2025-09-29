use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    widgets::Widget,
};

pub const DIGIT_SIZE: usize = 5;
pub const DIGIT_WIDTH: u16 = DIGIT_SIZE as u16;
pub const DIGIT_HEIGHT: u16 = DIGIT_SIZE as u16 + 1 /* border height */;
pub const TWO_DIGITS_WIDTH: u16 = DIGIT_WIDTH + DIGIT_SPACE_WIDTH + DIGIT_WIDTH; // digit-space-digit
pub const THREE_DIGITS_WIDTH: u16 =
    DIGIT_WIDTH + DIGIT_SPACE_WIDTH + DIGIT_WIDTH + DIGIT_SPACE_WIDTH + DIGIT_WIDTH; // digit-space-digit-space-digit
pub const COLON_WIDTH: u16 = 4; // incl. padding left + padding right
pub const DOT_WIDTH: u16 = 4; // incl. padding left + padding right
pub const DIGIT_SPACE_WIDTH: u16 = 1; // space between digits
pub const DIGIT_LABEL_WIDTH: u16 = 3; // label (single char) incl. padding left + padding right

#[rustfmt::skip]
const DIGIT_0: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    1, 1, 0, 1, 1,
    1, 1, 0, 1, 1,
    1, 1, 0, 1, 1,
    1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const DIGIT_1: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    0, 0, 0, 1, 1,
    0, 0, 0, 1, 1,
    0, 0, 0, 1, 1,
    0, 0, 0, 1, 1,
    0, 0, 0, 1, 1,
];

#[rustfmt::skip]
const DIGIT_2: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    0, 0, 0, 1, 1,
    1, 1, 1, 1, 1,
    1, 1, 0, 0, 0,
    1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const DIGIT_3: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    0, 0, 0, 1, 1,
    1, 1, 1, 1, 1,
    0, 0, 0, 1, 1,
    1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const DIGIT_4: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 0, 1, 1,
    1, 1, 0, 1, 1,
    1, 1, 1, 1, 1,
    0, 0, 0, 1, 1,
    0, 0, 0, 1, 1,
];

#[rustfmt::skip]
const DIGIT_5: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    1, 1, 0, 0, 0,
    1, 1, 1, 1, 1,
    0, 0, 0, 1, 1,
    1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const DIGIT_6: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    1, 1, 0, 0, 0,
    1, 1, 1, 1, 1,
    1, 1, 0, 1, 1,
    1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const DIGIT_7: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    0, 0, 0, 1, 1,
    0, 0, 0, 1, 1,
    0, 0, 0, 1, 1,
    0, 0, 0, 1, 1,
];

#[rustfmt::skip]
const DIGIT_8: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    1, 1, 0, 1, 1,
    1, 1, 1, 1, 1,
    1, 1, 0, 1, 1,
    1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const DIGIT_9: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    1, 1, 0, 1, 1,
    1, 1, 1, 1, 1,
    0, 0, 0, 1, 1,
    1, 1, 1, 1, 1,
];

#[rustfmt::skip]
const CHAR_E: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    1, 1, 0, 0, 0,
    1, 1, 1, 1, 0,
    1, 1, 0, 0, 0,
    1, 1, 1, 1, 1,
];

pub struct Digit<'a> {
    digit: u64,
    with_border: bool,
    symbol: &'a str,
}

impl<'a> Digit<'a> {
    pub fn new(digit: u64, with_border: bool, symbol: &'a str) -> Self {
        Self {
            digit,
            with_border,
            symbol,
        }
    }
}

impl Widget for Digit<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let left = area.left();
        let top = area.top();

        let patterns = match self.digit {
            0 => DIGIT_0,
            1 => DIGIT_1,
            2 => DIGIT_2,
            3 => DIGIT_3,
            4 => DIGIT_4,
            5 => DIGIT_5,
            6 => DIGIT_6,
            7 => DIGIT_7,
            8 => DIGIT_8,
            9 => DIGIT_9,
            _ => CHAR_E,
        };

        patterns.iter().enumerate().for_each(|(i, item)| {
            let x = i % DIGIT_SIZE;
            let y = i / DIGIT_SIZE;
            if *item == 1 {
                let p = Position {
                    x: left + x as u16,
                    y: top + y as u16,
                };
                if let Some(cell) = buf.cell_mut(p) {
                    cell.set_symbol(self.symbol);
                }
            }
        });

        // Add border at the bottom
        if self.with_border {
            for x in 0..area.width {
                let p = Position {
                    x: left + x,
                    y: top + area.height - 1,
                };
                if let Some(cell) = buf.cell_mut(p) {
                    cell.set_symbol("─");
                }
            }
        }
    }
}

pub struct Dot<'a> {
    symbol: &'a str,
}

impl<'a> Dot<'a> {
    pub fn new(symbol: &'a str) -> Self {
        Self { symbol }
    }
}

impl Widget for Dot<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let positions = [
            Position {
                x: area.left() + 1,
                y: area.top() + area.height - 2,
            },
            Position {
                x: area.left() + 2,
                y: area.top() + area.height - 2,
            },
        ];

        for pos in positions {
            if let Some(cell) = buf.cell_mut(pos) {
                cell.set_symbol(self.symbol);
            }
        }
    }
}

pub struct Colon<'a> {
    symbol: &'a str,
}

impl<'a> Colon<'a> {
    pub fn new(symbol: &'a str) -> Self {
        Self { symbol }
    }
}

impl Widget for Colon<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let left = area.left();
        let top = area.top();

        let positions = [
            Position {
                x: left + 1,
                y: top + 1,
            },
            Position {
                x: left + 2,
                y: top + 1,
            },
            Position {
                x: left + 1,
                y: top + 3,
            },
            Position {
                x: left + 2,
                y: top + 3,
            },
        ];

        for pos in positions {
            if let Some(cell) = buf.cell_mut(pos) {
                cell.set_symbol(self.symbol);
            }
        }
    }
}
