use std::marker::PhantomData;

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

    pub fn format(&mut self) -> String {
        let ms = self.current_value;

        let minutes = (ms % 3600000) / 60000;
        let seconds = (ms % 60000) / 1000;
        let tenths = (ms % 1000) / 100;

        format!("{:02}:{:02}.{}", minutes, seconds, tenths)
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
