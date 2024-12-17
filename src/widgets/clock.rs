use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;
use strum::Display;
use tracing::debug;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect, Size},
    symbols,
    widgets::{Block, Borders, StatefulWidget, Widget},
};

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
pub enum Time {
    Seconds,
    Minutes,
    Hours,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Initial,
    Tick,
    Pause,
    Editable(
        Time,
        Box<Mode>, /* previous mode before starting editing */
    ),
    Done,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Initial => write!(f, "[]"),
            Mode::Tick => write!(f, ">"),
            Mode::Pause => write!(f, "||"),
            Mode::Editable(time, _) => match time {
                Time::Seconds => write!(f, "[edit seconds]"),
                Time::Minutes => write!(f, "[edit minutes]"),
                Time::Hours => write!(f, "[edit hours]"),
            },
            Mode::Done => write!(f, "done"),
        }
    }
}

// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#32
const SECS_PER_MINUTE: u64 = 60;
// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#34
const MINS_PER_HOUR: u64 = 60;
// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#36
const HOURS_PER_DAY: u64 = 24;

// max. 99:59:59
const MAX_EDITABLE_DURATION: Duration =
    Duration::from_secs(100 * MINS_PER_HOUR * SECS_PER_MINUTE - 1);

const ONE_SECOND: Duration = Duration::from_secs(1);
const ONE_MINUTE: Duration = Duration::from_secs(SECS_PER_MINUTE);
const ONE_HOUR: Duration = Duration::from_secs(MINS_PER_HOUR * SECS_PER_MINUTE);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display)]
pub enum Format {
    MmSs,
    HhMmSs,
}

#[derive(Debug, Clone)]
pub struct Clock<T> {
    initial_value: Duration,
    tick_value: Duration,
    current_value: Duration,
    mode: Mode,
    format: Format,
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

    pub fn toggle_edit(&mut self) {
        self.mode = match self.mode.clone() {
            Mode::Editable(_, prev) => {
                let p = *prev;
                // special cases: Should `Mode` be updated?
                // 1. `Done` -> `Initial` ?
                if p == Mode::Done && self.current_value.gt(&Duration::ZERO) {
                    Mode::Initial
                }
                // 2. `_` -> `Done` ?
                else if p != Mode::Done && self.current_value.eq(&Duration::ZERO) {
                    Mode::Done
                }
                // 3. `_` -> `_` (no change)
                else {
                    p
                }
            }
            mode => Mode::Editable(Time::Minutes, Box::new(mode)),
        };
    }

    pub fn edit_current_up(&mut self) {
        self.current_value = match self.mode {
            Mode::Editable(Time::Seconds, _) => {
                if self
                    .current_value
                    // < 99:59:58
                    .lt(&MAX_EDITABLE_DURATION.saturating_sub(ONE_SECOND))
                {
                    self.current_value.saturating_add(ONE_SECOND)
                } else {
                    self.current_value
                }
            }
            Mode::Editable(Time::Minutes, _) => {
                if self
                    .current_value
                    // < 99:58:59
                    .lt(&MAX_EDITABLE_DURATION.saturating_sub(ONE_MINUTE))
                {
                    self.current_value.saturating_add(ONE_MINUTE)
                } else {
                    self.current_value
                }
            }
            Mode::Editable(Time::Hours, _) => {
                if self
                    .current_value
                    // < 98:59:59
                    .lt(&MAX_EDITABLE_DURATION.saturating_sub(ONE_HOUR))
                {
                    self.current_value.saturating_add(ONE_HOUR)
                } else {
                    self.current_value
                }
            }
            _ => self.current_value,
        };
    }
    pub fn edit_current_down(&mut self) {
        self.current_value = match self.mode {
            Mode::Editable(Time::Seconds, _) => self.current_value.saturating_sub(ONE_SECOND),
            Mode::Editable(Time::Minutes, _) => self.current_value.saturating_sub(ONE_MINUTE),
            Mode::Editable(Time::Hours, _) => self.current_value.saturating_sub(ONE_HOUR),
            _ => self.current_value,
        };
        self.update_mode();
    }

    pub fn get_mode(&mut self) -> &Mode {
        &self.mode
    }

