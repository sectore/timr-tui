use crate::{
    args::Args,
    common::{AppEditMode, AppTime, AppTimeFormat, ClockTypeId, Content, Style, Toggle},
    constants::TICK_VALUE_MS,
    events::{self, TuiEventHandler},
    storage::AppStorage,
    terminal::Terminal,
    widgets::{
        clock::{self, ClockState, ClockStateArgs},
        countdown::{Countdown, CountdownState, CountdownStateArgs},
        footer::{Footer, FooterState},
        header::Header,
        pomodoro::{Mode as PomodoroMode, PomodoroState, PomodoroStateArgs, PomodoroWidget},
        timer::{Timer, TimerState},
    },
};

#[cfg(feature = "sound")]
use crate::sound::Sound;

use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};
use std::path::PathBuf;
use std::time::Duration;
use time::OffsetDateTime;
use tracing::{debug, error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Running,
    Quit,
}

pub struct App {
    content: Content,
    mode: Mode,
    notification: Toggle,
    blink: Toggle,
    #[allow(dead_code)] // w/ `--features sound` available only
    sound_path: Option<PathBuf>,
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
    pub notification: Toggle,
    pub blink: Toggle,
    pub show_menu: bool,
    pub app_time_format: AppTimeFormat,
    pub content: Content,
    pub pomodoro_mode: PomodoroMode,
    pub pomodoro_round: u64,
    pub initial_value_work: Duration,
    pub current_value_work: Duration,
    pub initial_value_pause: Duration,
    pub current_value_pause: Duration,
    pub initial_value_countdown: Duration,
    pub current_value_countdown: Duration,
    pub elapsed_value_countdown: Duration,
    pub current_value_timer: Duration,
    pub app_tx: events::AppEventTx,
    pub sound_path: Option<PathBuf>,
}

pub struct FromAppArgs {
    pub args: Args,
    pub stg: AppStorage,
    pub app_tx: events::AppEventTx,
}

/// Creates an `App` by merging `Args` and `AppStorage` (`Args` wins)
/// and adding `AppEventTx`
impl From<FromAppArgs> for App {
    fn from(args: FromAppArgs) -> Self {
        let FromAppArgs { args, stg, app_tx } = args;

        App::new(AppArgs {
            with_decis: args.decis || stg.with_decis,
            show_menu: args.menu || stg.show_menu,
            notification: args.notification.unwrap_or(stg.notification),
            blink: args.blink.unwrap_or(stg.blink),
            app_time_format: stg.app_time_format,
            content: args.mode.unwrap_or(stg.content),
            style: args.style.unwrap_or(stg.style),
            pomodoro_mode: stg.pomodoro_mode,
            pomodoro_round: stg.pomodoro_count,
            initial_value_work: args.work.unwrap_or(stg.inital_value_work),
            // invalidate `current_value_work` if an initial value is set via args
            current_value_work: args.work.unwrap_or(stg.current_value_work),
            initial_value_pause: args.pause.unwrap_or(stg.inital_value_pause),
            // invalidate `current_value_pause` if an initial value is set via args
            current_value_pause: args.pause.unwrap_or(stg.current_value_pause),
            initial_value_countdown: args.countdown.unwrap_or(stg.inital_value_countdown),
            // invalidate `current_value_countdown` if an initial value is set via args
            current_value_countdown: args.countdown.unwrap_or(stg.current_value_countdown),
            elapsed_value_countdown: match args.countdown {
                // reset value if countdown is set by arguments
                Some(_) => Duration::ZERO,
                None => stg.elapsed_value_countdown,
            },
            current_value_timer: stg.current_value_timer,
            app_tx,
            #[cfg(feature = "sound")]
            sound_path: args.sound,
            #[cfg(not(feature = "sound"))]
            sound_path: None,
        })
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
            pomodoro_round,
            notification,
            blink,
            sound_path,
            app_tx,
        } = args;
        let app_time = get_app_time();

