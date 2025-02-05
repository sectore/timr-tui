use crate::{
    common::{AppTime, Style},
    constants::TICK_VALUE_MS,
    duration::{DurationEx, MAX_DURATION},
    events::{AppEventTx, TuiEvent, TuiEventHandler},
    utils::center,
    widgets::{
        clock::{self, ClockState, ClockStateArgs, ClockWidget, Mode as ClockMode},
        edit_time::{EditTimeState, EditTimeStateArgs, EditTimeWidget},
    },
};
use crossterm::event::KeyModifiers;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use std::ops::Sub;
use std::{cmp::max, time::Duration};
use time::OffsetDateTime;

pub struct CountdownStateArgs {
    pub initial_value: Duration,
    pub current_value: Duration,
    pub elapsed_value: Duration,
    pub app_time: AppTime,
    pub with_decis: bool,
    pub app_tx: AppEventTx,
}

/// State for Countdown Widget
pub struct CountdownState {
    /// clock to count down
    clock: ClockState<clock::Countdown>,
    /// clock to count time after `DONE` - similar to Mission Elapsed Time (MET)
    elapsed_clock: ClockState<clock::Timer>,
    app_time: AppTime,
    /// Edit by local time
    edit_time: Option<EditTimeState>,
}

impl CountdownState {
    pub fn new(args: CountdownStateArgs) -> Self {
        let CountdownStateArgs {
            initial_value,
            current_value,
            elapsed_value,
            with_decis,
            app_time,
            app_tx,
        } = args;

        Self {
            clock: ClockState::<clock::Countdown>::new(ClockStateArgs {
                initial_value,
                current_value,
                tick_value: Duration::from_millis(TICK_VALUE_MS),
                with_decis,
                app_tx: Some(app_tx.clone()),
            }),
            elapsed_clock: ClockState::<clock::Timer>::new(ClockStateArgs {
                initial_value: Duration::ZERO,
                current_value: elapsed_value,
                tick_value: Duration::from_millis(TICK_VALUE_MS),
                with_decis: false,
                app_tx: None,
            })
            .with_name("MET".to_owned())
            // A previous `elapsed_value > 0` means the `Clock` was running before,
            // but not in `Initial` state anymore. Updating `Mode` here
            // is needed to handle `Event::Tick` in `EventHandler::update` properly
            .with_mode(if elapsed_value.gt(&Duration::ZERO) {
                ClockMode::Pause
            } else {
                ClockMode::Initial
            }),
            app_time,
            edit_time: None,
        }
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock.with_decis = with_decis;
        self.elapsed_clock.with_decis = with_decis;
    }

    pub fn get_clock(&self) -> &ClockState<clock::Countdown> {
        &self.clock
    }

    pub fn is_running(&self) -> bool {
        self.clock.is_running() || self.elapsed_clock.is_running()
    }

    pub fn get_elapsed_value(&self) -> &DurationEx {
        self.elapsed_clock.get_current_value()
    }

    pub fn set_app_time(&mut self, app_time: AppTime) {
        self.app_time = app_time;
    }

    fn time_to_edit(&self) -> OffsetDateTime {
        // get current value
        let d: Duration = (*self.clock.get_current_value()).into();
        // transform
        let dd = time::Duration::try_from(d).unwrap_or(time::Duration::ZERO);
        // substract from `app_time`
        OffsetDateTime::from(self.app_time).saturating_add(dd)
    }

    pub fn min_time_to_edit(&self) -> OffsetDateTime {
        OffsetDateTime::from(self.app_time)
    }

    fn max_time_to_edit(&self) -> OffsetDateTime {
        OffsetDateTime::from(self.app_time)
            .saturating_add(time::Duration::try_from(MAX_DURATION).unwrap_or(time::Duration::ZERO))
    }

    fn edit_time_done(&mut self, edit_time: &mut EditTimeState) {
        // get diff
        let d: time::Duration = edit_time
            .get_time()
            .sub(OffsetDateTime::from(self.app_time));
        // transfrom
        let dx: DurationEx = Duration::try_from(d).unwrap_or(Duration::ZERO).into();
        // update clock
        self.clock.set_current_value(dx);
        // remove `edit_time`
        self.edit_time = None;
    }

    pub fn is_clock_edit_mode(&self) -> bool {
        self.clock.is_edit_mode()
    }

    pub fn is_time_edit_mode(&self) -> bool {
        self.edit_time.is_some()
    }
}

