use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;
use strum::Display;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    widgets::StatefulWidget,
};

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
pub enum Mode {
    Initial,
    Tick,
    Pause,
    Done,
}

#[derive(Debug, Clone, Copy)]
pub struct Clock<T> {
    initial_value: u64,
    tick_value: u64,
    current_value: u64,
    mode: Mode,
    phantom: PhantomData<T>,
}

impl<T> Clock<T> {
    pub fn toggle_pause(&mut self) {
        self.mode = if self.mode == Mode::Tick {
            Mode::Pause
        } else {
            Mode::Tick
        }
    }

    pub fn reset(&mut self) {
        self.mode = Mode::Initial;
        self.current_value = self.initial_value;
    }

    fn duration(&self) -> Duration {
        Duration::from_millis(self.current_value)
    }

    fn minutes(&self) -> u64 {
        self.duration().as_secs() / 60
    }
    fn seconds(&self) -> u64 {
        self.duration().as_secs() % 60
    }
    fn tenths(&self) -> u32 {
        self.duration().subsec_millis() / 100
    }
}

impl<T> fmt::Display for Clock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}.{}",
            self.minutes(),
            self.seconds(),
            self.tenths()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Countdown {}

#[derive(Debug, Clone)]
pub struct Timer {}

impl Clock<Countdown> {
    pub fn new(initial_value: u64, tick_value: u64) -> Self {
        Self {
            initial_value,
            tick_value,
            current_value: initial_value,
            mode: Mode::Initial,
            phantom: PhantomData,
        }
    }

    pub fn tick(&mut self) {
        if self.mode == Mode::Tick {
            self.current_value = self.current_value.saturating_sub(self.tick_value);
            self.check_done();
        }
    }

    fn check_done(&mut self) {
        if self.current_value == 0 {
            self.mode = Mode::Done;
        }
    }
}
impl Clock<Timer> {
    pub fn new(initial_value: u64, tick_value: u64) -> Self {
        Self {
            initial_value,
            tick_value,
            current_value: 0,
            mode: Mode::Initial,
            phantom: PhantomData,
        }
    }

    pub fn tick(&mut self) {
        if self.mode == Mode::Tick {
            self.current_value = self.current_value.saturating_add(self.tick_value);
            self.check_done();
        }
    }

    fn check_done(&mut self) {
        if self.current_value == self.initial_value {
            self.mode = Mode::Done;
        }
    }
}

const DIGIT_SYMBOL: &str = "█";

const DIGIT_SIZE: usize = 5;

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
const DIGIT_ERROR: [u8; DIGIT_SIZE * DIGIT_SIZE] = [
    1, 1, 1, 1, 1,
    1, 1, 0, 0, 0,
    1, 1, 1, 1, 0,
    1, 1, 0, 0, 0,
    1, 1, 1, 1, 1,
];

pub struct ClockWidget<T> {
    phantom: PhantomData<T>,
}

impl<T> ClockWidget<T> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn get_horizontal_lengths(&self) -> [u16; 3] {
        [11, 4, 11]
    }

    pub fn get_width(&self) -> u16 {
        self.get_horizontal_lengths().iter().sum()
    }

    pub fn get_height(&self) -> u16 {
        DIGIT_SIZE as u16
    }

    fn render_number(number: u64, area: Rect, buf: &mut Buffer) {
        let left = area.left();
        let top = area.top();

        let digits = match number {
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
            _ => DIGIT_ERROR,
        };

        digits.iter().enumerate().for_each(|(i, item)| {
            let x = i % DIGIT_SIZE;
            let y = i / DIGIT_SIZE;
            if *item == 1 {
                let p = Position {
                    x: left + x as u16,
                    y: top + y as u16,
                };
                if let Some(cell) = buf.cell_mut(p) {
                    cell.set_symbol(DIGIT_SYMBOL);
                }
            }
        });
    }

    fn render_digit_pair(d: u64, area: Rect, buf: &mut Buffer) {
        let h = Layout::new(
            Direction::Horizontal,
            Constraint::from_lengths([DIGIT_SIZE as u16, 2, DIGIT_SIZE as u16]),
        )
        .split(area);
        Self::render_number(d / 10, h[0], buf);
        Self::render_number(d % 10, h[2], buf);
    }

    fn render_colon(area: Rect, buf: &mut Buffer) {
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
                cell.set_symbol(DIGIT_SYMBOL);
            }
        }
    }
}

impl<T> StatefulWidget for ClockWidget<T> {
    type State = Clock<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // center
        let [_, h, _] = Layout::horizontal([
            Constraint::Fill(0),
            Constraint::Length(self.get_width()),
            Constraint::Fill(0),
        ])
        .areas(area);

        let [h1, h2, h3] = Layout::new(
            Direction::Horizontal,
            Constraint::from_lengths(self.get_horizontal_lengths()),
        )
        .areas(h);

        Self::render_digit_pair(state.minutes(), h1, buf);
        Self::render_colon(h2, buf);
        Self::render_digit_pair(state.seconds(), h3, buf);
    }
}
