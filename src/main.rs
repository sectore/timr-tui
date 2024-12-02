mod app;
mod clock;
mod constants;
mod events;
mod terminal;
mod utils;
mod widgets;

use app::App;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = terminal::init()?;

    let events = events::Events::new();
    App::new().run(terminal, events).await?;
    terminal::restore()?;
    Ok(())
}
