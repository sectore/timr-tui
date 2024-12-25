use crate::{
    args::Args,
    constants::TICK_VALUE_MS,
    events::{Event, EventHandler, Events},
    storage::AppStorage,
    terminal::Terminal,
    widgets::{
        clock::{self, Clock, ClockArgs, Style},
        countdown::{Countdown, CountdownWidget},
        footer::Footer,
        header::Header,
        pomodoro::{Mode as PomodoroMode, Pomodoro, PomodoroArgs, PomodoroWidget},
        timer::{Timer, TimerWidget},
    },
};
use clap::ValueEnum;
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default, Serialize, Deserialize,
)]
pub enum Content {
    #[default]
    #[value(name = "countdown", alias = "c")]
    Countdown,
    #[value(name = "timer", alias = "t")]
    Timer,
    #[value(name = "pomodoro", alias = "p")]
    Pomodoro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Running,
    Quit,
}

#[derive(Debug)]
pub struct App {
    content: Content,
    mode: Mode,
    show_menu: bool,
    countdown: Countdown,
    timer: Timer,
    pomodoro: Pomodoro,
    style: Style,
    with_decis: bool,
}

pub struct AppArgs {
    pub style: Style,
    pub with_decis: bool,
    pub show_menu: bool,
    pub content: Content,
    pub pomodoro_mode: PomodoroMode,
    pub initial_value_work: Duration,
    pub current_value_work: Duration,
    pub initial_value_pause: Duration,
    pub current_value_pause: Duration,
    pub initial_value_countdown: Duration,
    pub current_value_countdown: Duration,
    pub current_value_timer: Duration,
}

/// Getting `AppArgs` by merging `Args` and `AppStorage`.
/// `Args` wins btw.
impl From<(Args, AppStorage)> for AppArgs {
    fn from((args, stg): (Args, AppStorage)) -> Self {
        AppArgs {
            with_decis: args.decis || stg.with_decis,
            show_menu: stg.show_menu,
            content: args.mode.unwrap_or(stg.content),
            style: args.style.unwrap_or(stg.style),
            pomodoro_mode: stg.pomodoro_mode,
            initial_value_work: args.work.unwrap_or(stg.inital_value_work),
            current_value_work: stg.current_value_work,
            initial_value_pause: args.pause,
            current_value_pause: stg.current_value_pause,
            initial_value_countdown: args.countdown,
            current_value_countdown: stg.current_value_countdown,
            current_value_timer: stg.current_value_timer,
        }
    }
}

impl App {
    pub fn new(args: AppArgs) -> Self {
        let AppArgs {
            style,
            show_menu,
            initial_value_work,
            initial_value_pause,
            initial_value_countdown,
            current_value_work,
            current_value_pause,
            current_value_countdown,
            current_value_timer,
            content,
            with_decis,
            pomodoro_mode,
        } = args;
        Self {
            mode: Mode::Running,
            content,
            show_menu,
            style,
            with_decis,
            countdown: Countdown::new(Clock::<clock::Countdown>::new(ClockArgs {
                initial_value: initial_value_countdown,
                current_value: current_value_countdown,
                tick_value: Duration::from_millis(TICK_VALUE_MS),
                style,
                with_decis,
            })),
            timer: Timer::new(Clock::<clock::Timer>::new(ClockArgs {
                initial_value: Duration::ZERO,
                current_value: current_value_timer,
                tick_value: Duration::from_millis(TICK_VALUE_MS),
                style,
                with_decis,
            })),
            pomodoro: Pomodoro::new(PomodoroArgs {
                mode: pomodoro_mode,
                initial_value_work,
                current_value_work,
                initial_value_pause,
                current_value_pause,
                style,
                with_decis,
            }),
        }
    }

