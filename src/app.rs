use crate::{
    args::Args,
    constants::TICK_VALUE_MS,
    events::{Event, EventHandler, Events},
    terminal::Terminal,
    widgets::{
        clock::{self, Clock},
        countdown::{Countdown, CountdownWidget},
        footer::Footer,
        header::Header,
        pomodoro::{Pomodoro, PomodoroArgs, PomodoroWidget},
        timer::{Timer, TimerWidget},
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
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Running,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Content {
    Countdown,
    Timer,
    Pomodoro,
}

#[derive(Debug)]
pub struct App {
    content: Content,
    mode: Mode,
    show_menu: bool,
    countdown: Countdown,
    timer: Timer,
    pomodoro: Pomodoro,
}

impl App {
    pub fn new(args: Args) -> Self {
        Self {
            mode: Mode::Running,
            content: Content::Countdown,
            show_menu: false,
            countdown: Countdown::new(Clock::<clock::Countdown>::new(
                args.countdown,
                Duration::from_millis(TICK_VALUE_MS),
            )),
            timer: Timer::new(Clock::<clock::Timer>::new(
                Duration::ZERO,
                Duration::from_millis(TICK_VALUE_MS),
            )),
            pomodoro: Pomodoro::new(PomodoroArgs {
                work: args.work,
                pause: args.pause,
            }),
        }
    }

    pub async fn run(&mut self, mut terminal: Terminal, mut events: Events) -> Result<()> {
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
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        debug!("Received key {:?}", key.code);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('c') => self.content = Content::Countdown,
            KeyCode::Char('t') => self.content = Content::Timer,
            KeyCode::Char('p') => self.content = Content::Pomodoro,
            KeyCode::Char('m') => self.show_menu = !self.show_menu,
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
}

struct AppWidget;

impl AppWidget {
    fn render_content(&self, area: Rect, buf: &mut Buffer, state: &mut App) {
        match state.content {
            Content::Timer => TimerWidget.render(area, buf, &mut state.timer),
            Content::Countdown => CountdownWidget.render(area, buf, &mut state.countdown),
            Content::Pomodoro => PomodoroWidget.render(area, buf, &mut state.pomodoro),
        };
    }
}

impl StatefulWidget for AppWidget {
    type State = App;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(if state.show_menu { 2 } else { 1 }),
        ]);
        let [v0, v1, v2] = vertical.areas(area);

        Header::new(true).render(v0, buf);
        self.render_content(v1, buf, state);
        Footer::new(state.show_menu, state.content).render(v2, buf);
    }
}
