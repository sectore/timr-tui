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
    duration::DirectedDuration,
    events::{AppEventTx, TuiEvent, TuiEventHandler},
    utils::center,
    widgets::clock::{self, ClockState, ClockStateArgs, ClockWidget},
};
use std::{cmp::max, time::Duration};

/// State for `EventWidget`
pub struct EventState {
    title: String,
    event_time: OffsetDateTime,
    clock: ClockState<clock::Countdown>,
    directed_duration: DirectedDuration,
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

        let app_datetime = OffsetDateTime::from(app_time);
        // assume event has as same `offset` as `app_time`
        let event_offset = event_time.assume_offset(app_datetime.offset());
        let directed_duration =
            DirectedDuration::from_offset_date_times(event_offset, app_datetime);
        let current_value = directed_duration.into();

        let clock = ClockState::<clock::Countdown>::new(ClockStateArgs {
            initial_value: current_value,
            current_value,
            tick_value: Duration::from_millis(TICK_VALUE_MS),
            with_decis,
            app_tx: Some(app_tx.clone()),
        });

        Self {
            title: event_title,
            event_time: event_offset,
            directed_duration,
            clock,
        }
    }

    pub fn get_clock(&self) -> &ClockState<clock::Countdown> {
        &self.clock
    }

    pub fn set_app_time(&mut self, app_time: AppTime) {
        // update `directed_duration`
        let app_datetime = OffsetDateTime::from(app_time);
        self.directed_duration =
            DirectedDuration::from_offset_date_times(self.event_time, app_datetime);
        // update clock
        let duration: Duration = self.directed_duration.into();
        self.clock.set_current_value(duration.into());
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock.with_decis = with_decis;
    }

    pub fn get_percentage_done(&self) -> u16 {
        match self.directed_duration {
            DirectedDuration::Since(_) => 100,
            DirectedDuration::Until(_) => self.clock.get_percentage_done(),
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
        let clock = &mut state.clock;
        let clock_widget = ClockWidget::new(self.style, self.blink);
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
