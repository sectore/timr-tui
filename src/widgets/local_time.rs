use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
};

use crate::{
    common::{AppTime, AppTimeFormat, Style as DigitStyle},
    duration::DurationEx,
    events::{TuiEvent, TuiEventHandler},
    utils::center,
    widgets::clock_elements::{
        COLON_WIDTH, Colon, DIGIT_HEIGHT, DIGIT_SPACE_WIDTH, DIGIT_WIDTH, Digit,
    },
};
use std::cmp::max;

/// State for `LocalTimeWidget`
pub struct LocalTimeState {
    time: AppTime,
    format: AppTimeFormat,
}

pub struct LocalTimeStateArgs {
    pub app_time: AppTime,
    pub app_time_format: AppTimeFormat,
}

impl LocalTimeState {
    pub fn new(args: LocalTimeStateArgs) -> Self {
        let LocalTimeStateArgs {
            app_time,
            app_time_format,
        } = args;

        Self {
            time: app_time,
            format: app_time_format,
        }
    }

    pub fn set_app_time(&mut self, app_time: AppTime) {
        self.time = app_time;
    }

    pub fn set_app_time_format(&mut self, format: AppTimeFormat) {
        self.format = format;
    }
}

impl TuiEventHandler for LocalTimeState {
    fn update(&mut self, event: TuiEvent) -> Option<TuiEvent> {
        Some(event)
    }
}

#[derive(Debug)]
pub struct LocalTimeWidget {
    pub style: DigitStyle,
}

impl LocalTimeWidget {
    fn get_horizontal_lengths(&self, format: &AppTimeFormat) -> Vec<u16> {
        match format {
            AppTimeFormat::HhMmSs => vec![
                DIGIT_WIDTH,       // H
                DIGIT_SPACE_WIDTH, // (space)
                DIGIT_WIDTH,       // h
                COLON_WIDTH,       // :
                DIGIT_WIDTH,       // M
                DIGIT_SPACE_WIDTH, // (space)
                DIGIT_WIDTH,       // m
                COLON_WIDTH,       // :
                DIGIT_WIDTH,       // S
                DIGIT_SPACE_WIDTH, // (space)
                DIGIT_WIDTH,       // s
            ],
            AppTimeFormat::HhMm => vec![
                DIGIT_WIDTH,       // H
                DIGIT_SPACE_WIDTH, // (space)
                DIGIT_WIDTH,       // h
                COLON_WIDTH,       // :
                DIGIT_WIDTH,       // M
                DIGIT_SPACE_WIDTH, // (space)
                DIGIT_WIDTH,       // m
            ],
            AppTimeFormat::Hh12Mm => vec![
                DIGIT_WIDTH,       // H
                DIGIT_SPACE_WIDTH, // (space)
                DIGIT_WIDTH,       // h
                COLON_WIDTH,       // :
                DIGIT_WIDTH,       // M
                DIGIT_SPACE_WIDTH, // (space)
                DIGIT_WIDTH,       // m
                DIGIT_SPACE_WIDTH, // (space)
                2,                 // period (PM or AM)
            ],
        }
    }
}

impl StatefulWidget for LocalTimeWidget {
    type State = LocalTimeState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let current_value: DurationEx = state.time.as_duration_of_today().into();
        let hours = current_value.hours_mod();
        let minutes = current_value.minutes_mod();
        let seconds = current_value.seconds_mod();
        let symbol = self.style.get_digit_symbol();

        let label = Line::raw("Local Time".to_uppercase());

        let format = state.format;
        let widths = self.get_horizontal_lengths(&format);
        let width = widths.iter().sum();
        let area = center(
            area,
            Constraint::Length(max(width, label.width() as u16)),
            Constraint::Length(DIGIT_HEIGHT + 1 /* height of label */),
        );

        let [v1, v2] = Layout::vertical(Constraint::from_lengths([DIGIT_HEIGHT, 1])).areas(area);

        match state.format {
            AppTimeFormat::HhMmSs => {
                let [hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(v1);
                Digit::new(hours / 10, false, symbol).render(hh, buf);
                Digit::new(hours % 10, false, symbol).render(h, buf);
                Colon::new(symbol).render(c_hm, buf);
                Digit::new(minutes / 10, false, symbol).render(mm, buf);
                Digit::new(minutes % 10, false, symbol).render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(seconds / 10, false, symbol).render(ss, buf);
                Digit::new(seconds % 10, false, symbol).render(s, buf);
            }
            AppTimeFormat::HhMm => {
                let [hh, _, h, c_hm, mm, _, m] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(v1);
                Digit::new(hours / 10, false, symbol).render(hh, buf);
                Digit::new(hours % 10, false, symbol).render(h, buf);
                Colon::new(symbol).render(c_hm, buf);
                Digit::new(minutes / 10, false, symbol).render(mm, buf);
                Digit::new(minutes % 10, false, symbol).render(m, buf);
            }
            AppTimeFormat::Hh12Mm => {
                let hours12 = current_value.hours_mod_12();
                // Note: Format might be `h:Mm` OR `Hh:Mm`
                // depending on `hours12` having one OR two digits
                let mut widths = widths;
                if hours12 < 10 {
                    //we don't draw `H` and no space
                    widths[0] = 0; // `H`
                    widths[1] = 0; // `space`
                }
                let [hh, _, h, c_hm, mm, _, m, _, p] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(v1);
                // Hh
                if hours12 >= 10 {
                    Digit::new(hours12 / 10, false, symbol).render(hh, buf);
                    Digit::new(hours12 % 10, false, symbol).render(h, buf);
                }
                // h
                else {
                    Digit::new(hours12, false, symbol).render(h, buf);
                }
                Colon::new(symbol).render(c_hm, buf);
                Digit::new(minutes / 10, false, symbol).render(mm, buf);
                Digit::new(minutes % 10, false, symbol).render(m, buf);
                Span::styled(
                    state.time.get_period().to_uppercase(),
                    Style::default().add_modifier(Modifier::BOLD),
                )
                .render(p, buf);
            }
        }
        label.centered().render(v2, buf);
    }
}
