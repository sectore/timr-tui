use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;
use strum::Display;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    widgets::StatefulWidget,
};

use crate::{args::ClockStyle, utils::center_horizontal};

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
const MAX_DURATION: Duration =
    Duration::from_secs(100 * MINS_PER_HOUR * SECS_PER_MINUTE).saturating_sub(ONE_SECOND);

const ONE_SECOND: Duration = Duration::from_secs(1);
const ONE_MINUTE: Duration = Duration::from_secs(SECS_PER_MINUTE);
const ONE_HOUR: Duration = Duration::from_secs(MINS_PER_HOUR * SECS_PER_MINUTE);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display, PartialOrd, Ord)]
pub enum Format {
    S,
    Ss,
    MSs,
    MmSs,
    HMmSs,
    HhMmSs,
}

#[derive(Debug, Clone)]
pub struct Clock<T> {
    initial_value: Duration,
    tick_value: Duration,
    current_value: Duration,
    mode: Mode,
    format: Format,
    pub style: ClockStyle,
    pub with_decis: bool,
    phantom: PhantomData<T>,
}

pub struct ClockArgs {
    pub initial_value: Duration,
    pub tick_value: Duration,
    pub style: ClockStyle,
    pub with_decis: bool,
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
            mode => {
                if self.format <= Format::Ss {
                    Mode::Editable(Time::Seconds, Box::new(mode))
                } else {
                    Mode::Editable(Time::Minutes, Box::new(mode))
                }
            }
        };
    }

    pub fn edit_current_up(&mut self) {
        self.current_value = match self.mode {
            Mode::Editable(Time::Seconds, _) => {
                if self
                    .current_value
                    // < 99:59:58
                    .le(&MAX_DURATION.saturating_sub(ONE_SECOND))
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
                    .le(&MAX_DURATION.saturating_sub(ONE_MINUTE))
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
                    .lt(&MAX_DURATION.saturating_sub(ONE_HOUR))
                {
                    self.current_value.saturating_add(ONE_HOUR)
                } else {
                    self.current_value
                }
            }
            _ => self.current_value,
        };
        self.update_format();
    }
    pub fn edit_current_down(&mut self) {
        self.current_value = match self.mode {
            Mode::Editable(Time::Seconds, _) => self.current_value.saturating_sub(ONE_SECOND),
            Mode::Editable(Time::Minutes, _) => self.current_value.saturating_sub(ONE_MINUTE),
            Mode::Editable(Time::Hours, _) => self.current_value.saturating_sub(ONE_HOUR),
            _ => self.current_value,
        };
        self.update_format();
        self.update_mode();
    }

    pub fn get_mode(&mut self) -> &Mode {
        &self.mode
    }

    pub fn is_edit_mode(&mut self) -> bool {
        matches!(self.mode, Mode::Editable(_, _))
    }

    fn edit_mode_next(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Seconds, prev) if self.format >= Format::MSs => {
                Mode::Editable(Time::Minutes, prev)
            }
            Mode::Editable(Time::Minutes, prev) if self.format <= Format::MmSs => {
                Mode::Editable(Time::Seconds, prev)
            }
            Mode::Editable(Time::Minutes, prev) if self.format >= Format::MmSs => {
                Mode::Editable(Time::Hours, prev)
            }
            Mode::Editable(Time::Hours, prev) => Mode::Editable(Time::Seconds, prev),
            _ => mode,
        };
        self.update_format();
    }

    fn edit_mode_prev(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Seconds, prev) if self.format >= Format::HMmSs => {
                Mode::Editable(Time::Hours, prev)
            }
            Mode::Editable(Time::Seconds, prev) if self.format <= Format::MmSs => {
                Mode::Editable(Time::Minutes, prev)
            }
            Mode::Editable(Time::Minutes, prev) => Mode::Editable(Time::Seconds, prev),
            Mode::Editable(Time::Hours, prev) => Mode::Editable(Time::Minutes, prev),
            _ => mode,
        };
        self.update_format();
    }

    fn update_mode(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Hours, prev) if self.format <= Format::MmSs => {
                Mode::Editable(Time::Minutes, prev)
            }
            Mode::Editable(Time::Minutes, prev) if self.format <= Format::Ss => {
                Mode::Editable(Time::Seconds, prev)
            }
            _ => mode,
        }
    }

    pub fn reset(&mut self) {
        self.mode = Mode::Initial;
        self.current_value = self.initial_value;
        self.update_format();
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

    // deciseconds
    fn current_decis(&self) -> u64 {
        (self.current_value.subsec_millis() / 100) as u64
    }

    pub fn is_done(&mut self) -> bool {
        self.mode == Mode::Done
    }

    fn update_format(&mut self) {
        self.format = self.get_format();
    }

    pub fn get_format(&self) -> Format {
        if self.current_hours() >= 10 {
            Format::HhMmSs
        } else if self.current_hours() >= 1 {
            Format::HMmSs
        } else if self.current_minutes() >= 10 {
            Format::MmSs
        } else if self.current_minutes() >= 1 {
            Format::MSs
        } else if self.current_seconds() >= 10 {
            Format::Ss
        } else {
            Format::S
        }
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
            self.current_decis()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Countdown {}

impl Clock<Countdown> {
    pub fn new(args: ClockArgs) -> Self {
        let ClockArgs {
            initial_value,
            tick_value,
            style,
            with_decis,
        } = args;
        let mut instance = Self {
            initial_value,
            tick_value,
            current_value: initial_value,
            mode: Mode::Initial,
            format: Format::S,
            style,
            with_decis,
            phantom: PhantomData,
        };
        // update format once
        instance.update_format();
        instance
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
    }

    pub fn edit_prev(&mut self) {
        self.edit_mode_prev();
    }

    pub fn edit_up(&mut self) {
        self.edit_current_up();
        // update `initial_value` if needed
        if self.initial_value.lt(&self.current_value) {
            self.initial_value = self.current_value;
        }
    }

    pub fn edit_down(&mut self) {
        self.edit_current_down();
        // update `initial_value` if needed
        if self.initial_value.gt(&self.current_value) {
            self.initial_value = self.current_value;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timer {}

impl Clock<Timer> {
    pub fn new(args: ClockArgs) -> Self {
        let ClockArgs {
            initial_value,
            tick_value,
            style,
            with_decis,
        } = args;
        let mut instance = Self {
            initial_value,
            tick_value,
            current_value: Duration::ZERO,
            mode: Mode::Initial,
            format: Format::S,
            phantom: PhantomData,
            style,
            with_decis,
        };
        // update format once
        instance.update_format();
        instance
    }

    pub fn tick(&mut self) {
        if self.mode == Mode::Tick {
            self.current_value = self.current_value.saturating_add(self.tick_value);
            self.check_done();
            self.update_format();
        }
    }

    fn check_done(&mut self) {
        if self.current_value >= MAX_DURATION {
            self.mode = Mode::Done;
        }
    }

    pub fn edit_next(&mut self) {
        self.edit_mode_next();
    }

    pub fn edit_prev(&mut self) {
        self.edit_mode_prev();
    }

    pub fn edit_up(&mut self) {
        self.edit_current_up();
    }

    pub fn edit_down(&mut self) {
        self.edit_current_down();
    }
}

const DIGIT_SIZE: usize = 5;
const DIGIT_WIDTH: u16 = DIGIT_SIZE as u16;
const DIGIT_HEIGHT: u16 = DIGIT_SIZE as u16 + 1 /* border height */;
const COLON_WIDTH: u16 = 4; // incl. padding left + padding right
const SPACE_WIDTH: u16 = 1;

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

    fn get_digit_symbol(&self, style: &ClockStyle) -> &str {
        match &style {
            ClockStyle::Bold => "█",
            ClockStyle::Empty => "░",
            ClockStyle::Cross => "╬",
            ClockStyle::Thick => "┃",
        }
    }

    fn get_horizontal_lengths(&self, format: &Format, with_decis: bool) -> Vec<u16> {
        let add_decis = |mut lengths: Vec<u16>, with_decis: bool| -> Vec<u16> {
            if with_decis {
                lengths.extend_from_slice(&[
                    COLON_WIDTH, // .
                    DIGIT_WIDTH, // ds
                ])
            }
            lengths
        };

        match format {
            Format::HhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH, // h
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // h
                    COLON_WIDTH, // :
                    DIGIT_WIDTH, // m
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // m
                    COLON_WIDTH, // :
                    DIGIT_WIDTH, // s
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // s
                ],
                with_decis,
            ),
            Format::HMmSs => add_decis(
                vec![
                    DIGIT_WIDTH, // h
                    COLON_WIDTH, // :
                    DIGIT_WIDTH, // m
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // m
                    COLON_WIDTH, // :
                    DIGIT_WIDTH, // s
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // s
                ],
                with_decis,
            ),
            Format::MmSs => add_decis(
                vec![
                    DIGIT_WIDTH, // m
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // m
                    COLON_WIDTH, // :
                    DIGIT_WIDTH, // s
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // s
                ],
                with_decis,
            ),
            Format::MSs => add_decis(
                vec![
                    DIGIT_WIDTH, // m
                    COLON_WIDTH, // :
                    DIGIT_WIDTH, // s
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // s
                ],
                with_decis,
            ),
            Format::Ss => add_decis(
                vec![
                    DIGIT_WIDTH, // s
                    SPACE_WIDTH, // (space)
                    DIGIT_WIDTH, // s
                ],
                with_decis,
            ),
            Format::S => add_decis(
                vec![
                    DIGIT_WIDTH, // s
                ],
                with_decis,
            ),
        }
    }

    pub fn get_width(&self, format: &Format, with_decis: bool) -> u16 {
        self.get_horizontal_lengths(format, with_decis).iter().sum()
    }

    pub fn get_height(&self) -> u16 {
        DIGIT_HEIGHT
    }

    fn render_digit(
        &self,
        number: u64,
        symbol: &str,
        with_border: bool,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let left = area.left();
        let top = area.top();

        let symbols = match number {
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

        symbols.iter().enumerate().for_each(|(i, item)| {
            let x = i % DIGIT_SIZE;
            let y = i / DIGIT_SIZE;
            if *item == 1 {
                let p = Position {
                    x: left + x as u16,
                    y: top + y as u16,
                };
                if let Some(cell) = buf.cell_mut(p) {
                    cell.set_symbol(symbol);
                }
            }
        });

        // Add border at the bottom
        if with_border {
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

    fn render_colon(&self, symbol: &str, area: Rect, buf: &mut Buffer) {
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
                cell.set_symbol(symbol);
            }
        }
    }

    fn render_dot(&self, symbol: &str, area: Rect, buf: &mut Buffer) {
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
                cell.set_symbol(symbol);
            }
        }
    }
}

impl<T> StatefulWidget for ClockWidget<T>
where
    T: std::fmt::Debug,
{
    type State = Clock<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let with_decis = state.with_decis;
        let format = state.format;
        let symbol = self.get_digit_symbol(&state.style);
        let widths = self.get_horizontal_lengths(&format, with_decis);
        let area = center_horizontal(
            area,
            Constraint::Length(self.get_width(&format, with_decis)),
        );
        let edit_hours = matches!(state.mode, Mode::Editable(Time::Hours, _));
        let edit_minutes = matches!(state.mode, Mode::Editable(Time::Minutes, _));
        let edit_secs = matches!(state.mode, Mode::Editable(Time::Seconds, _));
        match format {
            Format::HhMmSs if with_decis => {
                let [hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(state.current_hours() / 10, symbol, edit_hours, hh, buf);
                self.render_digit(state.current_hours() % 10, symbol, edit_hours, h, buf);
                self.render_colon(symbol, c_hm, buf);
                self.render_digit(
                    state.current_minutes_mod() / 10,
                    symbol,
                    edit_minutes,
                    mm,
                    buf,
                );
                self.render_digit(
                    state.current_minutes_mod() % 10,
                    symbol,
                    edit_minutes,
                    m,
                    buf,
                );
                self.render_colon(symbol, c_ms, buf);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
                self.render_dot(symbol, d, buf);
                self.render_digit(state.current_decis(), symbol, false, ds, buf);
            }
            Format::HhMmSs => {
                let [hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(state.current_hours() / 10, symbol, edit_hours, hh, buf);
                self.render_digit(state.current_hours() % 10, symbol, edit_hours, h, buf);
                self.render_colon(symbol, c_hm, buf);
                self.render_digit(
                    state.current_minutes_mod() / 10,
                    symbol,
                    edit_minutes,
                    mm,
                    buf,
                );
                self.render_digit(
                    state.current_minutes_mod() % 10,
                    symbol,
                    edit_minutes,
                    m,
                    buf,
                );
                self.render_colon(symbol, c_ms, buf);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
            }
            Format::HMmSs if with_decis => {
                let [h, c_hm, mm, _, m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(state.current_hours() % 10, symbol, edit_hours, h, buf);
                self.render_colon(symbol, c_hm, buf);
                self.render_digit(
                    state.current_minutes_mod() / 10,
                    symbol,
                    edit_minutes,
                    mm,
                    buf,
                );
                self.render_digit(
                    state.current_minutes_mod() % 10,
                    symbol,
                    edit_minutes,
                    m,
                    buf,
                );
                self.render_colon(symbol, c_ms, buf);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
                self.render_dot(symbol, d, buf);
                self.render_digit(state.current_decis(), symbol, false, ds, buf);
            }
            Format::HMmSs => {
                let [h, c_hm, mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(state.current_hours() % 10, symbol, edit_hours, h, buf);
                self.render_colon(symbol, c_hm, buf);
                self.render_digit(
                    state.current_minutes_mod() / 10,
                    symbol,
                    edit_minutes,
                    mm,
                    buf,
                );
                self.render_digit(
                    state.current_minutes_mod() % 10,
                    symbol,
                    edit_minutes,
                    m,
                    buf,
                );
                self.render_colon(symbol, c_ms, buf);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
            }
            Format::MmSs if with_decis => {
                let [mm, _, m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(
                    state.current_minutes_mod() / 10,
                    symbol,
                    edit_minutes,
                    mm,
                    buf,
                );
                self.render_digit(
                    state.current_minutes_mod() % 10,
                    symbol,
                    edit_minutes,
                    m,
                    buf,
                );
                self.render_colon(symbol, c_ms, buf);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
                self.render_dot(symbol, d, buf);
                self.render_digit(state.current_decis(), symbol, false, ds, buf);
            }
            Format::MmSs => {
                let [mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(
                    state.current_minutes_mod() / 10,
                    symbol,
                    edit_minutes,
                    mm,
                    buf,
                );
                self.render_digit(
                    state.current_minutes_mod() % 10,
                    symbol,
                    edit_minutes,
                    m,
                    buf,
                );
                self.render_colon(symbol, c_ms, buf);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
            }
            Format::MSs if with_decis => {
                let [m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(
                    state.current_minutes_mod() % 10,
                    symbol,
                    edit_minutes,
                    m,
                    buf,
                );
                self.render_colon(symbol, c_ms, buf);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
                self.render_dot(symbol, d, buf);
                self.render_digit(state.current_decis(), symbol, false, ds, buf);
            }
            Format::MSs => {
                let [m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(
                    state.current_minutes_mod() % 10,
                    symbol,
                    edit_minutes,
                    m,
                    buf,
                );
                self.render_colon(symbol, c_ms, buf);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
            }
            Format::Ss if state.with_decis => {
                let [ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
                self.render_dot(symbol, d, buf);
                self.render_digit(state.current_decis(), symbol, false, ds, buf);
            }
            Format::Ss => {
                let [ss, _, s] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(state.current_seconds_mod() / 10, symbol, edit_secs, ss, buf);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
            }
            Format::S if with_decis => {
                let [s, d, ds] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
                self.render_dot(symbol, d, buf);
                self.render_digit(state.current_decis(), symbol, false, ds, buf);
            }
            Format::S => {
                let [s] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                self.render_digit(state.current_seconds_mod() % 10, symbol, edit_secs, s, buf);
            }
        }
    }
}
