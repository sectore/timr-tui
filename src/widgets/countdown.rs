use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use std::cmp::max;

use crate::{
    args::ClockStyle,
    events::{Event, EventHandler},
    utils::center,
    widgets::clock::{self, Clock, ClockWidget},
};

#[derive(Debug, Clone)]
pub struct Countdown {
    clock: Clock<clock::Countdown>,
}

impl Countdown {
    pub const fn new(clock: Clock<clock::Countdown>) -> Self {
        Self { clock }
    }

    pub fn set_style(&mut self, style: ClockStyle) {
        self.clock.style = style;
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock.with_decis = with_decis;
    }

    pub fn get_clock(&self) -> &Clock<clock::Countdown> {
        &self.clock
    }
}

impl EventHandler for Countdown {
    fn update(&mut self, event: Event) -> Option<Event> {
        let edit_mode = self.clock.is_edit_mode();
        match event {
            Event::Tick => {
                self.clock.tick();
            }
            Event::Key(key) if key.code == KeyCode::Char('r') => {
                self.clock.reset();
            }
            Event::Key(key) => match key.code {
                KeyCode::Char('r') => {
                    self.clock.reset();
                }
                KeyCode::Char('s') => {
                    self.clock.toggle_pause();
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

pub struct CountdownWidget;

impl StatefulWidget for CountdownWidget {
    type State = Countdown;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock = ClockWidget::new();
        let label = Line::raw((format!("Countdown {}", state.clock.get_mode())).to_uppercase());

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
