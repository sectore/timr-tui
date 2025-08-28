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
        COLON_WIDTH, Colon, DIGIT_HEIGHT, DIGIT_SPACE_WIDTH, DIGIT_WIDTH, DOT_WIDTH, Digit, Dot,
    },
};
use std::cmp::max;

/// State for `LocalTimeWidget`
pub struct LocalTimeState {
    with_decis: bool,
    time: AppTime,
    format: AppTimeFormat,
}

pub struct LocalTimeStateArgs {
    pub app_time: AppTime,
    pub app_time_format: AppTimeFormat,
    pub with_decis: bool,
}

impl LocalTimeState {
    pub fn new(args: LocalTimeStateArgs) -> Self {
        let LocalTimeStateArgs {
            app_time,
            app_time_format,
            with_decis,
        } = args;

        Self {
            time: app_time,
            format: app_time_format,
            with_decis,
        }
    }

    pub fn set_app_time(&mut self, app_time: AppTime) {
        self.time = app_time;
    }

    pub fn set_app_time_format(&mut self, format: AppTimeFormat) {
        self.format = format;
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.with_decis = with_decis;
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
    fn get_horizontal_lengths(&self, format: &AppTimeFormat, with_decis: bool) -> Vec<u16> {
        match format {
            AppTimeFormat::HhMmSs => {
                let mut lengths = vec![
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
                ];
                if with_decis {
                    lengths.extend_from_slice(&[
                        DOT_WIDTH,   // .
                        DIGIT_WIDTH, // ds
                    ])
                }
                lengths
            }
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
        let label = Line::raw("Local Time".to_uppercase());

        let with_decis = state.with_decis;
        let format = state.format;
        let widths = self.get_horizontal_lengths(&format, with_decis);
        let width = widths.iter().sum();
        let area = center(
            area,
            Constraint::Length(max(width, label.width() as u16)),
            Constraint::Length(DIGIT_HEIGHT + 1 /* height of label */),
        );

        let [v1, v2] = Layout::vertical(Constraint::from_lengths([DIGIT_HEIGHT, 1])).areas(area);

        let current_value: DurationEx = state.time.as_duration_of_today().into();
        let hours = current_value.hours();
        let minutes = current_value.minutes_mod();
        let seconds = current_value.seconds_mod();
        let decis = current_value.decis();
        let symbol = self.style.get_digit_symbol();
        match state.format {
            AppTimeFormat::HhMmSs if state.with_decis => {
                let [hh, _, h, c_hm, mm, _, m, c_ms, ss, _, s, d, ds] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(v1);
                Digit::new(hours / 10, false, symbol).render(hh, buf);
                Digit::new(hours % 10, false, symbol).render(h, buf);
                Colon::new(symbol).render(c_hm, buf);
                Digit::new(minutes / 10, false, symbol).render(mm, buf);
                Digit::new(minutes % 10, false, symbol).render(m, buf);
                Colon::new(symbol).render(c_ms, buf);
                Digit::new(seconds / 10, false, symbol).render(ss, buf);
                Digit::new(seconds % 10, false, symbol).render(s, buf);
                Dot::new(symbol).render(d, buf);
                Digit::new(decis, false, symbol).render(ds, buf);
            }
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
                let [h, c_hm, mm, _, m, _, p] =
                    Layout::horizontal(Constraint::from_lengths(widths)).areas(v1);
                Digit::new((hours - 1) % 12 + 1, false, symbol).render(h, buf);
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
