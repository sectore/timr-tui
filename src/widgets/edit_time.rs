use time::OffsetDateTime;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    common::Style,
    widgets::clock_elements::{Colon, Digit, COLON_WIDTH, DIGIT_SPACE_WIDTH, DIGIT_WIDTH},
};

use super::clock_elements::DIGIT_HEIGHT;

#[derive(Debug, Clone)]
enum Selected {
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

#[derive(Debug, Clone)]
pub struct EditTimeState {
    selected: Selected,
    time: OffsetDateTime,
}

impl EditTimeState {
    pub fn new(time: OffsetDateTime) -> Self {
        EditTimeState {
            time,
            selected: Selected::Minutes,
        }
    }

    pub fn next(&mut self) {
        self.selected = self.selected.next();
    }

    pub fn prev(&mut self) {
        self.selected = self.selected.prev();
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
