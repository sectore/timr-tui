use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use time::{OffsetDateTime, macros::format_description};

use crate::{
    common::{AppTime, ClockTypeId, Style},
    duration::CalendarDuration,
    event::Event,
    events::{AppEvent, AppEventTx, TuiEvent, TuiEventHandler},
    utils::center,
    widgets::{clock, clock_elements::DIGIT_HEIGHT},
};
use std::{cmp::max, time::Duration};

/// State for `EventWidget`
pub struct EventState {
    title: Option<String>,
    event_time: OffsetDateTime,
    app_time: OffsetDateTime,
    start_time: OffsetDateTime,
    with_decis: bool,
    /// counter to simulate `DONE` state
    /// Default value: `None`
    done_count: Option<u64>,
    app_tx: AppEventTx,
}

pub struct EventStateArgs {
    pub app_time: AppTime,
    pub event: Event,
    pub with_decis: bool,
    pub app_tx: AppEventTx,
}

impl EventState {
    pub fn new(args: EventStateArgs) -> Self {
        let EventStateArgs {
            app_time,
            event,
            with_decis,
            app_tx,
        } = args;

        let app_datetime = OffsetDateTime::from(app_time);
        // assume event has as same `offset` as `app_time`
        let event_offset = event.date_time.assume_offset(app_datetime.offset());

        Self {
            title: event.title,
            event_time: event_offset,
            app_time: app_datetime,
            start_time: app_datetime,
            with_decis,
            done_count: None,
            app_tx,
        }
    }

    // Sets `app_time`
    pub fn set_app_time(&mut self, app_time: AppTime) {
        let app_datetime = OffsetDateTime::from(app_time);
        self.app_time = app_datetime;

        // Since updating `app_time` is like a `Tick`, we check `done` state here
        self.check_done();
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.with_decis = with_decis;
    }

    pub fn get_percentage_done(&self) -> u16 {
        get_percentage(self.start_time, self.event_time, self.app_time)
    }

    pub fn get_duration(&mut self) -> CalendarDuration {
        CalendarDuration::from_start_end_times(self.event_time, self.app_time)
    }

    fn check_done(&mut self) {
        let clock_duration = self.get_duration();
        if clock_duration.is_since() {
            let duration: Duration = clock_duration.into();
            // give some offset to make sure we are around `Duration::ZERO`
            // Without that we might miss it, because the app runs on its own FPS
            if duration < Duration::from_millis(100) {
                // reset `done_count`
                self.done_count = Some(clock::MAX_DONE_COUNT);
                // send notification
                _ = self.app_tx.send(AppEvent::ClockDone(
                    ClockTypeId::Event,
                    self.title.clone().unwrap_or("".into()),
                ));
            }
            // count (possible) `done`
            self.done_count = clock::count_clock_done(self.done_count);
        }
    }
}

impl TuiEventHandler for EventState {
    fn update(&mut self, event: TuiEvent) -> Option<TuiEvent> {
        Some(event)
    }
}

fn get_percentage(start: OffsetDateTime, end: OffsetDateTime, current: OffsetDateTime) -> u16 {
    let total_millis = (end - start).whole_milliseconds();

    if total_millis <= 0 {
        return 100;
    }

    let elapsed_millis = (current - start).whole_milliseconds();

    if elapsed_millis <= 0 {
        return 0;
    }

    let percentage = (elapsed_millis * 100 / total_millis).min(100);
    percentage as u16
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
        let clock_duration = state.get_duration();
        let clock_format = clock::format_by_duration(&clock_duration);
        let clock_widths = clock::clock_horizontal_lengths(&clock_format, with_decis);
        let clock_width = clock_widths.iter().sum();

        let label_event = Line::raw(state.title.clone().unwrap_or("".into()).to_uppercase());
        let time_str = state
            .event_time
            .format(&format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .unwrap_or_else(|e| format!("time format error: {}", e));
        let time_prefix = if clock_duration.is_since() {
            let duration: Duration = clock_duration.clone().into();
            // Show `done` for a short of time (1 sec)
            if duration < Duration::from_secs(1) {
                "Done"
            } else {
                "Since"
            }
        } else {
            "Until"
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

        // To simulate a blink effect, just use an "empty" symbol (string)
        // It's "empty" all digits and creates an "empty" render area
        let symbol = if self.blink && clock::should_blink(state.done_count) {
            " "
        } else {
            self.style.get_digit_symbol()
        };

        let render_clock_state = clock::RenderClockState {
            with_decis,
            duration: clock_duration,
            editable_time: None,
            format: clock_format,
            symbol,
            widths: clock_widths,
        };

        clock::render_clock(v1, buf, render_clock_state);
        label_time.centered().render(v2, buf);
        label_event.centered().render(v3, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_get_percentage() {
        let start = datetime!(2024-01-01 10:00:00 UTC);
        let end = datetime!(2024-01-01 20:00:00 UTC);

        // current == start: 0%
        assert_eq!(get_percentage(start, end, start), 0);

        // current == end: 100%
        assert_eq!(get_percentage(start, end, end), 100);

        // current halfway: 50%
        let halfway = datetime!(2024-01-01 15:00:00 UTC);
        assert_eq!(get_percentage(start, end, halfway), 50);

        // current 25%
        let quarter = datetime!(2024-01-01 12:30:00 UTC);
        assert_eq!(get_percentage(start, end, quarter), 25);

        // current 75%
        let three_quarters = datetime!(2024-01-01 17:30:00 UTC);
        assert_eq!(get_percentage(start, end, three_quarters), 75);

        // current > end: clamped to 100%
        let after = datetime!(2024-01-01 22:00:00 UTC);
        assert_eq!(get_percentage(start, end, after), 100);

        // current < start: 0%
        let before = datetime!(2024-01-01 08:00:00 UTC);
        assert_eq!(get_percentage(start, end, before), 0);

        // end <= start: 100%
        assert_eq!(get_percentage(end, start, halfway), 100);
        assert_eq!(get_percentage(start, start, start), 100);
    }
}
