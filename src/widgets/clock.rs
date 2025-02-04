use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;
use strum::Display;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    common::Style,
    duration::{DurationEx, MAX_DURATION, ONE_DECI_SECOND, ONE_HOUR, ONE_MINUTE, ONE_SECOND},
    events::{AppEvent, AppEventTx},
    utils::center_horizontal,
    widgets::clock_elements::{
        Colon, Digit, Dot, COLON_WIDTH, DIGIT_HEIGHT, DIGIT_SPACE_WIDTH, DIGIT_WIDTH, DOT_WIDTH,
    },
};

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
pub enum Time {
    Decis,
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
                Time::Decis => write!(f, "[edit deciseconds]"),
                Time::Seconds => write!(f, "[edit seconds]"),
                Time::Minutes => write!(f, "[edit minutes]"),
                Time::Hours => write!(f, "[edit hours]"),
            },
            Mode::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display, PartialOrd, Ord)]
pub enum Format {
    S,
    Ss,
    MSs,
    MmSs,
    HMmSs,
    HhMmSs,
}

pub struct ClockState<T> {
    initial_value: DurationEx,
    current_value: DurationEx,
    tick_value: DurationEx,
    mode: Mode,
    format: Format,
    pub with_decis: bool,
    on_done: Option<Box<dyn Fn() + 'static>>,
    app_tx: Option<AppEventTx>,
    phantom: PhantomData<T>,
}

pub struct ClockStateArgs {
    pub initial_value: Duration,
    pub current_value: Duration,
    pub tick_value: Duration,
    pub with_decis: bool,
    pub app_tx: Option<AppEventTx>,
}

impl<T> ClockState<T> {
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    pub fn is_initial(&self) -> bool {
        self.mode == Mode::Initial
    }

    pub fn run(&mut self) {
        self.mode = Mode::Tick
    }

    pub fn is_running(&self) -> bool {
        self.mode == Mode::Tick
    }

    pub fn toggle_pause(&mut self) {
        self.mode = if self.mode == Mode::Tick {
            Mode::Pause
        } else {
            Mode::Tick
        }
    }

    pub fn get_initial_value(&self) -> &DurationEx {
        &self.initial_value
    }

    pub fn get_current_value(&self) -> &DurationEx {
        &self.current_value
    }

    pub fn set_current_value(&mut self, duration: DurationEx) {
        self.current_value = duration;
        self.update_format();
    }

