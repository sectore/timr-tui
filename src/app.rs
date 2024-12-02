use crate::{
    clock::{self, Clock},
    constants::TICK_VALUE_MS,
    events::{Event, Events},
    terminal::Terminal,
    utils::center,
    widgets::{
        countdown::Countdown, footer::Footer, header::Header, pomodoro::Pomodoro, timer::Timer,
    },
};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Widget},
};

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
    clock_countdown: Clock<clock::Countdown>,
    clock_timer: Clock<clock::Timer>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Running,
            content: Content::Countdown,
            show_menu: false,
            clock_countdown: Clock::<clock::Countdown>::new(
                10 * 60 * 1000, /* 10min in milliseconds */
                TICK_VALUE_MS,
            ),
            clock_timer: Clock::<clock::Timer>::new(0, TICK_VALUE_MS),
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
                match event {
                    Event::Render | Event::Resize(_, _) => {
                        self.draw(&mut terminal)?;
                    }
                    Event::Tick => {
                        self.tick();
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
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('c') => self.content = Content::Countdown,
            KeyCode::Char('s') => self.toggle(),
            KeyCode::Char('t') => self.content = Content::Timer,
            KeyCode::Char('p') => self.content = Content::Pomodoro,
            KeyCode::Char('m') => self.show_menu = !self.show_menu,
            KeyCode::Char('r') => self.reset(),
            _ => {}
        };
    }

    fn draw(&self, terminal: &mut Terminal) -> Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(self, frame.area());
        })?;
        Ok(())
    }

    fn render_content(&self, area: Rect, buf: &mut Buffer) {
        // center content
        let area = center(area, Constraint::Length(50), Constraint::Length(2));
        match self.content {
            Content::Timer => {
                Timer::new("Timer".into(), self.clock_timer.clone()).render(area, buf)
            }
            Content::Countdown => {
                Countdown::new("Countdown".into(), self.clock_countdown.clone()).render(area, buf)
            }
            Content::Pomodoro => Pomodoro::new("Pomodoro".into()).render(area, buf),
        };
    }

    fn reset(&mut self) {
        match self.content {
            Content::Timer => self.clock_timer.reset(),
            Content::Countdown => self.clock_countdown.reset(),
            _ => {}
        };
    }

    fn toggle(&mut self) {
        match self.content {
            Content::Timer => self.clock_timer.toggle_pause(),
            Content::Countdown => self.clock_countdown.toggle_pause(),
            _ => {}
        };
    }

    fn tick(&mut self) {
        match self.content {
            Content::Timer => self.clock_timer.tick(),
            Content::Countdown => self.clock_countdown.tick(),
            _ => {}
        };
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(0),
            Constraint::Length(if self.show_menu { 2 } else { 1 }),
        ]);
        let [v0, v1, v4] = vertical.areas(area);

        Block::new().render(area, buf);
        Header::new(true).render(v0, buf);
        self.render_content(v1, buf);
        Footer::new(self.show_menu, self.content).render(v4, buf);
    }
}
