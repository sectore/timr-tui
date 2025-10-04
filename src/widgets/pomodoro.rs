use crate::{
    common::Style,
    constants::TICK_VALUE_MS,
    events::{AppEventTx, TuiEvent, TuiEventHandler},
    utils::center,
    widgets::clock::{ClockState, ClockStateArgs, ClockWidget, Countdown},
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use serde::{Deserialize, Serialize};
use std::{cmp::max, time::Duration};
use strum::Display;

#[derive(Debug, Clone, Display, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum Mode {
    Work,
    Pause,
}

pub struct ClockMap {
    work: ClockState<Countdown>,
    pause: ClockState<Countdown>,
}

impl ClockMap {
    fn get_mut(&mut self, mode: &Mode) -> &mut ClockState<Countdown> {
        match mode {
            Mode::Work => &mut self.work,
            Mode::Pause => &mut self.pause,
        }
    }
    fn get(&self, mode: &Mode) -> &ClockState<Countdown> {
        match mode {
            Mode::Work => &self.work,
            Mode::Pause => &self.pause,
        }
    }
}

pub struct PomodoroState {
    mode: Mode,
    clock_map: ClockMap,
    round: u64,
    label: String,
    label_edit_mode: bool,
}

pub struct PomodoroStateArgs {
    pub mode: Mode,
    pub initial_value_work: Duration,
    pub current_value_work: Duration,
    pub initial_value_pause: Duration,
    pub current_value_pause: Duration,
    pub with_decis: bool,
    pub app_tx: AppEventTx,
    pub round: u64,
    pub label: String,
}

impl PomodoroState {
    pub fn new(args: PomodoroStateArgs) -> Self {
        let PomodoroStateArgs {
            mode,
            initial_value_work,
            current_value_work,
            initial_value_pause,
            current_value_pause,
            with_decis,
            app_tx,
            round,
            label,
        } = args;
        Self {
            mode,
            clock_map: ClockMap {
                work: ClockState::<Countdown>::new(ClockStateArgs {
                    initial_value: initial_value_work,
                    current_value: current_value_work,
                    tick_value: Duration::from_millis(TICK_VALUE_MS),
                    with_decis,
                    app_tx: Some(app_tx.clone()),
                })
                .with_name("Work".to_owned()),
                pause: ClockState::<Countdown>::new(ClockStateArgs {
                    initial_value: initial_value_pause,
                    current_value: current_value_pause,
                    tick_value: Duration::from_millis(TICK_VALUE_MS),
                    with_decis,
                    app_tx: Some(app_tx),
                })
                .with_name("Pause".to_owned()),
            },
            round,
            label,
            label_edit_mode: false,
        }
    }

    fn get_clock_mut(&mut self) -> &mut ClockState<Countdown> {
        self.clock_map.get_mut(&self.mode)
    }

    pub fn get_clock(&self) -> &ClockState<Countdown> {
        self.clock_map.get(&self.mode)
    }

    pub fn get_clock_work(&self) -> &ClockState<Countdown> {
        &self.clock_map.work
    }

    pub fn get_clock_work_mut(&mut self) -> &mut ClockState<Countdown> {
        self.clock_map.get_mut(&Mode::Work)
    }

    pub fn get_clock_pause(&self) -> &ClockState<Countdown> {
        &self.clock_map.pause
    }

