use crate::{
    common::Style,
    constants::TICK_VALUE_MS,
    events::{Event, EventHandler},
    utils::center,
    widgets::clock::{ClockState, ClockWidget, Countdown},
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

use serde::{Deserialize, Serialize};

use super::clock::ClockStateArgs;

#[derive(Debug, Clone, Display, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum Mode {
    Work,
    Pause,
}

#[derive(Debug, Clone)]
pub struct ClockMap {
    work: ClockState<Countdown>,
    pause: ClockState<Countdown>,
}

impl ClockMap {
    fn get_mut(&mut self, mode: &Mode) -> &mut ClockState<Countdown> {
        match mode {
            Mode::Work => &mut self.work,
            Mode::Pause => &mut self.pause,
        }
    }
    fn get(&self, mode: &Mode) -> &ClockState<Countdown> {
        match mode {
            Mode::Work => &self.work,
            Mode::Pause => &self.pause,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PomodoroState {
    mode: Mode,
    clock_map: ClockMap,
}

pub struct PomodoroStateArgs {
    pub mode: Mode,
    pub initial_value_work: Duration,
    pub current_value_work: Duration,
    pub initial_value_pause: Duration,
    pub current_value_pause: Duration,
    pub with_decis: bool,
}

impl PomodoroState {
    pub fn new(args: PomodoroStateArgs) -> Self {
        let PomodoroStateArgs {
            mode,
            initial_value_work,
            current_value_work,
            initial_value_pause,
            current_value_pause,
            with_decis,
        } = args;
        Self {
            mode,
            clock_map: ClockMap {
                work: ClockState::<Countdown>::new(ClockStateArgs {
                    initial_value: initial_value_work,
                    current_value: current_value_work,
                    tick_value: Duration::from_millis(TICK_VALUE_MS),
                    with_decis,
                }),
                pause: ClockState::<Countdown>::new(ClockStateArgs {
                    initial_value: initial_value_pause,
                    current_value: current_value_pause,
                    tick_value: Duration::from_millis(TICK_VALUE_MS),
                    with_decis,
                }),
            },
        }
    }

    fn get_clock_mut(&mut self) -> &mut ClockState<Countdown> {
        self.clock_map.get_mut(&self.mode)
    }

    pub fn get_clock(&self) -> &ClockState<Countdown> {
        self.clock_map.get(&self.mode)
    }

    pub fn get_clock_work(&self) -> &ClockState<Countdown> {
        &self.clock_map.work
    }

    pub fn get_clock_pause(&self) -> &ClockState<Countdown> {
        &self.clock_map.pause
    }

    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock_map.work.with_decis = with_decis;
        self.clock_map.pause.with_decis = with_decis;
    }

    pub fn next(&mut self) {
        self.mode = match self.mode {
            Mode::Pause => Mode::Work,
            Mode::Work => Mode::Pause,
        };
    }
}

impl EventHandler for PomodoroState {
    fn update(&mut self, event: Event) -> Option<Event> {
        let edit_mode = self.get_clock().is_edit_mode();
        match event {
            Event::Tick => {
                self.get_clock_mut().tick();
            }
            Event::Key(key) => match key.code {
                KeyCode::Char('s') => {
                    self.get_clock_mut().toggle_pause();
                }
                KeyCode::Char('e') => {
                    self.get_clock_mut().toggle_edit();
                }
                KeyCode::Left if edit_mode => {
                    self.get_clock_mut().edit_next();
                }
                KeyCode::Left => {
                    // `next` is acting as same as a `prev` function, we don't have
                    self.next();
                }
                KeyCode::Right if edit_mode => {
                    self.get_clock_mut().edit_prev();
                }
                KeyCode::Right => {
                    self.next();
                }
                KeyCode::Up if edit_mode => {
                    self.get_clock_mut().edit_up();
                }
                KeyCode::Down if edit_mode => {
                    self.get_clock_mut().edit_down();
                }
                KeyCode::Char('r') => {
                    self.get_clock_mut().reset();
                }
                _ => return Some(event),
            },
            _ => return Some(event),
        }
        None
    }
}

pub struct PomodoroWidget {
    pub style: Style,
}

impl StatefulWidget for PomodoroWidget {
    type State = PomodoroState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock_widget = ClockWidget::new(self.style);
        let label = Line::raw(
            (format!(
                "Pomodoro {} {}",
                state.mode.clone(),
                state.get_clock_mut().get_mode()
            ))
            .to_uppercase(),
        );

        let area = center(
            area,
            Constraint::Length(max(
                clock_widget.get_width(
                    &state.get_clock().get_format(),
                    state.get_clock().with_decis,
                ),
                label.width() as u16,
            )),
            Constraint::Length(clock_widget.get_height() + 1 /* height of mode_str */),
        );

        let [v1, v2] =
            Layout::vertical(Constraint::from_lengths([clock_widget.get_height(), 1])).areas(area);

        clock_widget.render(v1, buf, state.get_clock_mut());
        label.centered().render(v2, buf);
    }
}