    pub async fn run(mut self, mut terminal: Terminal, mut events: Events) -> Result<Self> {
        while self.is_running() {
            if let Some(event) = events.next().await {
                // Pipe events into subviews and handle only 'unhandled' events afterwards
                if let Some(unhandled) = match self.content {
                    Content::Countdown => self.countdown.update(event.clone()),
                    Content::Timer => self.timer.update(event.clone()),
                    Content::Pomodoro => self.pomodoro.update(event.clone()),
                } {
                    match unhandled {
                        Event::Render | Event::Resize => {
                            self.draw(&mut terminal)?;
                        }
                        Event::Key(key) => self.handle_key_event(key),
                        _ => {}
                    }
                }
            }
        }
        Ok(self)
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    fn is_edit_mode(&self) -> bool {
        match self.content {
            Content::Countdown => self.countdown.get_clock().is_edit_mode(),
            Content::Timer => self.timer.get_clock().is_edit_mode(),
            Content::Pomodoro => self.pomodoro.get_clock().is_edit_mode(),
        }
    }

    fn clock_is_running(&self) -> bool {
        match self.content {
            Content::Countdown => self.countdown.get_clock().is_running(),
            Content::Timer => self.timer.get_clock().is_running(),
            Content::Pomodoro => self.pomodoro.get_clock().is_running(),
        }
    }

    fn get_percentage_done(&self) -> Option<u16> {
        match self.content {
            Content::Countdown => Some(self.countdown.get_clock().get_percentage_done()),
            Content::Timer => None,
            Content::Pomodoro => Some(self.pomodoro.get_clock().get_percentage_done()),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        debug!("Received key {:?}", key.code);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('c') => self.content = Content::Countdown,
            KeyCode::Char('t') => self.content = Content::Timer,
            KeyCode::Char('p') => self.content = Content::Pomodoro,
            KeyCode::Char('m') => self.show_menu = !self.show_menu,
            KeyCode::Char(',') => {
                self.style = self.style.next();
                // update clocks
                self.timer.set_style(self.style);
                self.countdown.set_style(self.style);
                self.pomodoro.set_style(self.style);
            }
            KeyCode::Char('.') => {
                self.with_decis = !self.with_decis;
                // update clocks
                self.timer.set_with_decis(self.with_decis);
                self.countdown.set_with_decis(self.with_decis);
                self.pomodoro.set_with_decis(self.with_decis);
            }
            KeyCode::Up => self.show_menu = true,
            KeyCode::Down => self.show_menu = false,
            _ => {}
        };
    }

    fn draw(&mut self, terminal: &mut Terminal) -> Result<()> {
        terminal.draw(|frame| {
            frame.render_stateful_widget(AppWidget, frame.area(), self);
        })?;
        Ok(())
    }

    pub fn to_storage(&self) -> AppStorage {
        AppStorage {
            content: self.content,
            show_menu: self.show_menu,
            style: self.style,
            with_decis: self.with_decis,
            pomodoro_mode: self.pomodoro.get_mode().clone(),
            inital_value_work: self.pomodoro.get_clock_work().initial_value,
            current_value_work: self.pomodoro.get_clock_work().current_value,
            inital_value_pause: self.pomodoro.get_clock_pause().initial_value,
            current_value_pause: self.pomodoro.get_clock_pause().current_value,
            inital_value_countdown: self.countdown.get_clock().initial_value,
            current_value_countdown: self.countdown.get_clock().current_value,
            current_value_timer: self.timer.get_clock().current_value,
        }
    }
}

struct AppWidget;

impl AppWidget {
    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &mut App) {
        match state.content {
            Content::Timer => TimerWidget.render(area, buf, &mut state.timer.clone()),
            Content::Countdown => CountdownWidget.render(area, buf, &mut state.countdown.clone()),
            Content::Pomodoro => PomodoroWidget.render(area, buf, &mut state.pomodoro.clone()),
        };
    }
}

impl StatefulWidget for AppWidget {
    type State = App;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [v0, v1, v2] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(if state.show_menu { 4 } else { 1 }),
        ])
        .areas(area);

        // header
        Header {
            percentage: state.get_percentage_done(),
        }
        .render(v0, buf);
        // content
        self.render_content(v1, buf, state);
        // footer
        Footer {
            show_menu: state.show_menu,
            running_clock: state.clock_is_running(),
            selected_content: state.content,
            edit_mode: state.is_edit_mode(),
        }
        .render(v2, buf);
    }
}
