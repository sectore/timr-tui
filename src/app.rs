use crate::{
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
    tick: u128,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Running,
            content: Content::Countdown,
            show_menu: false,
            tick: 0,
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
                        self.tick = self.tick.saturating_add(1);
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
            KeyCode::Char('t') => self.content = Content::Timer,
            KeyCode::Char('p') => self.content = Content::Pomodoro,
            KeyCode::Char('m') => self.show_menu = !self.show_menu,
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
        let area = center(area, Constraint::Length(50), Constraint::Length(1));
        match self.content {
            Content::Timer => Timer::new(200, "Timer".into()).render(area, buf),
            Content::Countdown => Countdown::new("Countdown".into()).render(area, buf),
            Content::Pomodoro => Pomodoro::new("Pomodoro".into()).render(area, buf),
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
        let [header_area, content_area, footer_area] = vertical.areas(area);

        Block::new().render(area, buf);
        Header::new(self.tick).render(header_area, buf);
        self.render_content(content_area, buf);
        Footer::new(self.show_menu, self.content).render(footer_area, buf);
    }
}
