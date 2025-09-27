use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;
use strum::Display;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{StatefulWidget, Widget},
};

use crate::{
    common::{ClockTypeId, Style as DigitStyle},
    constants::{LABEL_DAYS, LABEL_YEARS},
    duration::{
        DurationEx, MAX_DURATION, ONE_DAY, ONE_DECI_SECOND, ONE_HOUR, ONE_MINUTE, ONE_SECOND,
        ONE_YEAR,
    },
    events::{AppEvent, AppEventTx},
    utils::center_horizontal,
    widgets::clock_elements::{
        COLON_WIDTH, Colon, DIGIT_HEIGHT, DIGIT_SPACE_WIDTH, DIGIT_WIDTH, DOT_WIDTH, Digit, Dot,
    },
};

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
pub enum Time {
    Decis,
    Seconds,
    Minutes,
    Hours,
    Days,
    Years,
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
                Time::Days => write!(f, "[edit days]"),
                Time::Years => write!(f, "[edit years]"),
            },
            Mode::Done => write!(f, "done"),
        }
    }
}

// Clock format:
// From `1 deciseconds` up to `999y 364d 23:59:59`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Display, PartialOrd, Ord)]
pub enum Format {
    S,
    Ss,
    MSs,
    MmSs,
    HMmSs,
    HhMmSs,
    DHhMmSs,
    DdHhMmSs,
    DddHhMmSs,
    YDddHhMmSs,
    YyDddHhMmSs,
    YyyDddHhMmSs,
}

const RANGE_OF_DONE_COUNT: u64 = 4;
const MAX_DONE_COUNT: u64 = RANGE_OF_DONE_COUNT * 5;

