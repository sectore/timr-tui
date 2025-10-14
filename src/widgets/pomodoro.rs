use crate::{
    common::Style,
    constants::TICK_VALUE_MS,
    events::{AppEventTx, TuiEvent, TuiEventHandler},
    utils::center,
    widgets::clock::{ClockState, ClockStateArgs, ClockWidget, Countdown},
};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
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
        match event {
            TuiEvent::Tick => {
                self.get_clock_mut().tick();
                self.get_clock_mut().update_done_count();
            }
            // EDIT mode
            TuiEvent::Crossterm(CrosstermEvent::Key(key)) if edit_mode => match key.code {
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
            TuiEvent::Crossterm(CrosstermEvent::Key(key)) => match key.code {
                // Toggle run/pause
                KeyCode::Char('s') => {
                    self.get_clock_mut().toggle_pause();
                }
                // Enter edit mode
                KeyCode::Char('e') => {
                    self.get_clock_mut().toggle_edit();
                }
                // toggle WORK/PAUSE
                KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // `next` is acting as same as a "prev" function we don't have
                    self.next();
                }
                // toggle WORK/PAUSE
                KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
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

        let area = center(
            area,
            Constraint::Length(max(
                clock_widget
                    .get_width(state.get_clock().get_format(), state.get_clock().with_decis),
                label.width() as u16,
            )),
            Constraint::Length(
                // empty label + height of `label` + `label_round`
                clock_widget.get_height() + 3,
            ),
        );

        let [v1, v2, v3, v4] = Layout::vertical(Constraint::from_lengths([
            1,
            clock_widget.get_height(),
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
    }
}
