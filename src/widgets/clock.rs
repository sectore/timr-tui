use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;
use strum::Display;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect, Size},
    symbols,
    widgets::{Block, Borders, StatefulWidget, Widget},
};

use crate::utils::center_horizontal;

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
pub enum Editable {
    None,
    Seconds,
    Minutes,
    // ignoring hours for now
    // Hours,
}

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
pub enum Mode {
    Initial,
    Tick,
    Pause(Editable),
    Done,
}

#[derive(Debug, Clone, Copy)]
pub struct Clock<T> {
    initial_value: Duration,
    tick_value: Duration,
    current_value: Duration,
    mode: Mode,
    phantom: PhantomData<T>,
}

const MAX_EDITABLE_DURATION: Duration = Duration::from_secs(60 * 60); // 1 hour

impl<T> Clock<T> {
    pub fn toggle_pause(&mut self) {
        self.mode = if self.mode == Mode::Tick {
            Mode::Pause(Editable::None)
        } else {
            Mode::Tick
        }
    }

    pub fn toggle_edit(&mut self) {
        if self.is_edit_mode() {
            self.mode = Mode::Pause(Editable::None)
        } else {
            self.mode = Mode::Pause(Editable::Minutes);
        }
    }

    pub fn edit_up(&mut self) {
        self.current_value = match self.mode {
            Mode::Pause(Editable::Seconds) => {
                if self
                    .current_value
                    // < 59:59
                    .lt(&MAX_EDITABLE_DURATION.saturating_sub(Duration::from_secs(1)))
                {
                    self.current_value.saturating_add(Duration::from_secs(1))
                } else {
                    self.current_value
                }
            }
            Mode::Pause(Editable::Minutes) => {
                if self
                    .current_value
                    // < 59:00
                    .lt(&MAX_EDITABLE_DURATION.saturating_sub(Duration::from_secs(60)))
                {
                    self.current_value.saturating_add(Duration::new(60, 0))
                } else {
                    self.current_value
                }
            }
            _ => self.current_value,
        };

        // update initial_value
        self.initial_value = self.current_value;
    }
    pub fn edit_down(&mut self) {
        self.current_value = match self.mode {
            Mode::Pause(Editable::Seconds) => {
                self.current_value.saturating_sub(Duration::new(1, 0))
            }
            Mode::Pause(Editable::Minutes) => {
                self.current_value.saturating_sub(Duration::new(60, 0))
            }
            _ => self.current_value,
        };
    }

    pub fn non_edit(&mut self) {
        self.mode = Mode::Pause(Editable::None);
    }

    pub fn get_mode(&mut self) -> Mode {
        self.mode
    }

    pub fn is_edit_mode(&mut self) -> bool {
        matches!(
            self.mode,
            Mode::Pause(Editable::Seconds) | Mode::Pause(Editable::Minutes)
        )
    }

    pub fn edit_mode(&mut self) -> Option<Editable> {
        match self.mode {
            Mode::Pause(Editable::None) => None,
            Mode::Pause(mode) => Some(mode),
            _ => None,
        }
    }

    pub fn edit_next(&mut self) {
        self.mode = Mode::Pause(match self.mode {
            Mode::Pause(Editable::Seconds) => Editable::Minutes,
            Mode::Pause(Editable::Minutes) => Editable::Seconds,
            _ => Editable::None,
        });
    }

    pub fn edit_prev(&mut self) {
        self.mode = Mode::Pause(match self.mode {
            Mode::Pause(Editable::Seconds) => Editable::Minutes,
            Mode::Pause(Editable::Minutes) => Editable::Seconds,
            _ => Editable::None,
        });
    }

    pub fn reset(&mut self) {
        self.mode = Mode::Initial;
        self.current_value = self.initial_value;
    }

    fn duration(&self) -> Duration {
        self.current_value
    }

    fn hours(&self) -> u64 {
        (self.duration().as_secs() / 60 / 60) % 60
    }

    fn minutes(&self) -> u64 {
        (self.duration().as_secs() / 60) % 60
    }

    fn seconds(&self) -> u64 {
        self.duration().as_secs() % 60
    }

    fn tenths(&self) -> u32 {
        self.duration().subsec_millis() / 100
    }

    pub fn is_done(&mut self) -> bool {
        self.mode == Mode::Done
    }
}

impl<T> fmt::Display for Clock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{}",
            self.hours(),
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
    pub fn new(initial_value: Duration, tick_value: Duration) -> Self {
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

    pub fn check_done(&mut self) {
        if self.current_value.is_zero() {
            self.mode = Mode::Done;
        }
    }
}
impl Clock<Timer> {
    pub fn new(initial_value: Duration, tick_value: Duration) -> Self {
        Self {
            initial_value,
            tick_value,
            current_value: Duration::ZERO,
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
const EDIT_BORDER_HEIGHT: usize = 1;

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

pub struct ClockWidget<T>
where
    T: std::fmt::Debug,
{
    phantom: PhantomData<T>,
}

impl<T> ClockWidget<T>
where
    T: std::fmt::Debug,
{
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

    pub fn get_digit_height(&self) -> u16 {
        DIGIT_SIZE as u16
    }

    pub fn get_height(&self) -> u16 {
        self.get_digit_height() + (EDIT_BORDER_HEIGHT as u16)
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

    fn render_digit_pair(d: u64, area: Rect, buf: &mut Buffer) -> Size {
        let widths = [DIGIT_SIZE as u16, 2, DIGIT_SIZE as u16];
        let h = Layout::horizontal(Constraint::from_lengths(widths)).split(area);
        Self::render_number(d / 10, h[0], buf);
        Self::render_number(d % 10, h[2], buf);

        Size::new(widths.iter().sum(), area.height)
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

    fn render_edit_border(mode: Mode, width: u16, area: Rect, buf: &mut Buffer) {
        match mode {
            Mode::Pause(Editable::Seconds) => {
                let [_, h2] = Layout::horizontal([Constraint::Fill(0), Constraint::Length(width)])
                    .areas(area);
                Block::new()
                    .borders(Borders::TOP)
                    .border_set(symbols::border::THICK)
                    .render(h2, buf);
            }
            Mode::Pause(Editable::Minutes) => {
                let [h1, _] = Layout::horizontal([Constraint::Length(width), Constraint::Fill(0)])
                    .areas(area);

                Block::new()
                    .borders(Borders::TOP)
                    .border_set(symbols::border::THICK)
                    .render(h1, buf)
            }
            _ => Block::new()
                .borders(Borders::TOP)
                .border_set(symbols::border::EMPTY)
                .render(area, buf),
        }
        // Span::raw(format!("{:?} {}", mode, s)).render(area, buf);
    }
}

impl<T> StatefulWidget for ClockWidget<T>
where
    T: std::fmt::Debug,
{
    type State = Clock<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // center
        let h = center_horizontal(area, Constraint::Length(self.get_width()));

        let [v1, v2] = Layout::vertical(Constraint::from_lengths([
            self.get_digit_height(),
            EDIT_BORDER_HEIGHT as u16,
        ]))
        .areas(h);

        let [h1, h2, h3] =
            Layout::horizontal(Constraint::from_lengths(self.get_horizontal_lengths())).areas(v1);

        let size_digits = Self::render_digit_pair(state.minutes(), h1, buf);
        Self::render_colon(h2, buf);
        Self::render_digit_pair(state.seconds(), h3, buf);

        Self::render_edit_border(state.mode, size_digits.width - 1, v2, buf);
    }
}
