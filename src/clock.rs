use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;
use strum::Display;

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
