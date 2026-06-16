use crate::{
    common::Style,
    constants::TICK_VALUE_MS,
    events::{AppEventTx, TuiEvent, TuiEventHandler},
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

fn work_clock_name(round: u64) -> String {
    format!("work (round {round})")
}

fn pause_clock_name(round: u64, pause_duration: &PauseDuration) -> String {
    format!(
        "{} (round {round})",
        if pause_duration.is_special_round(round) {
            "pause special"
        } else {
            "pause"
        }
    )
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum PauseDuration {
    Fixed(Duration),
    Variable {
        regular: Duration,
        special: Duration,
        special_every: u64,
    },
}

impl PauseDuration {
    pub fn is_special_round(&self, round: u64) -> bool {
        match self {
            Self::Variable { special_every, .. } => round.is_multiple_of(*special_every),
            Self::Fixed(_) => false,
        }
    }

    pub fn for_round(&self, round: u64) -> Duration {
        match self {
            Self::Fixed(d) => *d,
            Self::Variable {
                regular, special, ..
            } => {
                if self.is_special_round(round) {
                    *special
                } else {
                    *regular
                }
            }
        }
    }
}

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
    pause_duration: PauseDuration,
    vim_motions: bool,
    auto_switch: bool,
}

pub struct PomodoroStateArgs {
    pub mode: Mode,
    pub initial_value_work: Duration,
    pub current_value_work: Duration,
    pub pause_duration: PauseDuration,
    pub current_value_pause: Duration,
    pub with_decis: bool,
    pub app_tx: AppEventTx,
    pub round: u64,
    pub vim_motions: bool,
    pub auto_switch: bool,
}

impl PomodoroState {
    pub fn new(args: PomodoroStateArgs) -> Self {
        let PomodoroStateArgs {
            mode,
            initial_value_work,
            current_value_work,
            pause_duration,
            current_value_pause,
            with_decis,
            app_tx,
            round,
            vim_motions,
            auto_switch,
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
                .with_name(work_clock_name(round)),
                pause: ClockState::<Countdown>::new(ClockStateArgs {
                    initial_value: pause_duration.for_round(round),
                    current_value: current_value_pause,
                    tick_value: Duration::from_millis(TICK_VALUE_MS),
                    with_decis,
                    app_tx: Some(app_tx),
                })
                .with_name(pause_clock_name(round, &pause_duration)),
            },
            round,
            pause_duration,
            vim_motions,
            auto_switch,
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

    pub fn get_pause_duration(&self) -> &PauseDuration {
        &self.pause_duration
    }

    pub fn get_auto_switch(&self) -> bool {
        self.auto_switch
    }

    fn update_clock_names(&mut self) {
        let round = self.round;
        let work_name = work_clock_name(round);
        let pause_name = pause_clock_name(round, &self.pause_duration);
        self.get_clock_work_mut().set_name(work_name);
        self.get_clock_pause_mut().set_name(pause_name);
    }

    fn update_pause_initial(&mut self) {
        let initial = self.pause_duration.for_round(self.round);
        self.get_clock_pause_mut().set_initial_value(initial.into());
    }

    pub fn set_with_decis(&mut self, with_decis: bool) {
        self.clock_map.work.with_decis = with_decis;
        self.clock_map.pause.with_decis = with_decis;
    }

    fn next_round(&mut self) {
        // increase round before (!!) updating the clock
        self.round += 1;
        self.update_clock_names();
        self.update_pause_initial();
        self.get_clock_pause_mut().reset();
        self.get_clock_work_mut().reset();
    }

    fn prev_round(&mut self) {
        // decrease round before (!!) updating the clock
        if self.round > 1 {
            self.round -= 1;
        }
        self.update_clock_names();
        self.update_pause_initial();
        self.get_clock_pause_mut().reset();
        self.get_clock_work_mut().reset();
    }

    // Switch `Mode`
    fn switch_mode(&mut self) {
        match self.mode {
            Mode::Pause => {
                // count round if both clocks are done
                if self.get_clock_pause().is_done() && self.get_clock_work().is_done() {
                    self.next_round();
                }
                // switch
                self.mode = Mode::Work;
            }
            Mode::Work => {
                // switch
                self.mode = Mode::Pause;
            }
        }
    }

    // Switch `Mode` automatically
    fn switch_mode_auto(&mut self) {
        self.switch_mode();
        self.get_clock_mut().run();
    }
}

impl TuiEventHandler for PomodoroState {
    fn update(&mut self, event: TuiEvent) -> Option<TuiEvent> {
        let edit_mode = self.get_clock().is_edit_mode();
        match event {
            TuiEvent::Tick => {
                self.get_clock_mut().tick();
                self.get_clock_mut().update_done_count();
                if self.auto_switch && self.get_clock().is_done_counted() {
                    self.switch_mode_auto();
                }
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
                // change value up
                KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.get_clock_mut().edit_jump_up();
                }
                KeyCode::Char('k')
                    if key.modifiers.contains(KeyModifiers::CONTROL) && self.vim_motions =>
                {
                    self.get_clock_mut().edit_jump_up();
                }
                KeyCode::Up if !self.vim_motions => {
                    self.get_clock_mut().edit_up();
                }
                KeyCode::Char('k') if self.vim_motions => {
                    self.get_clock_mut().edit_up();
                }
                // change value down
                KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.get_clock_mut().edit_jump_down();
                }
                KeyCode::Char('j')
                    if key.modifiers.contains(KeyModifiers::CONTROL) && self.vim_motions =>
                {
                    self.get_clock_mut().edit_jump_down();
                }
                KeyCode::Down if !self.vim_motions => {
                    self.get_clock_mut().edit_down();
                }
                KeyCode::Char('j') if self.vim_motions => {
                    self.get_clock_mut().edit_down();
                }
                // move edit position to the left
                KeyCode::Left if !self.vim_motions => {
                    self.get_clock_mut().edit_next();
                }
                KeyCode::Char('h') if self.vim_motions => {
                    self.get_clock_mut().edit_next();
                }
                // move edit position to the right
                KeyCode::Right if !self.vim_motions => {
                    self.get_clock_mut().edit_prev();
                }
                KeyCode::Char('l') if self.vim_motions => {
                    self.get_clock_mut().edit_prev();
                }
                _ => return Some(event),
            },
            // default mode
            TuiEvent::Crossterm(CrosstermEvent::Key(key)) => match key.code {
                // Toggle run/pause
                KeyCode::Char(' ') => {
                    self.get_clock_mut().toggle_pause();
                }
                // Enter edit mode
                KeyCode::Char('e') => {
                    self.get_clock_mut().toggle_edit();
                }
                // toggle WORK/PAUSE
                KeyCode::Left
                    if key.modifiers.contains(KeyModifiers::CONTROL) && !self.vim_motions =>
                {
                    self.switch_mode();
                }
                KeyCode::Char('h')
                    if key.modifiers.contains(KeyModifiers::CONTROL) && self.vim_motions =>
                {
                    self.switch_mode();
                }
                // toggle WORK/PAUSE
                KeyCode::Right
                    if key.modifiers.contains(KeyModifiers::CONTROL) && !self.vim_motions =>
                {
                    self.switch_mode();
                }
                KeyCode::Char('l')
                    if key.modifiers.contains(KeyModifiers::CONTROL) && self.vim_motions =>
                {
                    self.switch_mode();
                }
                // next round
                KeyCode::Up => self.next_round(),
                KeyCode::Char('k') if self.vim_motions => {
                    self.next_round();
                }
                // prev. round
                KeyCode::Down => self.prev_round(),
                KeyCode::Char('j') if self.vim_motions => {
                    self.prev_round();
                }
                // toggle autoswitch
                KeyCode::Char('a') => {
                    self.auto_switch = !self.auto_switch;
                }
                // reset rounds AND clocks
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.round = 1;
                    self.update_pause_initial();
                    self.get_clock_pause_mut().reset();
                    self.get_clock_work_mut().reset();
                }
                // reset current clock
                KeyCode::Char('r') => {
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
        let is_special_pause = state.get_mode() == &Mode::Pause
            && state
                .get_pause_duration()
                .is_special_round(state.get_round());
        let label = Line::raw(
            (format!(
                "Pomodoro {} {}{}",
                state.mode.clone(),
                if is_special_pause { "Special " } else { "" },
                state.get_clock_mut().get_mode()
            ))
            .to_uppercase(),
        );
        let label_round = Line::raw((format!("round {}", state.get_round(),)).to_uppercase());

        let area = area.centered(
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
