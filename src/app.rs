use color_eyre::{eyre::Context, Result};
use crossterm::event;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use strum::{Display, EnumIter, FromRepr};

use crate::footer::Footer;
use crate::pomodoro::Pomodoro;
use crate::timer::Timer;
use crate::{countdown::Countdown, utils::center};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Running,
    Quit,
}

#[derive(Debug, Clone, Copy, Display, EnumIter, FromRepr, PartialEq, Eq, PartialOrd, Ord)]
pub enum Content {
    Countdown,
    Timer,
    Pomodoro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct App {
    content: Content,
    mode: Mode,
    show_menu: bool,
}

impl App {
    pub const fn new() -> Self {
        Self {
            content: Content::Countdown,
            mode: Mode::Running,
            show_menu: false,
        }
    }

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
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
                KeyCode::Char('c') => self.content = Content::Countdown,
                KeyCode::Char('t') => self.content = Content::Timer,
                KeyCode::Char('p') => self.content = Content::Pomodoro,
                KeyCode::Char('m') => self.show_menu = !self.show_menu,
                _ => {}
            };
        }
        Ok(())
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
        self.render_header(header_area, buf);
        self.render_content(content_area, buf);
        Footer::new(self.show_menu, self.content).render(footer_area, buf);
    }
}

impl App {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("tim:r").render(area, buf);
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
