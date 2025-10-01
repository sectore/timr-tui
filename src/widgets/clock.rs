use std::marker::PhantomData;
use std::time::Duration;
use std::{fmt, ops::Mul};
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
    duration::{
        DurationEx, MAX_DURATION, ONE_DAY, ONE_DECI_SECOND, ONE_HOUR, ONE_MINUTE, ONE_SECOND,
        ONE_YEAR,
    },
    events::{AppEvent, AppEventTx},
    utils::center_horizontal,
    widgets::clock_elements::{
        COLON_WIDTH, Colon, DIGIT_HEIGHT, DIGIT_LABEL_WIDTH, DIGIT_SPACE_WIDTH, DIGIT_WIDTH,
        DOT_WIDTH, Digit, Dot, THREE_DIGITS_WIDTH, TWO_DIGITS_WIDTH,
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
// From `1s` up to `999y 364d 23:59:59`
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
    YDHhMmSs,
    YDdHhMmSs,
    YDddHhMmSs,
    YyDHhMmSs,
    YyDdHhMmSs,
    YyDddHhMmSs,
    YyyDHhMmSs,
    YyyDdHhMmSs,
    YyyDddHhMmSs,
}

pub fn format_by_duration(d: &DurationEx) -> Format {
    if d.years() >= 100 && d.days_mod() >= 100 {
        Format::YyyDddHhMmSs
    } else if d.years() >= 100 && d.days_mod() >= 10 {
        Format::YyyDdHhMmSs
    } else if d.years() >= 100 && d.days() >= 1 {
        Format::YyyDHhMmSs
    } else if d.years() >= 10 && d.days_mod() >= 100 {
        Format::YyDddHhMmSs
    } else if d.years() >= 10 && d.days_mod() >= 10 {
        Format::YyDdHhMmSs
    } else if d.years() >= 10 && d.days() >= 1 {
        Format::YyDHhMmSs
    } else if d.years() >= 1 && d.days_mod() >= 100 {
        Format::YDddHhMmSs
    } else if d.years() >= 1 && d.days_mod() >= 10 {
        Format::YDdHhMmSs
    } else if d.years() >= 1 && d.days() >= 1 {
        Format::YDHhMmSs
    } else if d.days() >= 100 {
        Format::DddHhMmSs
    } else if d.days() >= 10 {
        Format::DdHhMmSs
    } else if d.days() >= 1 {
        Format::DHhMmSs
    } else if d.hours() >= 10 {
        Format::HhMmSs
    } else if d.hours() >= 1 {
        Format::HMmSs
    } else if d.minutes() >= 10 {
        Format::MmSs
    } else if d.minutes() >= 1 {
        Format::MSs
    } else if d.seconds() >= 10 {
        Format::Ss
    } else {
        Format::S
    }
}

pub fn time_by_format(format: &Format) -> Time {
    match format {
        Format::YDddHhMmSs
        | Format::YDdHhMmSs
        | Format::YDHhMmSs
        | Format::YyDddHhMmSs
        | Format::YyDdHhMmSs
        | Format::YyDHhMmSs
        | Format::YyyDddHhMmSs
        | Format::YyyDdHhMmSs
        | Format::YyyDHhMmSs => Time::Years,
        Format::DddHhMmSs | Format::DdHhMmSs | Format::DHhMmSs => Time::Days,
        Format::HhMmSs | Format::HMmSs => Time::Hours,
        Format::MmSs | Format::MSs => Time::Minutes,
        Format::Ss | Format::S => Time::Seconds,
    }
}

