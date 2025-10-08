use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use time::{OffsetDateTime, macros::format_description};

use crate::{
    common::{AppTime, Style},
    duration::{CalendarDuration, DirectedDuration},
    events::{AppEventTx, TuiEvent, TuiEventHandler},
    utils::center,
    widgets::clock,
    widgets::clock_elements::DIGIT_HEIGHT,
};
use std::{cmp::max, time::Duration};

/// State for `EventWidget`
pub struct EventState {
    title: String,
    event_time: OffsetDateTime,
    app_time: OffsetDateTime,
    calendar_duration: CalendarDuration,
    directed_duration: DirectedDuration,
    with_decis: bool,
}

pub struct EventStateArgs {
    pub app_time: AppTime,
    pub event_time: time::PrimitiveDateTime,
    pub event_title: String,
    pub with_decis: bool,
    pub app_tx: AppEventTx,
}

impl EventState {
    pub fn new(args: EventStateArgs) -> Self {
        let EventStateArgs {
            app_time,
            event_time,
            event_title,
            with_decis,
            app_tx,
        } = args;

        // TODO: Handle app Events
        let _ = app_tx;
        let app_datetime = OffsetDateTime::from(app_time);
        // assume event has as same `offset` as `app_time`
        let event_offset = event_time.assume_offset(app_datetime.offset());

        // Create calendar-aware duration (accounts for leap years!)
        let calendar_duration = CalendarDuration::between(event_offset, app_datetime);

        // Also keep DirectedDuration for "Since" vs "Until" logic
        let directed_duration =
            DirectedDuration::from_offset_date_times(event_offset, app_datetime);

        Self {
            title: event_title,
            event_time: event_offset,
            app_time: app_datetime,
            calendar_duration,
            directed_duration,
            with_decis,
        }
    }

    pub fn set_app_time(&mut self, app_time: AppTime) {
        let app_datetime = OffsetDateTime::from(app_time);
        self.app_time = app_datetime;

        // Update calendar-aware duration (accounts for leap years!)
        self.calendar_duration = CalendarDuration::between(self.event_time, app_datetime);

        // Update directed duration for "Since" vs "Until" logic
        self.directed_duration =
            DirectedDuration::from_offset_date_times(self.event_time, app_datetime);
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.with_decis = with_decis;
    }

    pub fn get_percentage_done(&self) -> u16 {
        match self.directed_duration {
            DirectedDuration::Since(_) => 100,
            // TODO: get percentage
            DirectedDuration::Until(_) => 22,
        }
    }
}

impl TuiEventHandler for EventState {
    fn update(&mut self, event: TuiEvent) -> Option<TuiEvent> {
        Some(event)
    }
}

#[derive(Debug)]
pub struct EventWidget {
    pub style: Style,
    pub blink: bool,
}

impl StatefulWidget for EventWidget {
    type State = EventState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let with_decis = state.with_decis;
        let duration = state.calendar_duration;
        let clock_format = clock::format_by_duration(&duration);
        let clock_widths = clock::clock_horizontal_lengths(&clock_format, with_decis);
        let clock_width = clock_widths.iter().sum();

        let label_event = Line::raw(state.title.to_uppercase());
        let time_str = state
            .event_time
            .format(&format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .unwrap_or_else(|e| format!("time format error: {}", e));
        let time_prefix = match state.directed_duration {
            DirectedDuration::Since(d) => {
                // Show `done` for a short of time (1 sec.)
                if d < Duration::from_secs(1) {
                    "Done"
                } else {
                    "Since"
                }
            }
            DirectedDuration::Until(_) => "Until",
        };
        let label_time = Line::raw(format!(
            "{} {}",
            time_prefix.to_uppercase(),
            time_str.to_uppercase()
        ));
        let max_label_width = max(label_event.width(), label_time.width()) as u16;

        let area = center(
            area,
            Constraint::Length(max(clock_width, max_label_width)),
            Constraint::Length(DIGIT_HEIGHT + 3 /* height of label */),
        );
        let [_, v1, v2, v3] = Layout::vertical(Constraint::from_lengths([
            1, // empty (offset) to keep everything centered vertically comparing to "clock" widgets with one label only
            DIGIT_HEIGHT,
            1, // event date
            1, // event title
        ]))
        .areas(area);

        // TODO: Add logic to handle blink in `DONE` mode, similar to `ClockWidget<T>::should_blink`
        let symbol = if self.blink {
            " "
        } else {
            self.style.get_digit_symbol()
        };

        let render_clock_state = clock::RenderClockState {
            with_decis,
            duration,
            // TODO: Should we track other modes (e.g. DONE)?
            mode: &clock::Mode::Tick,
            format: clock_format,
            symbol,
            widths: clock_widths,
        };

        clock::render_clock(v1, buf, render_clock_state);
        label_time.centered().render(v2, buf);
        label_event.centered().render(v3, buf);
    }
}