        Self {
            mode: Mode::Running,
            notification,
            blink,
            sound_path,
            content,
            app_time,
            style,
            with_decis,
            countdown: CountdownState::new(CountdownStateArgs {
                initial_value: initial_value_countdown,
                current_value: current_value_countdown,
                elapsed_value: elapsed_value_countdown,
                app_time,
                with_decis,
                app_tx: app_tx.clone(),
            }),
            timer: TimerState::new(
                ClockState::<clock::Timer>::new(ClockStateArgs {
                    initial_value: Duration::ZERO,
                    current_value: current_value_timer,
                    tick_value: Duration::from_millis(TICK_VALUE_MS),
                    with_decis,
                    app_tx: Some(app_tx.clone()),
                })
                .with_name("Timer".to_owned()),
            ),
            pomodoro: PomodoroState::new(PomodoroStateArgs {
                mode: pomodoro_mode,
                initial_value_work,
                current_value_work,
                initial_value_pause,
                current_value_pause,
                with_decis,
                round: pomodoro_round,
                app_tx: app_tx.clone(),
            }),
            footer: FooterState::new(show_menu, app_time_format),
        }
    }

    pub async fn run(
        mut self,
        terminal: &mut Terminal,
        mut events: events::Events,
    ) -> Result<Self> {
        // Closure to handle `KeyEvent`'s
        let handle_key_event = |app: &mut Self, key: KeyEvent| {
            debug!("Received key {:?}", key.code);
            match key.code {
                KeyCode::Char('q') => app.mode = Mode::Quit,
                KeyCode::Char('c') => app.content = Content::Countdown,
                KeyCode::Char('t') => app.content = Content::Timer,
                KeyCode::Char('p') => app.content = Content::Pomodoro,
                // toogle app time format
                KeyCode::Char(':') => app.footer.toggle_app_time_format(),
                // toogle menu
                KeyCode::Char('m') => app.footer.set_show_menu(!app.footer.get_show_menu()),
                KeyCode::Char(',') => {
                    app.style = app.style.next();
                }
                KeyCode::Char('.') => {
                    app.with_decis = !app.with_decis;
                    // update clocks
                    app.timer.set_with_decis(app.with_decis);
                    app.countdown.set_with_decis(app.with_decis);
                    app.pomodoro.set_with_decis(app.with_decis);
                }
                KeyCode::Up => app.footer.set_show_menu(true),
                KeyCode::Down => app.footer.set_show_menu(false),
                _ => {}
            };
        };
        // Closure to handle `TuiEvent`'s
        let mut handle_tui_events = |app: &mut Self, event: events::TuiEvent| -> Result<()> {
            if matches!(event, events::TuiEvent::Tick) {
                app.app_time = get_app_time();
                app.countdown.set_app_time(app.app_time);
            }

            // Pipe events into subviews and handle only 'unhandled' events afterwards
            if let Some(unhandled) = match app.content {
                Content::Countdown => app.countdown.update(event.clone()),
                Content::Timer => app.timer.update(event.clone()),
                Content::Pomodoro => app.pomodoro.update(event.clone()),
            } {
                match unhandled {
                    events::TuiEvent::Render | events::TuiEvent::Resize => {
                        app.draw(terminal)?;
                    }
                    events::TuiEvent::Key(key) => handle_key_event(app, key),
                    _ => {}
                }
            }
            Ok(())
        };

        #[allow(unused_variables)] // `app` is used by `--features sound` only
        // Closure to handle `AppEvent`'s
        let handle_app_events = |app: &mut Self, event: events::AppEvent| -> Result<()> {
            match event {
                events::AppEvent::ClockDone(type_id, name) => {
                    debug!("AppEvent::ClockDone");

                    if app.notification == Toggle::On {
                        let msg = match type_id {
                            ClockTypeId::Timer => {
                                format!("{name} stopped by reaching its maximum value.")
                            }
                            _ => format!("{:?} {name} done!", type_id),
                        };
                        // notification
                        let result = notify_rust::Notification::new()
                            .summary(&msg.to_uppercase())
                            .show();
                        if let Err(err) = result {
                            error!("on_done {name} error: {err}");
                        }
                    };

                    #[cfg(feature = "sound")]
                    if let Some(path) = app.sound_path.clone() {
                        _ = Sound::new(path).and_then(|sound| sound.play()).or_else(
                            |err| -> Result<()> {
                                error!("Sound error: {:?}", err);
                                Ok(())
                            },
                        );
                    }
                }
            }
            Ok(())
        };

        while self.is_running() {
            if let Some(event) = events.next().await {
                let _ = match event {
                    events::Event::Terminal(e) => handle_tui_events(&mut self, e),
                    events::Event::App(e) => handle_app_events(&mut self, e),
                };
            }
        }
        Ok(self)
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    fn get_edit_mode(&self) -> AppEditMode {
        match self.content {
            Content::Countdown => {
                if self.countdown.is_clock_edit_mode() {
                    AppEditMode::Clock
                } else if self.countdown.is_time_edit_mode() {
                    AppEditMode::Time
                } else {
                    AppEditMode::None
                }
            }

            Content::Timer => {
                if self.timer.get_clock().is_edit_mode() {
                    AppEditMode::Clock
                } else {
                    AppEditMode::None
                }
            }
            Content::Pomodoro => {
                if self.pomodoro.get_clock().is_edit_mode() {
                    AppEditMode::Clock
                } else {
                    AppEditMode::None
                }
            }
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
            notification: self.notification,
            blink: self.blink,
            app_time_format: *self.footer.app_time_format(),
            style: self.style,
            with_decis: self.with_decis,
            pomodoro_mode: self.pomodoro.get_mode().clone(),
            pomodoro_count: self.pomodoro.get_round(),
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
                Timer {
                    style: state.style,
                    blink: state.blink == Toggle::On,
                }
                .render(area, buf, &mut state.timer);
            }
            Content::Countdown => Countdown {
                style: state.style,
                blink: state.blink == Toggle::On,
            }
            .render(area, buf, &mut state.countdown),
            Content::Pomodoro => PomodoroWidget {
                style: state.style,
                blink: state.blink == Toggle::On,
            }
            .render(area, buf, &mut state.pomodoro),
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
            app_edit_mode: state.get_edit_mode(),
            app_time: state.app_time,
        }
        .render(v2, buf, &mut state.footer);
    }
}
