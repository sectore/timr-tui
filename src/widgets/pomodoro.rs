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

use super::clock::Style;

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

pub struct PomodoroArgs {
    pub work: Duration,
    pub pause: Duration,
}

impl Pomodoro {
    pub fn new(args: PomodoroArgs) -> Self {
        Self {
            mode: Mode::Work,
            clock_map: ClockMap {
                work: Clock::<Countdown>::new(args.work, Duration::from_millis(TICK_VALUE_MS)),
                pause: Clock::<Countdown>::new(args.pause, Duration::from_millis(TICK_VALUE_MS)),
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
}

impl EventHandler for Pomodoro {
    fn update(&mut self, event: Event) -> Option<Event> {
        let edit_mode = self.get_clock().is_edit_mode();
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
                KeyCode::Left if edit_mode => {
                    self.get_clock().edit_next();
                }
                KeyCode::Left => {
                    // `next` is acting as same as a `prev` function, we don't have
                    self.next();
                }
                KeyCode::Right if edit_mode => {
                    self.get_clock().edit_prev();
                }
                KeyCode::Right => {
                    self.next();
                }
                KeyCode::Up if edit_mode => {
                    self.get_clock().edit_up();
                }
                KeyCode::Down if edit_mode => {
                    self.get_clock().edit_down();
                }
                KeyCode::Char('r') => {
                    self.get_clock().reset();
                }
                _ => return Some(event),
            },
            _ => return Some(event),
        }
        None
    }
}

pub struct PomodoroWidget;

impl StatefulWidget for PomodoroWidget {
    type State = (Style, Pomodoro);
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let style = state.0;
        let state = &mut state.1;
        let clock_widget = ClockWidget::new(style);
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
            Constraint::Length(max(
                clock_widget.get_width(&state.get_clock().get_format()),
                label.width() as u16,
            )),
            Constraint::Length(clock_widget.get_height() + 1 /* height of mode_str */),
        );

        let [v1, v2] =
            Layout::vertical(Constraint::from_lengths([clock_widget.get_height(), 1])).areas(area);

        clock_widget.render(v1, buf, state.get_clock());
        label.centered().render(v2, buf);
    }
}
