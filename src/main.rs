mod app;
mod config;
mod constants;
mod events;
#[cfg(debug_assertions)]
mod logging;

mod args;
mod duration;
mod storage;
mod terminal;
mod utils;
mod widgets;

use app::App;
use args::Args;
use clap::Parser;
use color_eyre::Result;
use config::Config;
use storage::{AppStorage, Storage};

#[tokio::main]
async fn main() -> Result<()> {
    let Config { log_dir, data_dir } = Config::init()?;
    #[cfg(debug_assertions)]
    logging::Logger::new(log_dir).init()?;

    color_eyre::install()?;

    let terminal = terminal::setup()?;
    let events = events::Events::new();

    // get args given by CLI
    let args = Args::parse();

    // check persistant storage
    let storage = Storage::new(data_dir);
    // option to reset previous stored data to `default`
    let stg = if args.reset {
        AppStorage::default()
    } else {
        storage.load().unwrap_or_default()
    };

    // merge `Args` and `AppStorage`.
    let app_args = (args, stg).into();
    let app_storage = App::new(app_args).run(terminal, events).await?.to_storage();
    // store app state persistantly
    storage.save(app_storage)?;

    terminal::teardown()?;

    Ok(())
}
