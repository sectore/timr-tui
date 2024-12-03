mod app;
mod clock;
mod constants;
mod events;
mod logging;
mod terminal;
mod utils;
mod widgets;

use app::App;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init()?;
    color_eyre::install()?;

    let terminal = terminal::setup()?;
    let events = events::Events::new();
    App::new().run(terminal, events).await?;
    terminal::teardown()?;

    Ok(())
}
