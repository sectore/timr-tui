use crate::{
    clock::{self, Clock},
    constants::TICK_VALUE_MS,
    events::{Event, EventHandler, Events},
    terminal::Terminal,
    utils::center,
    widgets::{
        countdown::{Countdown, CountdownWidget},
        footer::Footer,
        header::Header,
        pomodoro::Pomodoro,
        timer::{Timer, TimerWidget},
    },
};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, StatefulWidget, Widget},
};
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
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Running,
            content: Content::Countdown,
            show_menu: false,
            countdown: Countdown::new(
                "Countdown".into(),
                Clock::<clock::Countdown>::new(
                    10 * 60 * 1000, /* 10min in milliseconds */
                    TICK_VALUE_MS,
                ),
            ),
            timer: Timer::new("Timer".into(), Clock::<clock::Timer>::new(0, TICK_VALUE_MS)),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(&mut self, mut terminal: Terminal, mut events: Events) -> Result<()> {
        while self.is_running() {
            if let Some(event) = events.next().await {
                match self.content {
                    Content::Countdown => self.countdown.update(event.clone()),
                    Content::Timer => self.timer.update(event.clone()),
                    _ => {}
                };
                match event {
                    Event::Render | Event::Resize => {
                        self.draw(&mut terminal)?;
                    }
                    Event::Key(key) => self.handle_key_event(key),
                    _ => {}
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
        // center content
        let area = center(area, Constraint::Length(50), Constraint::Length(2));
        match state.content {
            Content::Timer => TimerWidget.render(area, buf, &mut state.timer),
            Content::Countdown => CountdownWidget.render(area, buf, &mut state.countdown),
            Content::Pomodoro => Pomodoro::new("Pomodoro".into()).render(area, buf),
        };
    }
}

impl StatefulWidget for AppWidget {
    type State = App;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(0),
            Constraint::Length(if state.show_menu { 2 } else { 1 }),
        ]);
        let [v0, v1, v4] = vertical.areas(area);

        Block::new().render(area, buf);
        Header::new(true).render(v0, buf);
        self.render_content(v1, buf, state);
        Footer::new(state.show_menu, state.content).render(v4, buf);
    }
}