    pub fn get_clock_pause_mut(&mut self) -> &mut ClockState<Countdown> {
        self.clock_map.get_mut(&Mode::Pause)
    }

    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    pub fn get_round(&self) -> u64 {
        self.round
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    pub fn is_label_edit_mode(&self) -> bool {
        self.label_edit_mode
    }

    pub fn toggle_label_edit_mode(&mut self) {
        self.label_edit_mode = !self.label_edit_mode;
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock_map.work.with_decis = with_decis;
        self.clock_map.pause.with_decis = with_decis;
    }

    pub fn next(&mut self) {
        self.mode = match self.mode {
            Mode::Pause => Mode::Work,
            Mode::Work => Mode::Pause,
        };
    }
}

impl TuiEventHandler for PomodoroState {
    fn update(&mut self, event: TuiEvent) -> Option<TuiEvent> {
        let edit_mode = self.get_clock().is_edit_mode();
        let label_edit_mode = self.is_label_edit_mode();
        match event {
            TuiEvent::Tick => {
                self.get_clock_mut().tick();
                self.get_clock_mut().update_done_count();
            }
            // LABEL EDIT mode
            TuiEvent::Key(key) if label_edit_mode => match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.toggle_label_edit_mode();
                }
                KeyCode::Char(c) => {
                    self.label.push(c);
                }
                KeyCode::Backspace => {
                    self.label.pop();
                }
                _ => return Some(event),
            },
            // CLOCK EDIT mode
            TuiEvent::Key(key) if edit_mode => match key.code {
                // Skip changes
                KeyCode::Esc => {
                    let clock = self.get_clock_mut();
                    // Important: set current value first
                    clock.set_current_value(*clock.get_prev_value());
                    // before toggling back to non-edit mode
                    clock.toggle_edit();
                }
                // Apply changes and update initial value
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.get_clock_mut().toggle_edit();
                    // update initial value
                    let c = *self.get_clock().get_current_value();
                    self.get_clock_mut().set_initial_value(c);
                }
                // Apply changes
                KeyCode::Char('s') => {
                    self.get_clock_mut().toggle_edit();
                }
                // Value up
                KeyCode::Up => {
                    self.get_clock_mut().edit_up();
                }
                // Value down
                KeyCode::Down => {
                    self.get_clock_mut().edit_down();
                }
                // move edit position to the left
                KeyCode::Left => {
                    self.get_clock_mut().edit_next();
                }
                // move edit position to the right
                KeyCode::Right => {
                    self.get_clock_mut().edit_prev();
                }
                _ => return Some(event),
            },
            // default mode
            TuiEvent::Key(key) => match key.code {
                // Toggle run/pause
                KeyCode::Char('s') => {
                    self.get_clock_mut().toggle_pause();
                }
                // Enter edit mode
                KeyCode::Char('e') => {
                    self.get_clock_mut().toggle_edit();
                }
                // Enter label edit mode
                KeyCode::Char('n') => {
                    self.toggle_label_edit_mode();
                }
                // toggle WORK/PAUSE
                KeyCode::Left => {
                    // `next` is acting as same as a "prev" function we don't have
                    self.next();
                }
                // toggle WORK/PAUSE
                KeyCode::Right => {
                    self.next();
                }
                // reset rounds AND clocks
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.round = 1;
                    self.get_clock_work_mut().reset();
                    self.get_clock_pause_mut().reset();
                }
                // reset current clock
                KeyCode::Char('r') => {
                    // increase round before (!!) resetting the clock
                    if self.get_mode() == &Mode::Work && self.get_clock().is_done() {
                        self.round += 1;
                    }
                    self.get_clock_mut().reset();
                }
                _ => return Some(event),
            },
            _ => return Some(event),
        }
        None
    }
}

pub struct PomodoroWidget {
    pub style: Style,
    pub blink: bool,
}

impl StatefulWidget for PomodoroWidget {
    type State = PomodoroState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let clock_widget = ClockWidget::new(self.style, self.blink);
        let label = Line::raw(
            (format!(
                "Pomodoro {} {}",
                state.mode.clone(),
                state.get_clock_mut().get_mode()
            ))
            .to_uppercase(),
        );
        let label_round = Line::raw((format!("round {}", state.get_round(),)).to_uppercase());

        // Display label with edit indicator if in edit mode
        let label_task = if state.is_label_edit_mode() {
            Line::raw(format!("Task: {}█", state.get_label()))
        } else if !state.get_label().is_empty() {
            Line::raw(format!("Task: {}", state.get_label()))
        } else {
            Line::raw("")
        };

