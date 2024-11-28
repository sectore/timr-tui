mod app;
mod countdown;
mod footer;
mod pomodoro;
mod timer;
mod utils;

use app::App;
use color_eyre::{eyre::Context, Result};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal).context("app loop failed");
    ratatui::restore();
    app_result
}
