use crate::{
    args::Args,
    common::{AppEditMode, AppTime, AppTimeFormat, ClockTypeId, Content, Style, Toggle},
    constants::TICK_VALUE_MS,
    event::{Event, get_default_event},
    events::{self, TuiEventHandler},
    storage::AppStorage,
    terminal::Terminal,
    widgets::{
        clock::{self, ClockState, ClockStateArgs},
        countdown::{Countdown, CountdownState, CountdownStateArgs},
        event::{EventState, EventStateArgs, EventWidget},
        footer::{Footer, FooterState},
        header::Header,
        local_time::{LocalTimeState, LocalTimeStateArgs, LocalTimeWidget},
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
    app_time_format: AppTimeFormat,
    countdown: CountdownState,
    timer: TimerState,
    pomodoro: PomodoroState,
    event: EventState,
    local_time: LocalTimeState,
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
    pub event: Event,
    pub app_tx: events::AppEventTx,
    pub sound_path: Option<PathBuf>,
    pub footer_toggle_app_time: Toggle,
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
            // Check args to set a possible mode to start with.
            content: match args.mode {
                Some(mode) => mode,
                // check other args (especially durations)
                None => {
                    if args.work.is_some() || args.pause.is_some() {
                        Content::Pomodoro
                    } else if args.countdown.is_some() {
                        Content::Countdown
                    } else if args.event.is_some() {
                        Content::Event
                    }
                    // in other case just use latest stored state
                    else {
                        stg.content
                    }
                }
            },
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
            current_value_countdown: args.countdown.unwrap_or(stg.inital_value_countdown),
            elapsed_value_countdown: match args.countdown {
                // reset value if countdown is set by arguments
                Some(_) => Duration::ZERO,
                None => stg.elapsed_value_countdown,
            },
            current_value_timer: stg.current_value_timer,
            event: args.event.unwrap_or_else(get_default_event),
            app_tx,
            #[cfg(feature = "sound")]
            sound_path: args.sound,
            #[cfg(not(feature = "sound"))]
            sound_path: None,
            footer_toggle_app_time: stg.footer_app_time,
        })
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
            event,
            notification,
            blink,
            sound_path,
            app_tx,
            footer_toggle_app_time,
        } = args;
        let app_time = AppTime::new();

        Self {
            mode: Mode::Running,
            notification,
            blink,
            sound_path,
            content,
            app_time,
            app_time_format,
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
            local_time: LocalTimeState::new(LocalTimeStateArgs {
                app_time,
                app_time_format,
            }),
            event: EventState::new(EventStateArgs {
                app_time,
                event,
                with_decis,
                app_tx: app_tx.clone(),
            }),
            footer: FooterState::new(
                show_menu,
                if footer_toggle_app_time == Toggle::On {
                    Some(app_time_format)
                } else {
                    None
                },
            ),
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
                KeyCode::Char('1') | KeyCode::Char('c') /* TODO: deprecated, remove it in next verson */ => app.content = Content::Countdown,
                KeyCode::Char('2') | KeyCode::Char('t') /* TODO: deprecated, remove it in next verson */ => app.content = Content::Timer,
                KeyCode::Char('3') | KeyCode::Char('p') /* TODO: deprecated, remove it in next verson */ => app.content = Content::Pomodoro,
                KeyCode::Char('4') => app.content = Content::Event,
                KeyCode::Char('0') | KeyCode::Char('l') /* TODO: deprecated, remove it in next verson */ => app.content = Content::LocalTime,
                // toogle app time format
                KeyCode::Char(':') => {
                    if app.content == Content::LocalTime {
                        // For LocalTime content: just cycle through formats
                        app.app_time_format = app.app_time_format.next();
                        app.local_time.set_app_time_format(app.app_time_format);
                        // Only update footer if it's currently showing time
                        if app.footer.app_time_format().is_some() {
                            app.footer.set_app_time_format(Some(app.app_time_format));
                        }
                    } else {
                        // For other content: allow footer to toggle between formats and None
                        let new_format = match app.footer.app_time_format() {
                            // footer is hidden -> show first format
                            None => Some(AppTimeFormat::first()),
                            Some(v) => {
                                if v != &AppTimeFormat::last() {
                                    Some(v.next())
                                } else {
                                    // reached last format -> hide footer time
                                    None
                                }
                            }
                        };

                        if let Some(format) = new_format {
                            app.app_time_format = format;
                            app.local_time.set_app_time_format(format);
                        }
                        app.footer.set_app_time_format(new_format);
                    }
                }
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
                    app.event.set_with_decis(app.with_decis);
                }
                KeyCode::Up => app.footer.set_show_menu(true),
                KeyCode::Down => app.footer.set_show_menu(false),
                _ => {}
            };
        };
        // Closure to handle `TuiEvent`'s
        let mut handle_tui_events = |app: &mut Self, event: events::TuiEvent| -> Result<()> {
            if matches!(event, events::TuiEvent::Tick) {
                app.app_time = AppTime::new();
                app.countdown.set_app_time(app.app_time);
                app.local_time.set_app_time(app.app_time);
                app.event.set_app_time(app.app_time);
            }

            // Pipe events into subviews and handle only 'unhandled' events afterwards
            if let Some(unhandled) = match app.content {
                Content::Countdown => app.countdown.update(event.clone()),
                Content::Timer => app.timer.update(event.clone()),
                Content::Pomodoro => app.pomodoro.update(event.clone()),
                Content::Event => app.event.update(event.clone()),
                Content::LocalTime => app.local_time.update(event.clone()),
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
                            _ => format!("{type_id:?} {name} done!"),
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
            Content::Event => AppEditMode::None,
            Content::LocalTime => AppEditMode::None,
        }
    }

    fn clock_is_running(&self) -> bool {
        match self.content {
            Content::Countdown => self.countdown.is_running(),
            Content::Timer => self.timer.get_clock().is_running(),
            Content::Pomodoro => self.pomodoro.get_clock().is_running(),
            // Event clock runs forever
            Content::Event => true,
            // `LocalTime` does not use a `Clock`
            Content::LocalTime => false,
        }
    }

    fn get_percentage_done(&self) -> Option<u16> {
        match self.content {
            Content::Countdown => Some(self.countdown.get_clock().get_percentage_done()),
            Content::Timer => None,
            Content::Pomodoro => Some(self.pomodoro.get_clock().get_percentage_done()),
            Content::Event => Some(self.event.get_percentage_done()),
            Content::LocalTime => None,
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
            app_time_format: self.app_time_format,
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
            footer_app_time: self.footer.app_time_format().is_some().into(),
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
            Content::Event => EventWidget {
                style: state.style,
                blink: state.blink == Toggle::On,
            }
            .render(area, buf, &mut state.event),
            Content::LocalTime => {
                LocalTimeWidget { style: state.style }.render(area, buf, &mut state.local_time);
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
            Constraint::Length(if state.footer.get_show_menu() { 5 } else { 1 }),
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
