mod app;
mod clock;
mod config;
mod constants;
mod events;
#[cfg(debug_assertions)]
mod logging;

mod terminal;
mod utils;
mod widgets;

use app::App;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::init()?;
    #[cfg(debug_assertions)]
    logging::Logger::new(config.log_dir).init()?;

    color_eyre::install()?;

    let terminal = terminal::setup()?;
    let events = events::Events::new();
    App::new().run(terminal, events).await?;
    terminal::teardown()?;

    Ok(())
}
