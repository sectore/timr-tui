use crate::{
    events::{Event, EventHandler},
    utils::center,
    widgets::clock::{self, Clock, ClockWidget, Style},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use std::cmp::max;

#[derive(Debug, Clone)]
pub struct Timer {
    clock: Clock<clock::Timer>,
}

impl Timer {
    pub const fn new(clock: Clock<clock::Timer>) -> Self {
        Self { clock }
    }

    pub fn set_style(&mut self, style: Style) {
        self.clock.style = style;
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock.with_decis = with_decis;
    }

    pub fn get_clock(&self) -> &Clock<clock::Timer> {
        &self.clock
    }
}

impl EventHandler for Timer {
    fn update(&mut self, event: Event) -> Option<Event> {
        let edit_mode = self.clock.is_edit_mode();
        match event {
            Event::Tick => {
                self.clock.tick();
            }
            Event::Key(key) => match key.code {
                KeyCode::Char('s') => {
                    self.clock.toggle_pause();
                }
                KeyCode::Char('r') => {
                    self.clock.reset();
                }
                KeyCode::Char('e') => {
                    self.clock.toggle_edit();
                }
                KeyCode::Left if edit_mode => {
                    self.clock.edit_next();
                }
                KeyCode::Right if edit_mode => {
                    self.clock.edit_prev();
                }
                KeyCode::Up if edit_mode => {
                    self.clock.edit_up();
                }
                KeyCode::Down if edit_mode => {
                    self.clock.edit_down();
                }
                _ => return Some(event),
            },
            _ => return Some(event),
        }
        None
    }
}

pub struct TimerWidget;

impl StatefulWidget for &TimerWidget {
    type State = Timer;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock = &mut state.clock;
        let clock_widget = ClockWidget::new();
        let label = Line::raw((format!("Timer {}", clock.get_mode())).to_uppercase());

        let area = center(
            area,
            Constraint::Length(max(
                clock_widget.get_width(&clock.get_format(), clock.with_decis),
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
