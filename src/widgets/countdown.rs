use crate::{
    common::{AppTime, Style},
    constants::TICK_VALUE_MS,
    duration::DurationEx,
    events::{Event, EventHandler},
    utils::center,
    widgets::{
        clock::{self, ClockState, ClockStateArgs, ClockWidget, Mode as ClockMode},
        edit_time::EditTimeState,
    },
};
use crossterm::event::KeyModifiers;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use std::ops::Add;
use std::{cmp::max, time::Duration};
use time::OffsetDateTime;

use super::edit_time::EditTimeWidget;

/// State for Countdown Widget
#[derive(Debug, Clone)]
pub struct CountdownState {
    /// clock to count down
    clock: ClockState<clock::Countdown>,
    /// clock to count time after `DONE` - similar to Mission Elapsed Time (MET)
    elapsed_clock: ClockState<clock::Timer>,
    app_time: AppTime,
    /// Edit by local time
    edit_time: Option<EditTimeState>,
}

impl CountdownState {
    pub fn new(
        clock: ClockState<clock::Countdown>,
        elapsed_value: Duration,
        app_time: AppTime,
    ) -> Self {
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
            app_time,
            edit_time: None,
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

    pub fn set_app_time(&mut self, app_time: AppTime) {
        self.app_time = app_time;
    }
}

impl EventHandler for CountdownState {
    fn update(&mut self, event: Event) -> Option<Event> {
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
                    // Countdown values can by edited in 2 ways:
                    // (1) by local time
                    // (2) by countdown value
                    //
                    // Keys `STRG + e` => (1)
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        // toggle edit mode
                        if self.edit_time.is_some() {
                            self.edit_time = None;
                        } else {
                            let d: Duration = (*self.clock.get_current_value()).into();
                            let time: OffsetDateTime = OffsetDateTime::from(self.app_time).add(d);
                            self.edit_time = Some(EditTimeState::new(time));
                        }
                        // stop `clock`
                        if self.clock.is_running() {
                            self.clock.toggle_pause();
                        }
                    }
                    // Key `e` => (2)
                    else {
                        // toggle edit mode
                        self.clock.toggle_edit();
                    }

                    // stop `elapsed_clock` in both cases (1) + (2)
                    if self.elapsed_clock.is_running() {
                        self.elapsed_clock.toggle_pause();
                    }
                }
                KeyCode::Left if self.clock.is_edit_mode() => {
                    self.clock.edit_next();
                }
                KeyCode::Left if self.edit_time.is_some() => {
                    self.edit_time.as_mut().unwrap().next();
                }
                KeyCode::Right if self.clock.is_edit_mode() => {
                    self.clock.edit_prev();
                }
                KeyCode::Right if self.edit_time.is_some() => {
                    self.edit_time.as_mut().unwrap().prev();
                }
                KeyCode::Up if self.clock.is_edit_mode() => {
                    self.clock.edit_up();
                    // whenever `clock`'s value is changed, reset `elapsed_clock`
                    self.elapsed_clock.reset();
                }
                KeyCode::Down if self.clock.is_edit_mode() => {
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

        // render `edit_time` OR `clock`
        if let Some(edit_time) = &mut state.edit_time {
            let widget = EditTimeWidget::new(self.style);
            let area = center(
                area,
                Constraint::Length(max(widget.get_width(), label.width() as u16)),
                Constraint::Length(widget.get_height() + 1 /* height of label */),
            );
            let [v1, v2] =
                Layout::vertical(Constraint::from_lengths([widget.get_height(), 1])).areas(area);

            widget.render(v1, buf, edit_time);
            label.centered().render(v2, buf);
        } else {
            let widget = ClockWidget::new(self.style);
            let area = center(
                area,
                Constraint::Length(max(
                    widget.get_width(&state.clock.get_format(), state.clock.with_decis),
                    label.width() as u16,
                )),
                Constraint::Length(widget.get_height() + 1 /* height of label */),
            );
            let [v1, v2] =
                Layout::vertical(Constraint::from_lengths([widget.get_height(), 1])).areas(area);

            widget.render(v1, buf, &mut state.clock);
            label.centered().render(v2, buf);
        }
    }
}
