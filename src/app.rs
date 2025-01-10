use crate::{
    args::Args,
    common::{AppTime, AppTimeFormat, Content, Style},
    constants::TICK_VALUE_MS,
    events::{Event, EventHandler, Events},
    storage::AppStorage,
    terminal::Terminal,
    widgets::{
        clock::{self, ClockState, ClockStateArgs},
        countdown::{Countdown, CountdownState},
        footer::{Footer, FooterState},
        header::Header,
        pomodoro::{Mode as PomodoroMode, PomodoroState, PomodoroStateArgs, PomodoroWidget},
        timer::{Timer, TimerState},
    },
};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};
use std::time::Duration;
use time::OffsetDateTime;
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Running,
    Quit,
}

#[derive(Debug)]
pub struct App {
    content: Content,
    mode: Mode,
    app_time: AppTime,
    countdown: CountdownState,
    timer: TimerState,
    pomodoro: PomodoroState,
    style: Style,
    with_decis: bool,
    footer: FooterState,
}

pub struct AppArgs {
    pub style: Style,
    pub with_decis: bool,
    pub show_menu: bool,
    pub app_time_format: AppTimeFormat,
    pub content: Content,
    pub pomodoro_mode: PomodoroMode,
    pub initial_value_work: Duration,
    pub current_value_work: Duration,
    pub initial_value_pause: Duration,
    pub current_value_pause: Duration,
    pub initial_value_countdown: Duration,
    pub current_value_countdown: Duration,
    pub elapsed_value_countdown: Duration,
    pub current_value_timer: Duration,
}

/// Getting `AppArgs` by merging `Args` and `AppStorage`.
/// `Args` wins btw.
impl From<(Args, AppStorage)> for AppArgs {
    fn from((args, stg): (Args, AppStorage)) -> Self {
        AppArgs {
            with_decis: args.decis || stg.with_decis,
            show_menu: args.menu || stg.show_menu,
            app_time_format: stg.app_time_format,
            content: args.mode.unwrap_or(stg.content),
            style: args.style.unwrap_or(stg.style),
            pomodoro_mode: stg.pomodoro_mode,
            initial_value_work: args.work.unwrap_or(stg.inital_value_work),
            // invalidate `current_value_work` if an initial value is set via args
            current_value_work: args.work.unwrap_or(stg.current_value_work),
            initial_value_pause: args.pause.unwrap_or(stg.inital_value_pause),
            // invalidate `current_value_pause` if an initial value is set via args
            current_value_pause: args.pause.unwrap_or(stg.current_value_pause),
            initial_value_countdown: args.countdown.unwrap_or(stg.inital_value_countdown),
            // invalidate `current_value_countdown` if an initial value is set via args
            current_value_countdown: args.countdown.unwrap_or(stg.current_value_countdown),
            elapsed_value_countdown: stg.elapsed_value_countdown,
            current_value_timer: stg.current_value_timer,
        }
    }
}

fn get_app_time() -> AppTime {
    match OffsetDateTime::now_local() {
        Ok(t) => AppTime::Local(t),
        Err(_) => AppTime::Utc(OffsetDateTime::now_utc()),
    }
}

impl App {
    pub fn new(args: AppArgs) -> Self {
        let AppArgs {
            style,
            show_menu,
            app_time_format,
            initial_value_work,
            initial_value_pause,
            initial_value_countdown,
            current_value_work,
            current_value_pause,
            current_value_countdown,
            elapsed_value_countdown,
            current_value_timer,
            content,
            with_decis,
            pomodoro_mode,
        } = args;
        Self {
            mode: Mode::Running,
            content,
            app_time: get_app_time(),
            style,
            with_decis,
            countdown: CountdownState::new(
                ClockState::<clock::Countdown>::new(ClockStateArgs {
                    initial_value: initial_value_countdown,
                    current_value: current_value_countdown,
                    tick_value: Duration::from_millis(TICK_VALUE_MS),
                    with_decis,
                }),
                elapsed_value_countdown,
            ),
            timer: TimerState::new(ClockState::<clock::Timer>::new(ClockStateArgs {
                initial_value: Duration::ZERO,
                current_value: current_value_timer,
                tick_value: Duration::from_millis(TICK_VALUE_MS),
                with_decis,
            })),
            pomodoro: PomodoroState::new(PomodoroStateArgs {
                mode: pomodoro_mode,
                initial_value_work,
                current_value_work,
                initial_value_pause,
                current_value_pause,
                with_decis,
            }),
            footer: FooterState::new(show_menu, app_time_format),
        }
    }

    pub async fn run(mut self, mut terminal: Terminal, mut events: Events) -> Result<Self> {
        while self.is_running() {
            if let Some(event) = events.next().await {
                if matches!(event, Event::Tick) {
                    self.app_time = get_app_time();
                }

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
            Content::Countdown => self.countdown.is_running(),
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
            // toogle app time format
            KeyCode::Char(':') => self.footer.toggle_app_time_format(),
            // toogle menu
            KeyCode::Char('m') => self.footer.set_show_menu(!self.footer.get_show_menu()),
            KeyCode::Char(',') => {
                self.style = self.style.next();
            }
            KeyCode::Char('.') => {
                self.with_decis = !self.with_decis;
                // update clocks
                self.timer.set_with_decis(self.with_decis);
                self.countdown.set_with_decis(self.with_decis);
                self.pomodoro.set_with_decis(self.with_decis);
            }
            KeyCode::Up => self.footer.set_show_menu(true),
            KeyCode::Down => self.footer.set_show_menu(false),
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
            show_menu: self.footer.get_show_menu(),
            app_time_format: *self.footer.app_time_format(),
            style: self.style,
            with_decis: self.with_decis,
            pomodoro_mode: self.pomodoro.get_mode().clone(),
            inital_value_work: Duration::from(*self.pomodoro.get_clock_work().get_initial_value()),
            current_value_work: Duration::from(*self.pomodoro.get_clock_work().get_current_value()),
            inital_value_pause: Duration::from(
                *self.pomodoro.get_clock_pause().get_initial_value(),
            ),
            current_value_pause: Duration::from(
                *self.pomodoro.get_clock_pause().get_current_value(),
            ),
            inital_value_countdown: Duration::from(*self.countdown.get_clock().get_initial_value()),
            current_value_countdown: Duration::from(
                *self.countdown.get_clock().get_current_value(),
            ),
            elapsed_value_countdown: Duration::from(*self.countdown.get_elapsed_value()),
            current_value_timer: Duration::from(*self.timer.get_clock().get_current_value()),
        }
    }
}

struct AppWidget;

impl AppWidget {
    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &mut App) {
        match state.content {
            Content::Timer => {
                Timer { style: state.style }.render(area, buf, &mut state.timer);
            }
            Content::Countdown => {
                Countdown { style: state.style }.render(area, buf, &mut state.countdown)
            }
            Content::Pomodoro => {
                PomodoroWidget { style: state.style }.render(area, buf, &mut state.pomodoro)
            }
        };
    }
}

impl StatefulWidget for AppWidget {
    type State = App;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [v0, v1, v2] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(if state.footer.get_show_menu() { 4 } else { 1 }),
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
            running_clock: state.clock_is_running(),
            selected_content: state.content,
            edit_mode: state.is_edit_mode(),
            app_time: state.app_time,
        }
        .render(v2, buf, &mut state.footer);
    }
}
