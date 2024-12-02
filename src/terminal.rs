use std::io::{stdout, Stdout};

use color_eyre::eyre::Result;
use crossterm::{execute, terminal::*};
use ratatui::{backend::CrosstermBackend, Terminal as RatatuiTerminal};

pub type Terminal = RatatuiTerminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Terminal> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = RatatuiTerminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub fn restore() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
