use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use std::{cmp::max, time::Duration};

use crate::{
    common::Style,
    constants::TICK_VALUE_MS,
    duration::DurationEx,
    events::{Event, EventHandler},
    utils::center,
    widgets::clock::{self, ClockState, ClockStateArgs, ClockWidget, Mode as ClockMode},
};

/// State for Countdown Widget
#[derive(Debug, Clone)]
pub struct CountdownState {
    /// clock to count down
    clock: ClockState<clock::Countdown>,
    /// clock to count time after `DONE` - similar to Mission Elapsed Time (MET)
    elapsed_clock: ClockState<clock::Timer>,
}

impl CountdownState {
    pub fn new(clock: ClockState<clock::Countdown>, elapsed_value: Duration) -> Self {
        Self {
            clock,
            elapsed_clock: ClockState::<clock::Timer>::new(ClockStateArgs {
                initial_value: Duration::ZERO,
                current_value: elapsed_value,
                tick_value: Duration::from_millis(TICK_VALUE_MS),
                with_decis: false,
            })
            // A previous `elapsed_value > 0` means the `Clock` was running before,
            // but not in `Initial` state anymore. Updating `Mode` here
            // is needed to handle `Event::Tick` in `EventHandler::update` properly
            .with_mode(if elapsed_value.gt(&Duration::ZERO) {
                ClockMode::Pause
            } else {
                ClockMode::Initial
            }),
        }
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock.with_decis = with_decis;
        self.elapsed_clock.with_decis = with_decis;
    }

    pub fn get_clock(&self) -> &ClockState<clock::Countdown> {
        &self.clock
    }

    pub fn is_running(&self) -> bool {
        self.clock.is_running() || self.elapsed_clock.is_running()
    }

    pub fn get_elapsed_value(&self) -> &DurationEx {
        self.elapsed_clock.get_current_value()
    }
}

impl EventHandler for CountdownState {
    fn update(&mut self, event: Event) -> Option<Event> {
        let edit_mode = self.clock.is_edit_mode();
        match event {
            Event::Tick => {
                if !self.clock.is_done() {
                    self.clock.tick();
                } else {
                    self.elapsed_clock.tick();
                    if self.elapsed_clock.is_initial() {
                        self.elapsed_clock.run();
                    }
                }
            }
            Event::Key(key) => match key.code {
                KeyCode::Char('r') => {
                    // reset both clocks
                    self.clock.reset();
                    self.elapsed_clock.reset();
                }
                KeyCode::Char('s') => {
                    // toggle pause status depending on which clock is running
                    if !self.clock.is_done() {
                        self.clock.toggle_pause();
                    } else {
                        self.elapsed_clock.toggle_pause();
                    }
                }
                KeyCode::Char('e') => {
                    self.clock.toggle_edit();
                    // stop + reset timer entering `edit` mode
                    if self.elapsed_clock.is_running() {
                        self.elapsed_clock.toggle_pause();
                    }
                }
                KeyCode::Left if edit_mode => {
                    self.clock.edit_next();
                }
                KeyCode::Right if edit_mode => {
                    self.clock.edit_prev();
                }
                KeyCode::Up if edit_mode => {
                    self.clock.edit_up();
                    // whenever `clock`'s value is changed, reset `elapsed_clock`
                    self.elapsed_clock.reset();
                }
                KeyCode::Down if edit_mode => {
                    self.clock.edit_down();
                    // whenever clock value is changed, reset timer
                    self.elapsed_clock.reset();
                }
                _ => return Some(event),
            },
            _ => return Some(event),
        }
        None
    }
}

pub struct Countdown {
    pub style: Style,
}

impl StatefulWidget for Countdown {
    type State = CountdownState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock = ClockWidget::new(self.style);

        let label = Line::raw(
            if state.clock.is_done() {
                if state.clock.with_decis {
                    format!(
                        "Countdown {} +{}",
                        state.clock.get_mode(),
                        state
                            .elapsed_clock
                            .get_current_value()
                            .to_string_with_decis()
                    )
                } else {
                    format!(
                        "Countdown {} +{}",
                        state.clock.get_mode(),
                        state.elapsed_clock.get_current_value()
                    )
                }
            } else {
                format!("Countdown {}", state.clock.get_mode())
            }
            .to_uppercase(),
        );

        let area = center(
            area,
            Constraint::Length(max(
                clock.get_width(&state.clock.get_format(), state.clock.with_decis),
                label.width() as u16,
            )),
            Constraint::Length(clock.get_height() + 1 /* height of label */),
        );
        let [v1, v2] =
            Layout::vertical(Constraint::from_lengths([clock.get_height(), 1])).areas(area);

        clock.render(v1, buf, &mut state.clock);
        label.centered().render(v2, buf);
    }
}