    pub fn is_edit_mode(&mut self) -> bool {
        matches!(self.mode, Mode::Editable(_, _))
    }

    pub fn edit_mode(&mut self) -> Option<Time> {
        match self.mode {
            Mode::Editable(time, _) => Some(time),
            _ => None,
        }
    }

    fn edit_mode_next(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Seconds, prev) if self.format == Format::MmSs => {
                Mode::Editable(Time::Minutes, prev)
            }
            Mode::Editable(Time::Minutes, prev) if self.format == Format::MmSs => {
                Mode::Editable(Time::Seconds, prev)
            }
            Mode::Editable(Time::Minutes, prev) if self.format == Format::HhMmSs => {
                Mode::Editable(Time::Hours, prev)
            }
            Mode::Editable(Time::Hours, prev) => Mode::Editable(Time::Seconds, prev),
            _ => mode,
        }
    }

    fn edit_mode_prev(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Seconds, prev) if self.format == Format::HhMmSs => {
                Mode::Editable(Time::Hours, prev)
            }
            Mode::Editable(Time::Seconds, prev) if self.format == Format::MmSs => {
                Mode::Editable(Time::Minutes, prev)
            }
            Mode::Editable(Time::Minutes, prev) => Mode::Editable(Time::Seconds, prev),
            Mode::Editable(Time::Hours, prev) => Mode::Editable(Time::Minutes, prev),
            _ => mode,
        }
    }

    fn update_mode(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Hours, prev) if self.format == Format::HhMmSs => {
                Mode::Editable(Time::Minutes, prev)
            }
            _ => mode,
        }
    }

    pub fn reset(&mut self) {
        self.mode = Mode::Initial;
        self.current_value = self.initial_value;
    }

    fn current_hours(&self) -> u64 {
        self.current_seconds() / (SECS_PER_MINUTE * MINS_PER_HOUR)
    }

    fn current_hours_mod(&self) -> u64 {
        self.current_hours() % HOURS_PER_DAY
    }

    fn current_minutes(&self) -> u64 {
        self.current_seconds() / MINS_PER_HOUR
    }

    fn current_minutes_mod(&self) -> u64 {
        self.current_minutes() % SECS_PER_MINUTE
    }

    fn current_seconds(&self) -> u64 {
        self.current_value.as_secs()
    }

    fn current_seconds_mod(&self) -> u64 {
        self.current_seconds() % SECS_PER_MINUTE
    }

    fn current_tenths(&self) -> u32 {
        self.current_value.subsec_millis() / 100
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
            self.current_hours_mod(),
            self.current_minutes_mod(),
            self.current_seconds_mod(),
            self.current_tenths()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Countdown {}

impl Clock<Countdown> {
    pub fn new(initial_value: Duration, tick_value: Duration) -> Self {
        Self {
            initial_value,
            tick_value,
            current_value: initial_value,
            mode: Mode::Initial,
            format: Format::MmSs,
            phantom: PhantomData,
        }
    }

    pub fn tick(&mut self) {
        if self.mode == Mode::Tick {
            self.current_value = self.current_value.saturating_sub(self.tick_value);
            self.check_done();
            self.update_format();
        }
    }

    pub fn check_done(&mut self) {
        if self.current_value.is_zero() {
            self.mode = Mode::Done;
        }
    }

    pub fn edit_next(&mut self) {
        self.edit_mode_next();
        self.update_format();
    }

    pub fn edit_prev(&mut self) {
        self.edit_mode_prev();
        self.update_format();
    }

    pub fn edit_up(&mut self) {
        self.edit_current_up();
        self.update_format();
        // update `initial_value` if needed
        if self.initial_value.lt(&self.current_value) {
            self.initial_value = self.current_value;
        }
    }

    pub fn edit_down(&mut self) {
        self.edit_current_down();
        self.update_format();
        // update `initial_value` if needed
        if self.initial_value.gt(&self.current_value) {
            self.initial_value = self.current_value;
        }
    }

    fn update_format(&mut self) {
        self.format = self.get_format();
    }

    pub fn get_format(&self) -> Format {
        // show hours if initial available only
        if self.current_value.ge(&ONE_HOUR) {
            Format::HhMmSs
        } else {
            Format::MmSs
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timer {}

impl Clock<Timer> {
    pub fn new(initial_value: Duration, tick_value: Duration) -> Self {
        Self {
            initial_value,
            tick_value,
            current_value: Duration::ZERO,
            mode: Mode::Initial,
            format: Format::MmSs,
            phantom: PhantomData,
        }
    }

    pub fn tick(&mut self) {
        if self.mode == Mode::Tick {
            self.current_value = self.current_value.saturating_add(self.tick_value);
            self.check_done();
            self.update_format();
        }
    }

    fn check_done(&mut self) {
        if self.current_value == self.initial_value {
            self.mode = Mode::Done;
        }
    }

    fn update_format(&mut self) {
        self.format = self.get_format();
    }

    pub fn get_format(&self) -> Format {
        if self.current_value.ge(&ONE_HOUR) {
            Format::HhMmSs
        } else {
            Format::MmSs
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

    fn get_horizontal_lengths(&self, format: &Format) -> Vec<u16> {
        match format {
            Format::MmSs => vec![11, 4, 11],
            Format::HhMmSs => vec![11, 4, 11, 4, 11],
        }
    }

    pub fn get_width(&self, format: &Format) -> u16 {
        self.get_horizontal_lengths(format).iter().sum()
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

    fn render_edit_border(mode: &Mode, format: &Format, width: u16, area: Rect, buf: &mut Buffer) {
        let border_area = match mode {
            Mode::Editable(Time::Seconds, _) => {
                let [_, h] =
                    Layout::horizontal([Constraint::Percentage(100), Constraint::Length(width)])
                        .areas(area);
                h
            }
            Mode::Editable(Time::Minutes, _) if format == &Format::MmSs => {
                let [h, _] =
                    Layout::horizontal([Constraint::Length(width), Constraint::Percentage(100)])
                        .areas(area);
                h
            }
            Mode::Editable(Time::Minutes, _) if format == &Format::HhMmSs => {
                let [_, h, _] = Layout::horizontal([
                    Constraint::Fill(0),
                    Constraint::Length(width),
                    Constraint::Fill(0),
                ])
                .areas(area);
                h
            }
            Mode::Editable(Time::Hours, _) => {
                let [h, _] = Layout::horizontal([Constraint::Length(width), Constraint::Fill(0)])
                    .areas(area);

                h
            }
            _ => area,
        };

        let border_type = match mode {
            Mode::Editable(_, _) => symbols::border::THICK,
            _ => symbols::border::EMPTY,
        };

        Block::new()
            .borders(Borders::TOP)
            .border_set(border_type)
            .render(border_area, buf);
    }
}

impl<T> StatefulWidget for ClockWidget<T>
where
    T: std::fmt::Debug,
{
    type State = Clock<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [v1, v2] = Layout::vertical(Constraint::from_lengths([
            self.get_digit_height(),
            EDIT_BORDER_HEIGHT as u16,
        ]))
        .areas(area);

        debug!("clock render {:?} {:?}", state.format, area.width);

        match &state.format {
            Format::MmSs => {
                let [h1, h2, h3] = Layout::horizontal(Constraint::from_lengths(
                    self.get_horizontal_lengths(&state.format),
                ))
                .areas(v1);

                let size_digits = Self::render_digit_pair(state.current_minutes_mod(), h1, buf);
                Self::render_colon(h2, buf);
                Self::render_digit_pair(state.current_seconds_mod(), h3, buf);

                Self::render_edit_border(
                    &state.mode,
                    &state.format,
                    size_digits.width - 1,
                    v2,
                    buf,
                );
            }
            Format::HhMmSs => {
                let [h1, h2, h3, h4, h5] = Layout::horizontal(Constraint::from_lengths(
                    self.get_horizontal_lengths(&state.format),
                ))
                .areas(v1);

                let size_digits = Self::render_digit_pair(state.current_hours_mod(), h1, buf);
                Self::render_colon(h2, buf);
                Self::render_digit_pair(state.current_minutes_mod(), h3, buf);
                Self::render_colon(h4, buf);
                Self::render_digit_pair(state.current_seconds_mod(), h5, buf);

                Self::render_edit_border(
                    &state.mode,
                    &state.format,
                    size_digits.width - 1,
                    v2,
                    buf,
                );
            }
        }
    }
}
