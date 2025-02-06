use std::io;

use color_eyre::eyre::Result;
use crossterm::{
    cursor, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal as RatatuiTerminal};

pub type Terminal = RatatuiTerminal<CrosstermBackend<io::Stdout>>;

pub fn setup() -> Result<Terminal> {
    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    set_panic_hook();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let mut terminal = RatatuiTerminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub fn teardown() -> Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

// Panic hook
// see https://ratatui.rs/tutorials/counter-app/error-handling/#setup-hooks
fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = teardown(); // ignore any errors as we are already failing
        hook(panic_info);
    }));
}
