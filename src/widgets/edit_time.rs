use std::fmt;
use time::OffsetDateTime;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    common::Style,
    widgets::clock_elements::{COLON_WIDTH, Colon, DIGIT_SPACE_WIDTH, DIGIT_WIDTH, Digit},
};

use super::clock_elements::DIGIT_HEIGHT;

#[derive(Debug, Clone)]
pub enum Selected {
    Seconds,
    Minutes,
    Hours,
}

impl Selected {
    pub fn next(&self) -> Self {
        match self {
            Selected::Seconds => Selected::Minutes,
            Selected::Minutes => Selected::Hours,
            Selected::Hours => Selected::Seconds,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Selected::Seconds => Selected::Hours,
            Selected::Minutes => Selected::Seconds,
            Selected::Hours => Selected::Minutes,
        }
    }
}

impl fmt::Display for Selected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Selected::Seconds => write!(f, "[edit seconds]"),
            Selected::Minutes => write!(f, "[edit minutes]"),
            Selected::Hours => write!(f, "[edit hours]"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditTimeState {
    selected: Selected,
    time: OffsetDateTime,
    min: OffsetDateTime,
    max: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct EditTimeStateArgs {
    pub time: OffsetDateTime,
    pub min: OffsetDateTime,
    pub max: OffsetDateTime,
}

impl EditTimeState {
    pub fn new(args: EditTimeStateArgs) -> Self {
        EditTimeState {
            time: args.time,
            min: args.min,
            max: args.max,
            selected: Selected::Minutes,
        }
    }

    pub fn set_time(&mut self, time: OffsetDateTime) {
        self.time = time;
    }

    pub fn set_min_time(&mut self, min: OffsetDateTime) {
        self.min = min;
    }

    pub fn set_max_time(&mut self, min: OffsetDateTime) {
        self.max = min;
    }

    pub fn get_time(&mut self) -> &OffsetDateTime {
        &self.time
    }

    pub fn get_selected(&mut self) -> &Selected {
        &self.selected
    }

    pub fn next(&mut self) {
        self.selected = self.selected.next();
    }

    pub fn prev(&mut self) {
        self.selected = self.selected.prev();
    }

    fn up_by(&mut self, times: i64) {
        let seconds = match self.selected {
            Selected::Seconds => times,
            Selected::Minutes => 60 * times,
            Selected::Hours => 60 * 60 * times,
        };

        let delta = time::Duration::new(seconds, 0);

        if self.time.lt(&self.max.saturating_sub(delta)) {
            self.time = self.time.saturating_add(delta);
        }
    }

    pub fn up(&mut self) {
        self.up_by(1);
    }

    pub fn jump_up(&mut self) {
        self.up_by(10);
    }

    fn down_by(&mut self, times: i64) {
        let seconds = match self.selected {
            Selected::Seconds => times,
            Selected::Minutes => 60 * times,
            Selected::Hours => 60 * 60 * times,
        };

        let delta = time::Duration::new(seconds, 0);

        if self.time.ge(&self.min.saturating_add(delta)) {
            self.time = self.time.saturating_sub(delta);
        }
    }

    pub fn down(&mut self) {
        self.down_by(1);
    }

    pub fn jump_down(&mut self) {
        self.down_by(10);
    }
}

#[derive(Debug, Clone)]
pub struct EditTimeWidget {
    style: Style,
}

impl EditTimeWidget {
    pub fn new(style: Style) -> Self {
        Self { style }
    }

    fn get_horizontal_lengths(&self) -> Vec<u16> {
        vec![
            DIGIT_WIDTH,       // h
            DIGIT_SPACE_WIDTH, // (space)
            DIGIT_WIDTH,       // h
            COLON_WIDTH,       // :
            DIGIT_WIDTH,       // m
            DIGIT_SPACE_WIDTH, // (space)
            DIGIT_WIDTH,       // m
            COLON_WIDTH,       // :
            DIGIT_WIDTH,       // s
            DIGIT_SPACE_WIDTH, // (space)
            DIGIT_WIDTH,       // s
        ]
    }

    pub fn get_width(&self) -> u16 {
        self.get_horizontal_lengths().iter().sum()
    }

    pub fn get_height(&self) -> u16 {
        DIGIT_HEIGHT
    }
}

impl StatefulWidget for EditTimeWidget {
    type State = EditTimeState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let symbol = self.style.get_digit_symbol();
        let edit_hours = matches!(state.selected, Selected::Hours);
        let edit_minutes = matches!(state.selected, Selected::Minutes);
        let edit_secs = matches!(state.selected, Selected::Seconds);

        let [hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s] =
            Layout::horizontal(Constraint::from_lengths(self.get_horizontal_lengths())).areas(area);

        Digit::new((state.time.hour() as u64) / 10, edit_hours, symbol).render(hh, buf);
        Digit::new((state.time.hour() as u64) % 10, edit_hours, symbol).render(h, buf);
        Colon::new(symbol).render(c_hm, buf);
        Digit::new((state.time.minute() as u64) / 10, edit_minutes, symbol).render(mm, buf);
        Digit::new((state.time.minute() as u64) % 10, edit_minutes, symbol).render(m, buf);
        Colon::new(symbol).render(c_ms, buf);
        Digit::new((state.time.second() as u64) / 10, edit_secs, symbol).render(ss, buf);
        Digit::new((state.time.second() as u64) % 10, edit_secs, symbol).render(s, buf);
    }
}
