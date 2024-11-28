use color_eyre::{eyre::Context, Result};
use crossterm::event;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::time::Duration;
use strum::{Display, EnumIter, FromRepr};

use crate::pomodoro::Pomodoro;
use crate::timer::Timer;
use crate::{countdown::Countdown, utils::center};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Running,
    Quit,
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
enum Content {
    #[default]
    Countdown,
    Timer,
    Pomodoro,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct App {
    content: Content,
    mode: Mode,
}

impl App {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.is_running() {
            terminal
                .draw(|frame| self.draw(frame))
                .wrap_err("terminal.draw")?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    /// Draw a single frame of the app.
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        if !event::poll(timeout)? {
            return Ok(());
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_press(key),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('c') => self.content = Content::Countdown,
            KeyCode::Char('t') => self.content = Content::Timer,
            KeyCode::Char('p') => self.content = Content::Pomodoro,
            _ => {}
        };
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]);
        let [header_area, content_area, footer_area] = vertical.areas(area);

        Block::new().render(area, buf);
        self.render_header(header_area, buf);
        self.render_content(content_area, buf);
        self.render_footer(footer_area, buf);
    }
}

impl App {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("tim:r").render(area, buf);
    }
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("footer").render(area, buf);
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