    pub fn toggle_edit(&mut self) {
        self.mode = match self.mode.clone() {
            Mode::Editable(_, prev) => {
                let p = *prev;
                // special cases: Should `Mode` be updated?
                // 1. `Done` -> `Initial` ?
                if p == Mode::Done && self.current_value.gt(&Duration::ZERO.into()) {
                    Mode::Initial
                }
                // 2. `_` -> `Done` ?
                else if p != Mode::Done && self.current_value.eq(&Duration::ZERO.into()) {
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
            Mode::Editable(Time::Decis, _) => {
                if self
                    .current_value
                    // < 99:59:58
                    .le(&MAX_DURATION.saturating_sub(ONE_DECI_SECOND).into())
                {
                    self.current_value.saturating_add(ONE_DECI_SECOND.into())
                } else {
                    self.current_value
                }
            }
            Mode::Editable(Time::Seconds, _) => {
                if self
                    .current_value
                    // < 99:59:58
                    .le(&MAX_DURATION.saturating_sub(ONE_SECOND).into())
                {
                    self.current_value.saturating_add(ONE_SECOND.into())
                } else {
                    self.current_value
                }
            }
            Mode::Editable(Time::Minutes, _) => {
                if self
                    .current_value
                    // < 99:58:59
                    .le(&MAX_DURATION.saturating_sub(ONE_MINUTE).into())
                {
                    self.current_value.saturating_add(ONE_MINUTE.into())
                } else {
                    self.current_value
                }
            }
            Mode::Editable(Time::Hours, _) => {
                if self
                    .current_value
                    // < 98:59:59
                    .lt(&MAX_DURATION.saturating_sub(ONE_HOUR).into())
                {
                    self.current_value.saturating_add(ONE_HOUR.into())
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
            Mode::Editable(Time::Decis, _) => {
                self.current_value.saturating_sub(ONE_DECI_SECOND.into())
            }
            Mode::Editable(Time::Seconds, _) => {
                self.current_value.saturating_sub(ONE_SECOND.into())
            }
            Mode::Editable(Time::Minutes, _) => {
                self.current_value.saturating_sub(ONE_MINUTE.into())
            }
            Mode::Editable(Time::Hours, _) => self.current_value.saturating_sub(ONE_HOUR.into()),
            _ => self.current_value,
        };
        self.update_format();
        self.update_mode();
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, Mode::Editable(_, _))
    }

    fn edit_mode_next(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Decis, prev) => Mode::Editable(Time::Seconds, prev),
            Mode::Editable(Time::Seconds, prev) if self.format <= Format::Ss && self.with_decis => {
                Mode::Editable(Time::Decis, prev)
            }
            Mode::Editable(Time::Seconds, prev) if self.format <= Format::Ss => {
                Mode::Editable(Time::Seconds, prev)
            }
            Mode::Editable(Time::Seconds, prev) => Mode::Editable(Time::Minutes, prev),
            Mode::Editable(Time::Minutes, prev)
                if self.format <= Format::MmSs && self.with_decis =>
            {
                Mode::Editable(Time::Decis, prev)
            }
            Mode::Editable(Time::Minutes, prev) if self.format <= Format::MmSs => {
                Mode::Editable(Time::Seconds, prev)
            }
            Mode::Editable(Time::Minutes, prev) => Mode::Editable(Time::Hours, prev),
            Mode::Editable(Time::Hours, prev) if self.with_decis => {
                Mode::Editable(Time::Decis, prev)
            }
            Mode::Editable(Time::Hours, prev) => Mode::Editable(Time::Seconds, prev),
            _ => mode,
        };
        self.update_format();
    }

    fn edit_mode_prev(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Decis, prev) if self.format <= Format::Ss => {
                Mode::Editable(Time::Seconds, prev)
            }
            Mode::Editable(Time::Decis, prev) if self.format <= Format::MmSs => {
                Mode::Editable(Time::Minutes, prev)
            }
            Mode::Editable(Time::Decis, prev) if self.format <= Format::HhMmSs => {
                Mode::Editable(Time::Hours, prev)
            }
            Mode::Editable(Time::Seconds, prev) if self.with_decis => {
                Mode::Editable(Time::Decis, prev)
            }
            Mode::Editable(Time::Seconds, prev) if self.format <= Format::Ss => {
                Mode::Editable(Time::Seconds, prev)
            }
            Mode::Editable(Time::Seconds, prev) if self.format <= Format::MmSs => {
                Mode::Editable(Time::Minutes, prev)
            }
            Mode::Editable(Time::Seconds, prev) if self.format <= Format::HhMmSs => {
                Mode::Editable(Time::Hours, prev)
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

    pub fn is_done(&self) -> bool {
        self.mode == Mode::Done
    }

    pub fn with_on_done_by_condition(
        mut self,
        condition: bool,
        handler: impl Fn() + 'static,
    ) -> Self {
        if condition {
            self.on_done = Some(Box::new(handler));
            self
        } else {
            self
        }
    }

    fn done(&mut self) {
        if !self.is_done() {
            self.mode = Mode::Done;
            if let Some(handler) = &mut self.on_done {
                handler();
            };
            if let Some(tx) = &mut self.app_tx {
                _ = tx.send(AppEvent::ClockDone);
            };
        }
    }

    fn update_format(&mut self) {
        self.format = self.get_format();
    }

    pub fn get_format(&self) -> Format {
        if self.current_value.hours() >= 10 {
            Format::HhMmSs
        } else if self.current_value.hours() >= 1 {
            Format::HMmSs
        } else if self.current_value.minutes() >= 10 {
            Format::MmSs
        } else if self.current_value.minutes() >= 1 {
            Format::MSs
        } else if self.current_value.seconds() >= 10 {
            Format::Ss
        } else {
            Format::S
        }
    }
}

#[derive(Debug, Clone)]
pub struct Countdown {}

impl ClockState<Countdown> {
    pub fn new(args: ClockStateArgs) -> Self {
        let ClockStateArgs {
            initial_value,
            current_value,
            tick_value,
            with_decis,
            app_tx,
        } = args;
        let mut instance = Self {
            initial_value: initial_value.into(),
            current_value: current_value.into(),
            tick_value: tick_value.into(),
            mode: if current_value == Duration::ZERO {
                Mode::Done
            } else if current_value == initial_value {
                Mode::Initial
            } else {
                Mode::Pause
            },
            format: Format::S,
            with_decis,
            on_done: None,
            app_tx,
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

    fn check_done(&mut self) {
        if self.current_value.eq(&Duration::ZERO.into()) {
            self.done();
        }
    }

    pub fn get_percentage_done(&self) -> u16 {
        let elapsed = self.initial_value.saturating_sub(self.current_value);

        (elapsed.millis() * 100 / self.initial_value.millis()) as u16
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

#[derive(Debug, Clone)]
pub struct Timer {}

impl ClockState<Timer> {
    pub fn new(args: ClockStateArgs) -> Self {
        let ClockStateArgs {
            initial_value,
            current_value,
            tick_value,
            with_decis,
            app_tx,
        } = args;
        let mut instance = Self {
            initial_value: initial_value.into(),
            current_value: current_value.into(),
            tick_value: tick_value.into(),
            mode: if current_value == initial_value {
                Mode::Initial
            } else if current_value >= MAX_DURATION {
                Mode::Done
            } else {
                Mode::Pause
            },
            format: Format::S,
            with_decis,
            on_done: None,
            app_tx,
            phantom: PhantomData,
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
        if self.current_value.ge(&MAX_DURATION.into()) {
            self.done();
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

pub struct ClockWidget<T>
where
    T: std::fmt::Debug,
{
    style: Style,
    phantom: PhantomData<T>,
}

impl<T> ClockWidget<T>
where
    T: std::fmt::Debug,
{
    pub fn new(style: Style) -> Self {
        Self {
            style,
            phantom: PhantomData,
        }
    }

    fn get_horizontal_lengths(&self, format: &Format, with_decis: bool) -> Vec<u16> {
        let add_decis = |mut lengths: Vec<u16>, with_decis: bool| -> Vec<u16> {
            if with_decis {
                lengths.extend_from_slice(&[
                    DOT_WIDTH,   // .
                    DIGIT_WIDTH, // ds
                ])
            }
            lengths
        };

        match format {
            Format::HhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,       // h
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // h
                    COLON_WIDTH,       // :
                    DIGIT_WIDTH,       // m
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // m
                    COLON_WIDTH,       // :
                    DIGIT_WIDTH,       // s
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // s
                ],
                with_decis,
            ),
            Format::HMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,       // h
                    COLON_WIDTH,       // :
                    DIGIT_WIDTH,       // m
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // m
                    COLON_WIDTH,       // :
                    DIGIT_WIDTH,       // s
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // s
                ],
                with_decis,
            ),
            Format::MmSs => add_decis(
                vec![
                    DIGIT_WIDTH,       // m
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // m
                    COLON_WIDTH,       // :
                    DIGIT_WIDTH,       // s
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // s
                ],
                with_decis,
            ),
            Format::MSs => add_decis(
                vec![
                    DIGIT_WIDTH,       // m
                    COLON_WIDTH,       // :
                    DIGIT_WIDTH,       // s
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // s
                ],
                with_decis,
            ),
            Format::Ss => add_decis(
                vec![
                    DIGIT_WIDTH,       // s
                    DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,       // s
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
}

impl<T> StatefulWidget for ClockWidget<T>
where
    T: std::fmt::Debug,
{
    type State = ClockState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let with_decis = state.with_decis;
        let format = state.format;
        let symbol = self.style.get_digit_symbol();
        let widths = self.get_horizontal_lengths(&format, with_decis);
        let area = center_horizontal(
            area,
            Constraint::Length(self.get_width(&format, with_decis)),
        );
        let edit_hours = matches!(state.mode, Mode::Editable(Time::Hours, _));
        let edit_minutes = matches!(state.mode, Mode::Editable(Time::Minutes, _));
        let edit_secs = matches!(state.mode, Mode::Editable(Time::Seconds, _));
        let edit_decis = matches!(state.mode, Mode::Editable(Time::Decis, _));
        match format {
            Format::HhMmSs if with_decis => {
                let [hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.hours() / 10, edit_hours, symbol).render(hh, buf);
                Digit::new(state.current_value.hours() % 10, edit_hours, symbol).render(h, buf);
                Colon::new(symbol).render(c_hm, buf);
                Digit::new(state.current_value.minutes_mod() / 10, edit_minutes, symbol)
                    .render(mm, buf);
                Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                    .render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
                Dot::new(symbol).render(d, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::HhMmSs => {
                let [hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.hours() / 10, edit_hours, symbol).render(hh, buf);
                Digit::new(state.current_value.hours() % 10, edit_hours, symbol).render(h, buf);
                Colon::new(symbol).render(c_hm, buf);
                Digit::new(state.current_value.minutes_mod() / 10, edit_minutes, symbol)
                    .render(mm, buf);
                Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                    .render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
            }
            Format::HMmSs if with_decis => {
                let [h, c_hm, mm, _, m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.hours() % 10, edit_hours, symbol).render(h, buf);
                Colon::new(symbol).render(c_hm, buf);
                Digit::new(state.current_value.minutes_mod() / 10, edit_minutes, symbol)
                    .render(mm, buf);
                Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                    .render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
                Dot::new(symbol).render(d, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::HMmSs => {
                let [h, c_hm, mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.hours() % 10, edit_hours, symbol).render(h, buf);
                Colon::new(symbol).render(c_hm, buf);
                Digit::new(state.current_value.minutes_mod() / 10, edit_minutes, symbol)
                    .render(mm, buf);
                Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                    .render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
            }
            Format::MmSs if with_decis => {
                let [mm, _, m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.minutes_mod() / 10, edit_minutes, symbol)
                    .render(mm, buf);
                Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                    .render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
                Dot::new(symbol).render(d, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::MmSs => {
                let [mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.minutes_mod() / 10, edit_minutes, symbol)
                    .render(mm, buf);
                Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                    .render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
            }
            Format::MSs if with_decis => {
                let [m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                    .render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
                Dot::new(symbol).render(d, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::MSs => {
                let [m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                    .render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
            }
            Format::Ss if state.with_decis => {
                let [ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
                Dot::new(symbol).render(d, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::Ss => {
                let [ss, _, s] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.seconds_mod() / 10, edit_secs, symbol)
                    .render(ss, buf);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
            }
            Format::S if with_decis => {
                let [s, d, ds] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
                Dot::new(symbol).render(d, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::S => {
                let [s] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol)
                    .render(s, buf);
            }
        }
    }
}