impl TuiEventHandler for CountdownState {
    fn update(&mut self, event: TuiEvent) -> Option<TuiEvent> {
        let is_edit_clock = self.clock.is_edit_mode();
        let is_edit_time = self.edit_time.is_some();
        match event {
            TuiEvent::Tick => {
                if !self.clock.is_done() {
                    self.clock.tick();
                } else {
                    self.clock.update_done_count();
                    self.elapsed_clock.tick();
                    if self.elapsed_clock.is_initial() {
                        self.elapsed_clock.run();
                    }
                }
                let min_time = self.min_time_to_edit();
                let max_time = self.max_time_to_edit();
                if let Some(edit_time) = &mut self.edit_time {
                    edit_time.set_min_time(min_time);
                    edit_time.set_max_time(max_time);
                }
            }
            TuiEvent::Key(key) => match key.code {
                KeyCode::Char('r') => {
                    // reset both clocks to use intial values
                    self.clock.reset();
                    self.elapsed_clock.reset();

                    // reset `edit_time` back initial value
                    let time = self.time_to_edit();
                    if let Some(edit_time) = &mut self.edit_time {
                        edit_time.set_time(time);
                    }
                }
                KeyCode::Char('s') => {
                    // toggle pause status depending on which clock is running
                    if !self.clock.is_done() {
                        self.clock.toggle_pause();
                    } else {
                        self.elapsed_clock.toggle_pause();
                    }

                    // finish `edit_time` and continue for using `clock`
                    if let Some(edit_time) = &mut self.edit_time.clone() {
                        self.edit_time_done(edit_time);
                    }
                }
                // STRG + e => toggle edit time
                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // stop editing clock
                    if self.clock.is_edit_mode() {
                        // toggle edit mode
                        self.clock.toggle_edit();
                    }

                    if let Some(edit_time) = &mut self.edit_time.clone() {
                        self.edit_time_done(edit_time)
                    } else {
                        // update `edit_time`
                        self.edit_time = Some(EditTimeState::new(EditTimeStateArgs {
                            time: self.time_to_edit(),
                            min: self.min_time_to_edit(),
                            max: self.max_time_to_edit(),
                        }));
                    }

                    // stop `clock`
                    if self.clock.is_running() {
                        self.clock.toggle_pause();
                    }

                    // stop `elapsed_clock`
                    if self.elapsed_clock.is_running() {
                        self.elapsed_clock.toggle_pause();
                    }
                }
                // STRG + e => toggle edit clock
                KeyCode::Char('e') => {
                    // toggle edit mode
                    self.clock.toggle_edit();

                    // stop `elapsed_clock`
                    if self.elapsed_clock.is_running() {
                        self.elapsed_clock.toggle_pause();
                    }

                    // finish `edit_time` and continue for using `clock`
                    if let Some(edit_time) = &mut self.edit_time.clone() {
                        self.edit_time_done(edit_time);
                    }
                }
                KeyCode::Left if is_edit_clock => {
                    self.clock.edit_next();
                }
                KeyCode::Left if is_edit_time => {
                    // safe unwrap because of previous check in `is_edit_time`
                    self.edit_time.as_mut().unwrap().next();
                }
                KeyCode::Right if is_edit_clock => {
                    self.clock.edit_prev();
                }
                KeyCode::Right if is_edit_time => {
                    // safe unwrap because of previous check in `is_edit_time`
                    self.edit_time.as_mut().unwrap().prev();
                }
                KeyCode::Up if is_edit_clock => {
                    self.clock.edit_up();
                    // whenever `clock`'s value is changed, reset `elapsed_clock`
                    self.elapsed_clock.reset();
                }
                KeyCode::Up if is_edit_time => {
                    // safe unwrap because of previous check in `is_edit_time`
                    self.edit_time.as_mut().unwrap().up();
                    // whenever `clock`'s value is changed, reset `elapsed_clock`
                    self.elapsed_clock.reset();
                }
                KeyCode::Down if is_edit_clock => {
                    self.clock.edit_down();
                    // whenever clock value is changed, reset timer
                    self.elapsed_clock.reset();
                }
                KeyCode::Down if is_edit_time => {
                    // safe unwrap because of previous check in `is_edit_time`
                    self.edit_time.as_mut().unwrap().down();
                    // whenever clock value is changed, reset timer
                    self.elapsed_clock.reset();
                }
                _ => return Some(event),
            },
            _ => return Some(event),
        }
        None
    }
}

pub struct Countdown {
    pub style: Style,
    pub blink: bool,
}

fn human_days_diff(a: &OffsetDateTime, b: &OffsetDateTime) -> String {
    let days_diff = (a.date() - b.date()).whole_days();
    match days_diff {
        0 => "today".to_owned(),
        1 => "tomorrow".to_owned(),
        n => format!("+{}days", n),
    }
}

impl StatefulWidget for Countdown {
    type State = CountdownState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // render `edit_time` OR `clock`
        if let Some(edit_time) = &mut state.edit_time {
            let label = Line::raw(
                format!(
                    "Countdown {} {}",
                    edit_time.get_selected().clone(),
                    human_days_diff(edit_time.get_time(), &state.app_time.into())
                )
                .to_uppercase(),
            );
            let widget = EditTimeWidget::new(self.style);
            let area = center(
                area,
                Constraint::Length(max(widget.get_width(), label.width() as u16)),
                Constraint::Length(widget.get_height() + 1 /* height of label */),
            );
            let [v1, v2] =
                Layout::vertical(Constraint::from_lengths([widget.get_height(), 1])).areas(area);

            widget.render(v1, buf, edit_time);
            label.centered().render(v2, buf);
        } else {
            let label = Line::raw(
                if state.clock.is_done() {
                    if state.clock.with_decis {
                        format!(
                            "Countdown {} +{}",
                            state.clock.get_mode(),
                            state
                                .elapsed_clock
                                .get_current_value()
                                .to_string_with_decis()
                        )
                    } else {
                        format!(
                            "Countdown {} +{}",
                            state.clock.get_mode(),
                            state.elapsed_clock.get_current_value()
                        )
                    }
                } else {
                    format!("Countdown {}", state.clock.get_mode())
                }
                .to_uppercase(),
            );
            let widget = ClockWidget::new(self.style, self.blink);
            let area = center(
                area,
                Constraint::Length(max(
                    widget.get_width(&state.clock.get_format(), state.clock.with_decis),
                    label.width() as u16,
                )),
                Constraint::Length(widget.get_height() + 1 /* height of label */),
            );
            let [v1, v2] =
                Layout::vertical(Constraint::from_lengths([widget.get_height(), 1])).areas(area);

            widget.render(v1, buf, &mut state.clock);
            label.centered().render(v2, buf);
        }
    }
}
