use crate::{
    common::Style,
    events::{TuiEvent, TuiEventHandler},
    utils::center,
    widgets::clock::{self, ClockState, ClockWidget},
};
use crossterm::event::KeyModifiers;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use std::cmp::max;

pub struct TimerState {
    clock: ClockState<clock::Timer>,
}

impl TimerState {
    pub const fn new(clock: ClockState<clock::Timer>) -> Self {
        Self { clock }
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock.with_decis = with_decis;
    }

    pub fn get_clock(&self) -> &ClockState<clock::Timer> {
        &self.clock
    }
}

impl TuiEventHandler for TimerState {
    fn update(&mut self, event: TuiEvent) -> Option<TuiEvent> {
        let edit_mode = self.clock.is_edit_mode();
        match event {
            TuiEvent::Tick => {
                self.clock.tick();
                self.clock.update_done_count();
            }
            // EDIT mode
            TuiEvent::Key(key) if edit_mode => match key.code {
                // Skip changes
                KeyCode::Esc => {
                    // Important: set current value first
                    self.clock.set_current_value(*self.clock.get_prev_value());
                    // before toggling back to non-edit mode
                    self.clock.toggle_edit();
                }
                // Apply changes
                KeyCode::Char('s') => {
                    self.clock.toggle_edit();
                }
                // move change position to the left
                KeyCode::Left => {
                    self.clock.edit_next();
                }
                // move change position to the right
                KeyCode::Right => {
                    self.clock.edit_prev();
                }
                KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.clock.edit_jump_up();
                }
                // change value up
                KeyCode::Up => {
                    self.clock.edit_up();
                }
                // change value down
                KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.clock.edit_jump_down();
                }
                KeyCode::Down => {
                    self.clock.edit_down();
                }
                _ => return Some(event),
            },
            // default mode
            TuiEvent::Key(key) => match key.code {
                // Toggle run/pause
                KeyCode::Char('s') => {
                    self.clock.toggle_pause();
                }
                // reset clock
                KeyCode::Char('r') => {
                    self.clock.reset();
                }
                // enter edit mode
                KeyCode::Char('e') => {
                    self.clock.toggle_edit();
                }
                _ => return Some(event),
            },
            _ => return Some(event),
        }
        None
    }
}

pub struct Timer {
    pub style: Style,
    pub blink: bool,
}

impl StatefulWidget for Timer {
    type State = TimerState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock = &mut state.clock;
        let clock_widget = ClockWidget::new(self.style, self.blink);
        let label = Line::raw((format!("Timer {}", clock.get_mode())).to_uppercase());

        let area = center(
            area,
            Constraint::Length(max(
                clock_widget.get_width(clock.get_format(), clock.with_decis),
                label.width() as u16,
            )),
            Constraint::Length(clock_widget.get_height() + 1 /* height of label */),
        );
        let [v1, v2] =
            Layout::vertical(Constraint::from_lengths([clock_widget.get_height(), 1])).areas(area);

        clock_widget.render(v1, buf, clock);
        label.centered().render(v2, buf);
    }
}
