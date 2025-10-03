use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use time::{OffsetDateTime, macros::format_description};

use crate::{
    common::{AppTime, Style},
    constants::TICK_VALUE_MS,
    events::{AppEventTx, TuiEvent, TuiEventHandler},
    utils::center,
    widgets::clock::{self, ClockState, ClockStateArgs, ClockWidget},
};
use std::{cmp::max, time::Duration};

/// State for `EventWidget`
pub struct EventState {
    title: String,
    event_time: time::PrimitiveDateTime,
    // TODO(#105) `Timer` or `Countdown`
    clock: ClockState<clock::Countdown>,
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

        let t = event_time.time();
        // TODO(#105) Extract into `ExDuration` ??
        let initial_value = Duration::new(
            t.hour() as u64 * 3600 + t.minute() as u64 * 60 + t.second() as u64,
            t.nanosecond(),
        );
        let app_datetime = OffsetDateTime::from(app_time);
        // assume `offset` as same as `app_time`
        let event_offset = event_time.assume_offset(app_datetime.offset());

        let current_value = (event_offset - app_datetime).unsigned_abs();

        // TODO(#105) `Timer` or `Countdown`
        let clock = ClockState::<clock::Countdown>::new(ClockStateArgs {
            initial_value,
            current_value,
            tick_value: Duration::from_millis(TICK_VALUE_MS),
            with_decis,
            app_tx: Some(app_tx.clone()),
        });

        Self {
            title: event_title,
            event_time,
            clock,
        }
    }

    pub fn get_clock(&self) -> &ClockState<clock::Countdown> {
        &self.clock
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
        let clock = &mut state.clock;
        let clock_widget = ClockWidget::new(self.style, self.blink);
        let label_event = Line::raw(state.title.to_uppercase());
        let time_str = state
            .event_time
            .format(&format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .unwrap_or_else(|e| format!("time format error: {}", e));
        let label_time = Line::raw(time_str.to_uppercase());
        let max_label_width = max(label_event.width(), label_time.width()) as u16;

        let area = center(
            area,
            Constraint::Length(max(
                clock_widget.get_width(clock.get_format(), clock.with_decis),
                max_label_width,
            )),
            Constraint::Length(clock_widget.get_height() + 3 /* height of label */),
        );
        let [_, v1, v2, v3] = Layout::vertical(Constraint::from_lengths([
            1, // empty (offset) to keep everything centered vertically comparing to "clock" widgets with one label only
            clock_widget.get_height(),
            1, // event date
            1, // event title
        ]))
        .areas(area);

        clock_widget.render(v1, buf, clock);
        label_time.centered().render(v2, buf);
        label_event.centered().render(v3, buf);
    }
}
