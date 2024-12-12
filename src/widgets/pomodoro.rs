use crate::{
    constants::TICK_VALUE_MS,
    events::{Event, EventHandler},
    utils::center,
    widgets::clock::{Clock, ClockWidget, Countdown},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use std::{cmp::max, time::Duration};

use strum::Display;

static PAUSE_MS: u64 = 5 * 60 * 1000; /* 5min in milliseconds */
static WORK_MS: u64 = 25 * 60 * 1000; /* 25min in milliseconds */

#[derive(Debug, Clone, Display, Hash, Eq, PartialEq)]
enum Mode {
    Work,
    Pause,
}

#[derive(Debug, Clone)]
pub struct ClockMap {
    work: Clock<Countdown>,
    pause: Clock<Countdown>,
}

impl ClockMap {
    fn get(&mut self, mode: &Mode) -> &mut Clock<Countdown> {
        match mode {
            Mode::Work => &mut self.work,
            Mode::Pause => &mut self.pause,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pomodoro {
    mode: Mode,
    clock_map: ClockMap,
}

impl Pomodoro {
    pub fn new() -> Self {
        Self {
            mode: Mode::Work,
            clock_map: ClockMap {
                work: Clock::<Countdown>::new(
                    Duration::from_millis(WORK_MS),
                    Duration::from_millis(TICK_VALUE_MS),
                ),
                pause: Clock::<Countdown>::new(
                    Duration::from_millis(PAUSE_MS),
                    Duration::from_millis(TICK_VALUE_MS),
                ),
            },
        }
    }

    fn get_clock(&mut self) -> &mut Clock<Countdown> {
        self.clock_map.get(&self.mode)
    }

    pub fn next(&mut self) {
        self.mode = match self.mode {
            Mode::Pause => Mode::Work,
            Mode::Work => Mode::Pause,
        };
    }

    pub fn is_edit_mode(&mut self) -> bool {
        self.get_clock().is_edit_mode()
    }
}

impl EventHandler for Pomodoro {
    fn update(&mut self, event: Event) {
        match event {
            Event::Tick => {
                self.get_clock().tick();
            }
            Event::Key(key) => match key.code {
                KeyCode::Char('s') => {
                    self.get_clock().toggle_pause();
                }
                KeyCode::Char('e') => {
                    self.get_clock().toggle_edit();
                }
                KeyCode::Left => {
                    if self.get_clock().is_edit_mode() {
                        self.get_clock().edit_next();
                    } else {
                        // `next` is acting as same as a `prev` function, we don't have
                        self.next();
                    }
                }
                KeyCode::Right => {
                    if self.get_clock().is_edit_mode() {
                        self.get_clock().edit_prev();
                    } else {
                        self.next();
                    }
                }
                KeyCode::Up => {
                    self.get_clock().edit_up();
                }
                KeyCode::Down => {
                    self.get_clock().edit_down();
                }
                KeyCode::Char('r') => {
                    self.get_clock().reset();
                }
                _ => {}
            },

            _ => {}
        }
    }
}

pub struct PomodoroWidget;

impl StatefulWidget for PomodoroWidget {
    type State = Pomodoro;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock = ClockWidget::new();
        let label = Line::raw(
            (format!(
                "Pomodoro {} {}",
                state.mode.clone(),
                state.get_clock().get_mode()
            ))
            .to_uppercase(),
        );

        let area = center(
            area,
            Constraint::Length(max(clock.get_width(), label.width() as u16)),
            Constraint::Length(clock.get_height() + 1 /* height of mode_str */),
        );

        let [v1, v2] =
            Layout::vertical(Constraint::from_lengths([clock.get_height(), 1])).areas(area);

        clock.render(v1, buf, state.get_clock());
        label.centered().render(v2, buf);
    }
}