pub struct ClockState<T> {
    type_id: ClockTypeId,
    name: Option<String>,
    initial_value: DurationEx,
    current_value: DurationEx,
    prev_value: DurationEx,
    tick_value: DurationEx,
    mode: Mode,
    format: Format,
    pub with_decis: bool,
    app_tx: Option<AppEventTx>,
    /// Tick counter starting whenever `Mode::DONE` has been reached.
    /// Initial value is set in `done()`.
    /// Updates happened in `update_done_count`
    /// Default value: `None`
    done_count: Option<u64>,
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
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or_default()
    }

    pub fn get_type_id(&self) -> &ClockTypeId {
        &self.type_id
    }

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

    pub fn set_initial_value(&mut self, duration: DurationEx) {
        self.initial_value = duration;
    }

    pub fn get_current_value(&self) -> &DurationEx {
        &self.current_value
    }

    pub fn set_current_value(&mut self, duration: DurationEx) {
        self.current_value = duration;
        self.update_format();
    }

    pub fn get_prev_value(&self) -> &DurationEx {
        &self.prev_value
    }

    pub fn toggle_edit(&mut self) {
        self.mode = match self.mode.clone() {
            Mode::Editable(_, prev) => {
                let p = *prev;
                // Update `Mode`
                // 1. `Done` -> `Pause`
                if p == Mode::Done && self.current_value.gt(&Duration::ZERO.into()) {
                    Mode::Pause
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
                // store prev. value
                self.prev_value = self.current_value;
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
                    .le(&MAX_DURATION.saturating_sub(ONE_HOUR).into())
                {
                    self.current_value.saturating_add(ONE_HOUR.into())
                } else {
                    self.current_value
                }
            }
            Mode::Editable(Time::Days, _) => {
                if self
                    .current_value
                    .le(&MAX_DURATION.saturating_sub(ONE_DAY).into())
                {
                    self.current_value.saturating_add(ONE_DAY.into())
                } else {
                    self.current_value
                }
            }
            Mode::Editable(Time::Years, _) => {
                if self
                    .current_value
                    .lt(&MAX_DURATION.saturating_sub(ONE_YEAR).into())
                {
                    self.current_value.saturating_add(ONE_YEAR.into())
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
            Mode::Editable(Time::Days, _) => self.current_value.saturating_sub(ONE_DAY.into()),
            Mode::Editable(Time::Years, _) => self.current_value.saturating_sub(ONE_YEAR.into()),
            _ => self.current_value,
        };
        self.update_format();
        self.update_mode();
    }

    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, Mode::Editable(_, _))
    }

    // Circulating to next `Mode::Editable`
    // (Deciseconds ->) -> Seconds -> Minutes -> Hours → Days → Years
    // Note: next mode depends on `with_decis` and current format
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
            Mode::Editable(Time::Hours, prev) => Mode::Editable(Time::Days, prev),
            Mode::Editable(Time::Days, prev) => Mode::Editable(Time::Years, prev),
            Mode::Editable(Time::Years, prev) if self.with_decis => {
                Mode::Editable(Time::Decis, prev)
            }
            Mode::Editable(Time::Years, prev) => Mode::Editable(Time::Seconds, prev),
            _ => mode,
        };
        self.update_format();
    }

    // Circulating to previous `Mode::Editable`
    // Years -> Days -> Hours → Minutes → Seconds (→ Deciseconds)
    // Note: previous mode depends on `with_decis` and current format
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
            Mode::Editable(Time::Decis, prev) if self.format <= Format::DddHhMmSs => {
                Mode::Editable(Time::Days, prev)
            }
            Mode::Editable(Time::Decis, prev) => Mode::Editable(Time::Years, prev),
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
            Mode::Editable(Time::Seconds, prev) if self.format <= Format::DddHhMmSs => {
                Mode::Editable(Time::Days, prev)
            }
            Mode::Editable(Time::Seconds, prev) => Mode::Editable(Time::Years, prev),
            Mode::Editable(Time::Minutes, prev) => Mode::Editable(Time::Seconds, prev),
            Mode::Editable(Time::Hours, prev) => Mode::Editable(Time::Minutes, prev),
            Mode::Editable(Time::Days, prev) => Mode::Editable(Time::Hours, prev),
            Mode::Editable(Time::Years, prev) => Mode::Editable(Time::Days, prev),
            _ => mode,
        };
        self.update_format();
    }

    fn update_mode(&mut self) {
        let mode = self.mode.clone();
        self.mode = match mode {
            Mode::Editable(Time::Years, prev) if self.format <= Format::DddHhMmSs => {
                Mode::Editable(Time::Days, prev)
            }
            Mode::Editable(Time::Days, prev) if self.format <= Format::HhMmSs => {
                Mode::Editable(Time::Hours, prev)
            }
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

    fn done(&mut self) {
        if !self.is_done() {
            self.mode = Mode::Done;
            let type_id = self.get_type_id().clone();
            let name = self.get_name();
            if let Some(tx) = &self.app_tx {
                _ = tx.send(AppEvent::ClockDone(type_id, name));
            };
            self.done_count = Some(MAX_DONE_COUNT);
        }
    }

    fn update_format(&mut self) {
        self.format = self.get_format();
    }

    pub fn get_format(&self) -> Format {
        let v = self.current_value;
        if v.years() >= 100 {
            Format::YyyDddHhMmSs
        } else if v.years() >= 10 {
            Format::YyDddHhMmSs
        } else if v.years() >= 1 {
            Format::YDddHhMmSs
        } else if v.days() >= 100 {
            Format::DddHhMmSs
        } else if v.days() >= 10 {
            Format::DdHhMmSs
        } else if v.days() >= 1 {
            Format::DHhMmSs
        } else if v.hours() >= 10 {
            Format::HhMmSs
        } else if v.hours() >= 1 {
            Format::HMmSs
        } else if v.minutes() >= 10 {
            Format::MmSs
        } else if v.minutes() >= 1 {
            Format::MSs
        } else if v.seconds() >= 10 {
            Format::Ss
        } else {
            Format::S
        }
    }

    /// Updates inner value of `done_count`.
    /// It should be called whenever `TuiEvent::Tick` is handled.
    /// At first glance it might happen in `Clock::tick`, but sometimes
    /// `tick` won't be called again after `Mode::Done` event (e.g. in `widget::Countdown`).
    /// That's why `update_done_count` is called from "outside".
    pub fn update_done_count(&mut self) {
        if let Some(count) = self.done_count {
            if count > 0 {
                let value = count - 1;
                self.done_count = Some(value)
            } else {
                // None means we are done and no counting anymore.
                self.done_count = None
            }
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
            type_id: ClockTypeId::Countdown,
            name: None,
            initial_value: initial_value.into(),
            current_value: current_value.into(),
            prev_value: current_value.into(),
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
            app_tx,
            done_count: None,
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
            type_id: ClockTypeId::Timer,
            name: None,
            initial_value: initial_value.into(),
            current_value: current_value.into(),
            prev_value: current_value.into(),
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
            app_tx,
            done_count: None,
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
    style: DigitStyle,
    blink: bool,
    phantom: PhantomData<T>,
}

impl<T> ClockWidget<T>
where
    T: std::fmt::Debug,
{
    pub fn new(style: DigitStyle, blink: bool) -> Self {
        Self {
            style,
            blink,
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

        // TODO: Add x-offset (before+after)
        const LABEL_WIDTH: u16 = 1; // `Y` or `D`

        match format {
            Format::YyyDddHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,           // y
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // y
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // y
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // h
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // h
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // m
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // m
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // s
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // s
                ],
                with_decis,
            ),
            Format::YyDddHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,           // y
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // y
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // h
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // h
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // m
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // m
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // s
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // s
                ],
                with_decis,
            ),
            Format::YDddHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,           // Y
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // h
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // h
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // m
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // m
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // s
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // s
                ],
                with_decis,
            ),
            Format::DddHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // h
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // h
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // m
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // m
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // s
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // s
                ],
                with_decis,
            ),
            Format::DdHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // d
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // h
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // h
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // m
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // m
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // s
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // s
                ],
                with_decis,
            ),
            Format::DHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,           // D
                    DIGIT_SPACE_WIDTH,     // (space)
                    LABEL_WIDTH,           // label
                    2 * DIGIT_SPACE_WIDTH, // (space)
                    DIGIT_WIDTH,           // h
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // h
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // m
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // m
                    COLON_WIDTH,           // :
                    DIGIT_WIDTH,           // s
                    DIGIT_SPACE_WIDTH,     // (space)
                    DIGIT_WIDTH,           // s
                ],
                with_decis,
            ),
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

    /// Checks whether to blink the clock while rendering.
    /// Its logic is based on a given `count` value.
    fn should_blink(&self, count_value: &Option<u64>) -> bool {
        // Example:
        // if `RANGE_OF_DONE_COUNT` is 4
        // then for ranges `0..4`, `8..12` etc. it will return `true`
        count_value
            .map(|b| (b % (RANGE_OF_DONE_COUNT * 2)) < RANGE_OF_DONE_COUNT)
            .unwrap_or(false)
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
        // to simulate a blink effect, just use an "empty" symbol (string)
        // to "empty" all digits and to have an "empty" render area
        let symbol = if self.blink && self.should_blink(&state.done_count) {
            " "
        } else {
            self.style.get_digit_symbol()
        };
        let widths = self.get_horizontal_lengths(&format, with_decis);
        let area = center_horizontal(
            area,
            Constraint::Length(self.get_width(&format, with_decis)),
        );
        let edit_years = matches!(state.mode, Mode::Editable(Time::Years, _));
        let edit_days = matches!(state.mode, Mode::Editable(Time::Days, _));
        let edit_hours = matches!(state.mode, Mode::Editable(Time::Hours, _));
        let edit_minutes = matches!(state.mode, Mode::Editable(Time::Minutes, _));
        let edit_secs = matches!(state.mode, Mode::Editable(Time::Seconds, _));
        let edit_decis = matches!(state.mode, Mode::Editable(Time::Decis, _));

        match format {
            Format::YyyDddHhMmSs if with_decis => {
                let [
                    yyy,
                    _,
                    yy,
                    _,
                    y,
                    _,
                    ly,
                    _,
                    ddd,
                    _,
                    dd,
                    _,
                    d,
                    _,
                    ld,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                    dot,
                    ds,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new((state.current_value.years() / 100) % 10, edit_years, symbol)
                    .render(yyy, buf);
                Digit::new((state.current_value.years() / 10) % 10, edit_years, symbol)
                    .render(yy, buf);
                Digit::new(state.current_value.years() % 10, edit_years, symbol).render(y, buf);
                Span::styled(
                    LABEL_YEARS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ly, buf);
                Digit::new(
                    (state.current_value.days_mod() / 100) % 10,
                    edit_days,
                    symbol,
                )
                .render(ddd, buf);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ld, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Dot::new(symbol).render(dot, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::YyyDddHhMmSs => {
                let [
                    yyy,
                    _,
                    yy,
                    _,
                    y,
                    _,
                    ly,
                    _,
                    ddd,
                    _,
                    dd,
                    _,
                    d,
                    _,
                    ld,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new((state.current_value.years() / 100) % 10, edit_years, symbol)
                    .render(yyy, buf);
                Digit::new((state.current_value.years() / 10) % 10, edit_years, symbol)
                    .render(yy, buf);
                Digit::new(state.current_value.years() % 10, edit_years, symbol).render(y, buf);
                Span::styled(
                    LABEL_YEARS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ly, buf);
                Digit::new(
                    (state.current_value.days_mod() / 100) % 10,
                    edit_days,
                    symbol,
                )
                .render(ddd, buf);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ld, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
            Format::YyDddHhMmSs if with_decis => {
                let [
                    yy,
                    _,
                    y,
                    _,
                    ly,
                    _,
                    ddd,
                    _,
                    dd,
                    _,
                    d,
                    _,
                    ld,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                    dot,
                    ds,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new((state.current_value.years() / 10) % 10, edit_years, symbol)
                    .render(yy, buf);
                Digit::new(state.current_value.years() % 10, edit_years, symbol).render(y, buf);
                Span::styled(
                    LABEL_YEARS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ly, buf);
                Digit::new(
                    (state.current_value.days_mod() / 100) % 10,
                    edit_days,
                    symbol,
                )
                .render(ddd, buf);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ld, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Dot::new(symbol).render(dot, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::YyDddHhMmSs => {
                let [
                    yy,
                    _,
                    y,
                    _,
                    ly,
                    _,
                    ddd,
                    _,
                    dd,
                    _,
                    d,
                    _,
                    ld,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new((state.current_value.years() / 10) % 10, edit_years, symbol)
                    .render(yy, buf);
                Digit::new(state.current_value.years() % 10, edit_years, symbol).render(y, buf);
                Span::styled(
                    LABEL_YEARS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ly, buf);
                Digit::new(
                    (state.current_value.days_mod() / 100) % 10,
                    edit_days,
                    symbol,
                )
                .render(ddd, buf);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ld, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
            Format::YDddHhMmSs if with_decis => {
                let [
                    y,
                    _,
                    ly,
                    _,
                    ddd,
                    _,
                    dd,
                    _,
                    d,
                    _,
                    ld,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                    dot,
                    ds,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.years() % 10, edit_years, symbol).render(y, buf);
                Span::styled(
                    LABEL_YEARS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ly, buf);
                Digit::new(
                    (state.current_value.days_mod() / 100) % 10,
                    edit_days,
                    symbol,
                )
                .render(ddd, buf);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ld, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Dot::new(symbol).render(dot, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::YDddHhMmSs => {
                let [
                    y,
                    _,
                    ly,
                    _,
                    ddd,
                    _,
                    dd,
                    _,
                    d,
                    _,
                    ld,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.years() % 10, edit_years, symbol).render(y, buf);
                Span::styled(
                    LABEL_YEARS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ly, buf);
                Digit::new(
                    (state.current_value.days_mod() / 100) % 10,
                    edit_days,
                    symbol,
                )
                .render(ddd, buf);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(ld, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
            Format::DddHhMmSs if with_decis => {
                let [
                    ddd,
                    _,
                    dd,
                    _,
                    d,
                    _,
                    l,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                    dot,
                    ds,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(
                    (state.current_value.days_mod() / 100) % 10,
                    edit_days,
                    symbol,
                )
                .render(ddd, buf);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(l, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Dot::new(symbol).render(dot, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::DddHhMmSs => {
                let [
                    ddd,
                    _,
                    dd,
                    _,
                    d,
                    _,
                    l,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(
                    (state.current_value.days_mod() / 100) % 10,
                    edit_days,
                    symbol,
                )
                .render(ddd, buf);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(l, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
            Format::DdHhMmSs if with_decis => {
                let [
                    dd,
                    _,
                    d,
                    _,
                    l,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                    dot,
                    ds,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(l, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Dot::new(symbol).render(dot, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::DdHhMmSs => {
                let [dd, _, d, _, l, _, hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(
                    (state.current_value.days_mod() / 10) % 10,
                    edit_days,
                    symbol,
                )
                .render(dd, buf);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(l, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
            Format::DHhMmSs if with_decis => {
                let [
                    d,
                    _,
                    l,
                    _,
                    hh,
                    _,
                    h,
                    c_hm,
                    mm,
                    _,
                    m,
                    c_ms,
                    ss,
                    _,
                    s,
                    dot,
                    ds,
                ] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(l, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Dot::new(symbol).render(dot, buf);
                Digit::new(state.current_value.decis(), edit_decis, symbol).render(ds, buf);
            }
            Format::DHhMmSs => {
                let [d, _, l, _, hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(d, buf);
                Span::styled(
                    LABEL_DAYS.to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(l, buf);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
            Format::HhMmSs if with_decis => {
                let [hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Digit::new(state.current_value.hours_mod() / 10, edit_hours, symbol)
                    .render(hh, buf);
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
                Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(h, buf);
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