        let area = center(
            area,
            Constraint::Length(max(
                max(
                    clock_widget
                        .get_width(state.get_clock().get_format(), state.get_clock().with_decis),
                    label.width() as u16,
                ),
                label_task.width() as u16,
            )),
            Constraint::Length(
                // empty label + height of `label` + `label_round` + `label_task`
                clock_widget.get_height() + 4,
            ),
        );

        let [v1, v2, v3, v4, v5] = Layout::vertical(Constraint::from_lengths([
            1,
            clock_widget.get_height(),
            1,
            1,
            1,
        ]))
        .areas(area);

        // empty line keep everything in center vertically comparing to other
        // views (which have one label below the clock only)
        Line::raw("").centered().render(v1, buf);
        clock_widget.render(v2, buf, state.get_clock_mut());
        label.centered().render(v3, buf);
        label_round.centered().render(v4, buf);
        label_task.centered().render(v5, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::AppEventTx;
    use tokio::sync::mpsc;

    fn default_pomodoro_args() -> PomodoroStateArgs {
        let (app_tx, _rx) = mpsc::unbounded_channel();
        PomodoroStateArgs {
            mode: Mode::Work,
            initial_value_work: Duration::from_secs(60 * 25),
            current_value_work: Duration::from_secs(60 * 25),
            initial_value_pause: Duration::from_secs(60 * 5),
            current_value_pause: Duration::from_secs(60 * 5),
            with_decis: false,
            app_tx,
            round: 1,
            label: String::new(),
        }
    }

    #[test]
    fn test_label_starts_empty() {
        let state = PomodoroState::new(default_pomodoro_args());
        assert_eq!(state.get_label(), "");
    }

    #[test]
    fn test_label_initialization() {
        let (app_tx, _rx) = mpsc::unbounded_channel();
        let args = PomodoroStateArgs {
            label: "Test task".to_string(),
            app_tx,
            ..default_pomodoro_args()
        };
        let state = PomodoroState::new(args);
        assert_eq!(state.get_label(), "Test task");
    }

    #[test]
    fn test_label_edit_mode_toggle() {
        let mut state = PomodoroState::new(default_pomodoro_args());
        assert!(!state.is_label_edit_mode());

        state.toggle_label_edit_mode();
        assert!(state.is_label_edit_mode());

        state.toggle_label_edit_mode();
        assert!(!state.is_label_edit_mode());
    }

    #[test]
    fn test_label_editing_via_events() {
        let mut state = PomodoroState::new(default_pomodoro_args());

        // Enter label edit mode
        state.toggle_label_edit_mode();
        assert!(state.is_label_edit_mode());

        // Type some characters
        state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('t')
        )));
        state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('e')
        )));
        state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('s')
        )));
        state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('t')
        )));

        assert_eq!(state.get_label(), "test");

        // Exit edit mode
        state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Enter
        )));
        assert!(!state.is_label_edit_mode());
        assert_eq!(state.get_label(), "test");
    }

    #[test]
    fn test_label_backspace() {
        let mut state = PomodoroState::new(default_pomodoro_args());

        state.toggle_label_edit_mode();

        // Type "hello"
        for c in ['h', 'e', 'l', 'l', 'o'] {
            state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
                crossterm::event::KeyCode::Char(c)
            )));
        }
        assert_eq!(state.get_label(), "hello");

        // Backspace twice
        state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Backspace
        )));
        state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Backspace
        )));

        assert_eq!(state.get_label(), "hel");
    }

    #[test]
    fn test_label_escape_exits_edit_mode() {
        let mut state = PomodoroState::new(default_pomodoro_args());

        state.toggle_label_edit_mode();
        assert!(state.is_label_edit_mode());

        state.update(TuiEvent::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Esc
        )));

        assert!(!state.is_label_edit_mode());
    }
}
