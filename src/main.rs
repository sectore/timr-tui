mod app;
mod config;
mod constants;
mod events;
#[cfg(debug_assertions)]
mod logging;

mod args;
mod terminal;
mod utils;
mod widgets;

use app::App;
use args::Args;
use clap::Parser;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::init()?;
    #[cfg(debug_assertions)]
    logging::Logger::new(config.log_dir).init()?;

    color_eyre::install()?;

    let args = Args::parse();

    let terminal = terminal::setup()?;
    let events = events::Events::new();
    App::new(args).run(terminal, events).await?;
    terminal::teardown()?;

    Ok(())
}
