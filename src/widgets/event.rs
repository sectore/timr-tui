use color_eyre::{Report, eyre::eyre, owo_colors::OwoColorize};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Paragraph, StatefulWidget, Widget},
};
use time::{OffsetDateTime, macros::format_description};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::{
    common::{AppTime, ClockTypeId, Style as DigitStyle},
    duration::CalendarDuration,
    event::Event,
    events::{AppEvent, AppEventTx, TuiEvent, TuiEventHandler},
    utils::center,
    widgets::{clock, clock_elements::DIGIT_HEIGHT},
};
use std::{cmp::max, time::Duration};

#[derive(PartialEq)]
enum Editable {
    DateTime,
    Title,
}

impl Editable {
    pub fn next(&self) -> Self {
        match self {
            Editable::DateTime => Editable::Title,
            Editable::Title => Editable::DateTime,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Editable::DateTime => Editable::Title,
            Editable::Title => Editable::DateTime,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum EditMode {
    None,
    Editing,
}

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
    // inputs
    input_datetime: Input,
    input_datetime_error: Option<Report>,
    input_title: Input,
    input_title_error: Option<Report>,
    edit_mode: EditMode,
    editable: Editable,
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
        let input_datetime_value = format_offsetdatetime(&event_offset);
        let input_title_value = event.title.clone().unwrap_or("".into());

        Self {
            title: event.title.clone(),
            event_time: event_offset,
            app_time: app_datetime,
            start_time: app_datetime,
            with_decis,
            done_count: None,
            app_tx,
            input_datetime: Input::default().with_value(input_datetime_value),
            input_datetime_error: None,
            input_title: Input::default().with_value(input_title_value),
            input_title_error: None,
            edit_mode: EditMode::None,
            editable: Editable::DateTime,
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

    pub fn get_event(&self) -> Event {
        Event {
            title: self.title.clone(),
            date_time: time::PrimitiveDateTime::new(self.event_time.date(), self.event_time.time()),
        }
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

    fn reset_input_datetime(&mut self) {
        self.input_datetime = Input::default().with_value(format_offsetdatetime(&self.event_time));
        self.input_datetime_error = None;
    }

    fn reset_input_title(&mut self) {
        self.input_title = Input::default().with_value(self.title.clone().unwrap_or_default());
        self.input_title_error = None;
    }
}

fn validate_datetime(value: &str) -> Result<time::PrimitiveDateTime, Report> {
    time::PrimitiveDateTime::parse(
        value,
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    )
    .map_err(|_| eyre!("Expected format 'YYYY-MM-DD HH:MM:SS'"))
}

const MAX_LABEL_WIDTH: usize = 60;

fn validate_title(value: &str) -> Result<&str, Report> {
    if value.len() > MAX_LABEL_WIDTH {
        return Err(eyre!("Max. {} chars", MAX_LABEL_WIDTH));
    }
    Ok(value)
}

impl TuiEventHandler for EventState {
    fn update(&mut self, event: TuiEvent) -> Option<TuiEvent> {
        let edit_mode = self.edit_mode != EditMode::None;
        match event {
            // EDIT mode
            TuiEvent::Crossterm(crossterm_event @ CrosstermEvent::Key(key)) if edit_mode => {
                match key.code {
                    // Skip changes
                    KeyCode::Esc => {
                        // reset inputs
                        self.reset_input_datetime();
                        self.reset_input_title();

                        self.edit_mode = EditMode::None;
                    }
                    KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        self.editable = self.editable.prev()
                    }
                    KeyCode::Tab => self.editable = self.editable.next(),
                    KeyCode::Enter => match self.editable {
                        Editable::DateTime => {
                            // validate
                            if let Ok(date_time) = validate_datetime(self.input_datetime.value()) {
                                // apply offset
                                self.event_time = date_time.assume_offset(self.app_time.offset());
                            } else {
                                // reset
                                self.reset_input_datetime();
                            }
                            self.edit_mode = EditMode::None;
                        }
                        Editable::Title => {
                            self.title = validate_title(self.input_title.value())
                                .ok()
                                .filter(|v| !v.is_empty())
                                .map(str::to_string);
                            self.edit_mode = EditMode::None;
                        }
                    },
                    _ => match self.editable {
                        Editable::DateTime => {
                            self.input_datetime.handle_event(&crossterm_event);
                            if let Err(e) = validate_datetime(self.input_datetime.value()) {
                                self.input_datetime_error = Some(e);
                            } else {
                                self.input_datetime_error = None;
                            }
                        }
                        Editable::Title => {
                            self.input_title.handle_event(&crossterm_event);
                            if let Err(e) = validate_title(self.input_title.value()) {
                                self.input_title_error = Some(e);
                            } else {
                                self.input_title_error = None;
                            }
                        }
                    },
                }
            }
            // default mode
            TuiEvent::Crossterm(CrosstermEvent::Key(key)) => match key.code {
                // Enter edit mode
                KeyCode::Char('e') => {
                    self.edit_mode = EditMode::Editing;
                }
                _ => return Some(event),
            },
            _ => return Some(event),
        }

        None
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

fn format_offsetdatetime(dt: &OffsetDateTime) -> String {
    dt.format(&format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ))
    .unwrap_or_else(|e| format!("time format error: {}", e))
}

#[derive(Debug)]
pub struct EventWidget {
    pub style: DigitStyle,
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
        let time_str = format_offsetdatetime(&state.event_time);
        let time_prefix = if state.edit_mode == EditMode::Editing {
            "Edit"
        } else if clock_duration.is_since() {
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

        // let label_time = Line::raw(format!(
        //     "{} {}",
        //     time_prefix.to_uppercase(),
        //     time_str.to_uppercase()
        // ));
        // let max_label_width = max(label_event.width(), label_time.width()) as u16;

        let area = center(
            area,
            Constraint::Length(max(clock_width, MAX_LABEL_WIDTH as u16)),
            Constraint::Length(
                DIGIT_HEIGHT + 7, /* height of all labels + empty lines */
            ),
        );
        let [_, v1, v2, v3, _, v4] = Layout::vertical(Constraint::from_lengths([
            3, // empty (offset) to keep everything centered vertically comparing to "clock" widgets with one label only
            DIGIT_HEIGHT,
            1, // label: event date
            1, // label: event title
            1, // empty
            1, // label: error
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

        // Helper to calculate centered area, cursor x position, and scroll
        let centered_input = |input: &Input, area: Rect| -> (Rect, u16, usize) {
            // Calculate scroll position to keep cursor visible
            let input_scroll = input.visual_scroll(area.width as usize);

            // Get correct visual width (handles unicode properly)
            let text_width = Line::raw(input.value()).width() as u16;

            // Calculate visible text width after scrolling
            let visible_text_width = text_width
                .saturating_sub(input_scroll as u16)
                .min(area.width);

            // Center the visible portion
            let offset_x = (area.width.saturating_sub(visible_text_width)) / 2;

            let centered_area = Rect {
                x: area.x + offset_x,
                y: area.y,
                width: visible_text_width,
                height: area.height,
            };

            // Cursor position relative to the visible scrolled text
            let cursor_offset = input.visual_cursor().saturating_sub(input_scroll);
            let cursor_x = area.x + offset_x + cursor_offset as u16;

            (centered_area, cursor_x, input_scroll)
        };

        fn input_style(edit_mode: EditMode, editable: bool, error: bool) -> Style {
            match edit_mode {
                EditMode::Editing if editable && error => Style::default()
                    .add_modifier(Modifier::UNDERLINED)
                    .fg(Color::Red),
                EditMode::Editing if editable => {
                    Style::default().add_modifier(Modifier::UNDERLINED)
                }
                _ => Style::default(),
            }
        }

        // Calculate centered areas, cursor positions, and scroll values
        let (datetime_area, datetime_cursor_x, datetime_scroll) =
            centered_input(&state.input_datetime, v2);
        let (title_area, title_cursor_x, title_scroll) = centered_input(&state.input_title, v3);

        // Calculate cursor position if in edit mode
        let cursor_position = if state.edit_mode == EditMode::Editing {
            let (cursor_x, cursor_y) = match state.editable {
                Editable::DateTime => (datetime_cursor_x, v2.y),
                Editable::Title => (title_cursor_x, v3.y),
            };
            Some(Position::new(cursor_x, cursor_y))
        } else {
            None
        };

        // Send cursor position via event
        let _ = state.app_tx.send(AppEvent::SetCursor(cursor_position));

        // Render datetime input
        let input_datetime_widget = Paragraph::new(state.input_datetime.value())
            .style(input_style(
                state.edit_mode,
                state.editable == Editable::DateTime,
                state.input_datetime_error.is_some(),
            ))
            .scroll((0, datetime_scroll as u16));
        input_datetime_widget.render(datetime_area, buf);

        // Render title input
        let title_input_widget = Paragraph::new(state.input_title.value().to_uppercase())
            .style(input_style(
                state.edit_mode,
                state.editable == Editable::Title,
                state.input_title_error.is_some(),
            ))
            .scroll((0, title_scroll as u16));
        title_input_widget.render(title_area, buf);

        let error_txt: String = match (&state.input_datetime_error, &state.input_title_error) {
            (Some(e), _) => e.to_string(),
            (_, Some(e)) => e.to_string(),
            _ => "".into(),
        };
        Paragraph::new(error_txt.to_lowercase())
            // .style(Style::default().fg(Color::Red))
            .style(Style::default().add_modifier(Modifier::ITALIC))
            .centered()
            .render(v4, buf);
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