pub fn count_by_mode(times: u32, mode: &Mode) -> Duration {
    match mode {
        Mode::Editable(Time::Decis, _) => ONE_DECI_SECOND.mul(times),
        Mode::Editable(Time::Seconds, _) => ONE_SECOND.mul(times),
        Mode::Editable(Time::Minutes, _) => ONE_MINUTE.mul(times),
        Mode::Editable(Time::Hours, _) => ONE_HOUR.mul(times),
        Mode::Editable(Time::Days, _) => ONE_DAY.mul(times),
        Mode::Editable(Time::Years, _) => ONE_YEAR.mul(times),
        _ => Duration::ZERO,
    }
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

    pub fn get_format(&self) -> &Format {
        &self.format
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

    fn edit_current_up(&mut self, times: u32) {
        let count_value = count_by_mode(times, self.get_mode());

        if self
            .get_current_value()
            .le(&MAX_DURATION.saturating_sub(count_value).into())
        {
            self.current_value = self.get_current_value().saturating_add(count_value.into());
            self.update_format();
        }
    }

    fn edit_current_down(&mut self, times: u32) {
        let count_value = count_by_mode(times, self.get_mode()).into();

        self.current_value = self.get_current_value().saturating_sub(count_value);

        self.update_format();
        let updated_format = *self.get_format();
        self.downgrade_mode_by_format(&updated_format);
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

    // Note: Since `Format` does not include `deciseconds` for different reason,
    // `Mode::Editable` can be downgraded up to `Time::Seconds` (but not to `Time::Decis`)
    fn downgrade_mode_by_format(&mut self, format: &Format) {
        let mode = self.mode.clone();
        let time = time_by_format(format);
        self.mode = match mode {
            Mode::Editable(Time::Years, prev) if format <= &Format::DddHhMmSs => {
                Mode::Editable(time, prev)
            }
            Mode::Editable(Time::Days, prev) if format <= &Format::HhMmSs => {
                Mode::Editable(time, prev)
            }
            Mode::Editable(Time::Hours, prev) if format <= &Format::MmSs => {
                Mode::Editable(time, prev)
            }
            Mode::Editable(Time::Minutes, prev) if format <= &Format::Ss => {
                Mode::Editable(time, prev)
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
        let d = self.get_current_value();
        self.format = format_by_duration(d);
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
        if Duration::is_zero(&self.initial_value.into()) {
            return 0;
        }

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
        self.edit_current_up(1);
    }

    pub fn edit_down(&mut self) {
        self.edit_current_down(1);
    }

    pub fn edit_jump_up(&mut self) {
        self.edit_current_up(10);
    }

    pub fn edit_jump_down(&mut self) {
        self.edit_current_down(10);
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
        self.edit_current_up(1);
    }

    pub fn edit_down(&mut self) {
        self.edit_current_down(1);
    }

    pub fn edit_jump_up(&mut self) {
        self.edit_current_up(10);
    }

    pub fn edit_jump_down(&mut self) {
        self.edit_current_down(10);
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

        const LABEL_WIDTH: u16 = DIGIT_LABEL_WIDTH + DIGIT_SPACE_WIDTH;

        match format {
            Format::YyyDddHhMmSs => add_decis(
                vec![
                    THREE_DIGITS_WIDTH, // y_y_y
                    LABEL_WIDTH,        // _l__
                    THREE_DIGITS_WIDTH, // d_d_d
                    LABEL_WIDTH,        // _l__
                    TWO_DIGITS_WIDTH,   // h_h
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // m_m
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // s_s
                ],
                with_decis,
            ),
            Format::YyyDdHhMmSs => add_decis(
                vec![
                    THREE_DIGITS_WIDTH, // y_y_y
                    LABEL_WIDTH,        // _l__
                    TWO_DIGITS_WIDTH,   // d_d
                    LABEL_WIDTH,        // _l__
                    TWO_DIGITS_WIDTH,   // h_h
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // m_m
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // s_s
                ],
                with_decis,
            ),
            Format::YyyDHhMmSs => add_decis(
                vec![
                    THREE_DIGITS_WIDTH, // y_y_y
                    LABEL_WIDTH,        // _l__
                    DIGIT_WIDTH,        // d
                    LABEL_WIDTH,        // _l__
                    TWO_DIGITS_WIDTH,   // h_h
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // m_m
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // s_s
                ],
                with_decis,
            ),
            Format::YyDddHhMmSs => add_decis(
                vec![
                    TWO_DIGITS_WIDTH,   // y_y
                    LABEL_WIDTH,        // _l__
                    THREE_DIGITS_WIDTH, // d_d_d
                    LABEL_WIDTH,        // _l__
                    TWO_DIGITS_WIDTH,   // h_h
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // m_m
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // s_s
                ],
                with_decis,
            ),
            Format::YyDdHhMmSs => add_decis(
                vec![
                    TWO_DIGITS_WIDTH, // y_y
                    LABEL_WIDTH,      // _l__
                    TWO_DIGITS_WIDTH, // d_d
                    LABEL_WIDTH,      // _l__
                    TWO_DIGITS_WIDTH, // h_h
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::YyDHhMmSs => add_decis(
                vec![
                    TWO_DIGITS_WIDTH, // y_y
                    LABEL_WIDTH,      // _l__
                    DIGIT_WIDTH,      // d
                    LABEL_WIDTH,      // _l__
                    TWO_DIGITS_WIDTH, // h_h
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::YDddHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,        // Y
                    LABEL_WIDTH,        // _l__
                    THREE_DIGITS_WIDTH, // d_d_d
                    LABEL_WIDTH,        // _l__
                    TWO_DIGITS_WIDTH,   // h_h
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // m_m
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // s_s
                ],
                with_decis,
            ),
            Format::YDdHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,      // Y
                    LABEL_WIDTH,      // _l__
                    TWO_DIGITS_WIDTH, // d_d
                    LABEL_WIDTH,      // _l__
                    TWO_DIGITS_WIDTH, // h_h
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::YDHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,      // Y
                    LABEL_WIDTH,      // _l__
                    DIGIT_WIDTH,      // d
                    LABEL_WIDTH,      // _l__
                    TWO_DIGITS_WIDTH, // h_h
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),

            Format::DddHhMmSs => add_decis(
                vec![
                    THREE_DIGITS_WIDTH, // d_d_d
                    LABEL_WIDTH,        // _l__
                    TWO_DIGITS_WIDTH,   // h_h
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // m_m
                    COLON_WIDTH,        // :
                    TWO_DIGITS_WIDTH,   // s_s
                ],
                with_decis,
            ),
            Format::DdHhMmSs => add_decis(
                vec![
                    TWO_DIGITS_WIDTH, // d_d
                    LABEL_WIDTH,      // _l__
                    TWO_DIGITS_WIDTH, // h_h
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::DHhMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,      // D
                    LABEL_WIDTH,      // _l__
                    TWO_DIGITS_WIDTH, // h_h
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::HhMmSs => add_decis(
                vec![
                    TWO_DIGITS_WIDTH, // h_h
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::HMmSs => add_decis(
                vec![
                    DIGIT_WIDTH,      // h
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::MmSs => add_decis(
                vec![
                    TWO_DIGITS_WIDTH, // m_m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::MSs => add_decis(
                vec![
                    DIGIT_WIDTH,      // m
                    COLON_WIDTH,      // :
                    TWO_DIGITS_WIDTH, // s_s
                ],
                with_decis,
            ),
            Format::Ss => add_decis(
                vec![
                    TWO_DIGITS_WIDTH, // s_s
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

        let render_three_digits = |d1, d2, d3, editable, area, buf: &mut Buffer| {
            let [a1, a2, a3] = Layout::horizontal(Constraint::from_lengths([
                DIGIT_WIDTH + DIGIT_SPACE_WIDTH,
                DIGIT_WIDTH + DIGIT_SPACE_WIDTH,
                DIGIT_WIDTH,
            ]))
            .areas(area);
            Digit::new(d1, editable, symbol).render(a1, buf);
            Digit::new(d2, editable, symbol).render(a2, buf);
            Digit::new(d3, editable, symbol).render(a3, buf);
        };

        let render_two_digits = |d1, d2, editable, area, buf: &mut Buffer| {
            let [a1, a2] = Layout::horizontal(Constraint::from_lengths([
                DIGIT_WIDTH + DIGIT_SPACE_WIDTH,
                DIGIT_WIDTH,
            ]))
            .areas(area);
            Digit::new(d1, editable, symbol).render(a1, buf);
            Digit::new(d2, editable, symbol).render(a2, buf);
        };

        let render_colon = |area, buf: &mut Buffer| {
            Colon::new(symbol).render(area, buf);
        };

        let render_dot = |area, buf: &mut Buffer| {
            Dot::new(symbol).render(area, buf);
        };

        let render_yyy = |area, buf| {
            render_three_digits(
                (state.current_value.years() / 100) % 10,
                (state.current_value.years() / 10) % 10,
                state.current_value.years() % 10,
                edit_years,
                area,
                buf,
            );
        };

        let render_yy = |area, buf| {
            render_two_digits(
                (state.current_value.years() / 10) % 10,
                state.current_value.years() % 10,
                edit_years,
                area,
                buf,
            );
        };

        let render_y = |area, buf| {
            Digit::new(state.current_value.years() % 10, edit_years, symbol).render(area, buf);
        };

        let render_ddd = |area, buf| {
            render_three_digits(
                (state.current_value.days_mod() / 100) % 10,
                (state.current_value.days_mod() / 10) % 10,
                state.current_value.days_mod() % 10,
                edit_days,
                area,
                buf,
            );
        };

        let render_dd = |area, buf| {
            render_two_digits(
                (state.current_value.days_mod() / 10) % 10,
                state.current_value.days_mod() % 10,
                edit_days,
                area,
                buf,
            );
        };

        let render_d = |area, buf| {
            Digit::new(state.current_value.days_mod() % 10, edit_days, symbol).render(area, buf);
        };

        let render_hh = |area, buf| {
            render_two_digits(
                state.current_value.hours_mod() / 10,
                state.current_value.hours_mod() % 10,
                edit_hours,
                area,
                buf,
            );
        };

        let render_h = |area, buf| {
            Digit::new(state.current_value.hours_mod() % 10, edit_hours, symbol).render(area, buf);
        };

        let render_mm = |area, buf| {
            render_two_digits(
                state.current_value.minutes_mod() / 10,
                state.current_value.minutes_mod() % 10,
                edit_minutes,
                area,
                buf,
            );
        };

        let render_m = |area, buf| {
            Digit::new(state.current_value.minutes_mod() % 10, edit_minutes, symbol)
                .render(area, buf);
        };

        let render_ss = |area, buf| {
            render_two_digits(
                state.current_value.seconds_mod() / 10,
                state.current_value.seconds_mod() % 10,
                edit_secs,
                area,
                buf,
            );
        };

        let render_s = |area, buf| {
            Digit::new(state.current_value.seconds_mod() % 10, edit_secs, symbol).render(area, buf);
        };

        let render_ds = |area, buf| {
            Digit::new(state.current_value.decis(), edit_decis, symbol).render(area, buf);
        };

        let render_label = |l: &str, area, buf: &mut Buffer| {
            Span::styled(
                format!(" {l}").to_uppercase(),
                Style::default().add_modifier(Modifier::BOLD),
            )
            .render(area, buf);
        };

        let render_label_y = |area, buf| {
            render_label("Y", area, buf);
        };

        let render_label_d = |area, buf| {
            render_label("D", area, buf);
        };

        match format {
            Format::YyyDddHhMmSs if with_decis => {
                let [y_y_y, ly, d_d_d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yyy(y_y_y, buf);
                render_label_y(ly, buf);
                render_ddd(d_d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YyyDddHhMmSs => {
                let [y_y_y, ly, d_d_d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yyy(y_y_y, buf);
                render_label_y(ly, buf);
                render_ddd(d_d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::YyyDdHhMmSs if with_decis => {
                let [y_y_y, ly, d_d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yyy(y_y_y, buf);
                render_label_y(ly, buf);
                render_dd(d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YyyDdHhMmSs => {
                let [y_y_y, ly, d_d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yyy(y_y_y, buf);
                render_label_y(ly, buf);
                render_dd(d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::YyyDHhMmSs if with_decis => {
                let [y_y_y, ly, d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yyy(y_y_y, buf);
                render_label_y(ly, buf);
                render_d(d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YyyDHhMmSs => {
                let [y_y_y, ly, d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yyy(y_y_y, buf);
                render_label_y(ly, buf);
                render_d(d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::YyDddHhMmSs if with_decis => {
                let [y_y, ly, d_d_d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yy(y_y, buf);
                render_label_y(ly, buf);
                render_ddd(d_d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YyDddHhMmSs => {
                let [y_y, ly, d_d_d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yy(y_y, buf);
                render_label_y(ly, buf);
                render_ddd(d_d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::YyDdHhMmSs if with_decis => {
                let [y_y, ly, d_d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yy(y_y, buf);
                render_label_y(ly, buf);
                render_dd(d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YyDdHhMmSs => {
                let [y_y, ly, d_d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yy(y_y, buf);
                render_label_y(ly, buf);
                render_dd(d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::YyDHhMmSs if with_decis => {
                let [y_y, ly, d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yy(y_y, buf);
                render_label_y(ly, buf);
                render_d(d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YyDHhMmSs => {
                let [y_y, ly, d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_yy(y_y, buf);
                render_label_y(ly, buf);
                render_d(d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::YDddHhMmSs if with_decis => {
                let [y, ly, d_d_d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_y(y, buf);
                render_label_y(ly, buf);
                render_ddd(d_d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YDddHhMmSs => {
                let [y, ly, d_d_d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_y(y, buf);
                render_label_y(ly, buf);
                render_ddd(d_d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::YDdHhMmSs if with_decis => {
                let [y, ly, d_d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_y(y, buf);
                render_label_y(ly, buf);
                render_dd(d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YDdHhMmSs => {
                let [y, ly, d_d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_y(y, buf);
                render_label_y(ly, buf);
                render_dd(d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::YDHhMmSs if with_decis => {
                let [y, ly, d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_y(y, buf);
                render_label_y(ly, buf);
                render_d(d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::YDHhMmSs => {
                let [y, ly, d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_y(y, buf);
                render_label_y(ly, buf);
                render_d(d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::DddHhMmSs if with_decis => {
                let [d_d_d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_ddd(d_d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::DddHhMmSs => {
                let [d_d_d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_ddd(d_d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::DdHhMmSs if with_decis => {
                let [d_d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_dd(d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::DdHhMmSs => {
                let [d_d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_dd(d_d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::DHhMmSs if with_decis => {
                let [d, ld, h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_d(d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::DHhMmSs => {
                let [d, ld, h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_d(d, buf);
                render_label_d(ld, buf);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::HhMmSs if with_decis => {
                let [h_h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::HhMmSs => {
                let [h_h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_hh(h_h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::HMmSs if with_decis => {
                let [h, c_hm, m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_h(h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::HMmSs => {
                let [h, c_hm, m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_h(h, buf);
                render_colon(c_hm, buf);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::MmSs if with_decis => {
                let [m_m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::MmSs => {
                let [m_m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_mm(m_m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::MSs if with_decis => {
                let [m, c_ms, s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_m(m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::MSs => {
                let [m, c_ms, s_s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_m(m, buf);
                render_colon(c_ms, buf);
                render_ss(s_s, buf);
            }
            Format::Ss if state.with_decis => {
                let [s_s, dot, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_ss(s_s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::Ss => {
                let [s_s] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_ss(s_s, buf);
            }
            Format::S if with_decis => {
                let [s, dot, ds] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_s(s, buf);
                render_dot(dot, buf);
                render_ds(ds, buf);
            }
            Format::S => {
                let [s] = Layout::horizontal(Constraint::from_lengths(widths)).areas(area);
                render_s(s, buf);
            }
        }
    }
}
