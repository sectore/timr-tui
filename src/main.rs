mod app;
mod common;
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

use app::{App, AppArgs};
use args::Args;
use clap::Parser;
use color_eyre::Result;
use config::Config;
use storage::{AppStorage, Storage};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::init()?;
    #[cfg(debug_assertions)]
    logging::Logger::new(cfg.log_dir).init()?;

    color_eyre::install()?;
    // get args given by CLI
    let args = Args::parse();

    let terminal = terminal::setup()?;
    let events = events::Events::new();

    // check persistant storage
    let storage = Storage::new(cfg.data_dir);
    // option to reset previous stored data to `default`
    let stg = if args.reset {
        AppStorage::default()
    } else {
        storage.load().unwrap_or_default()
    };

    // merge `Args` and `AppStorage`.
    let app_args = AppArgs::from((args, stg));
    let app_storage = App::new(app_args).run(terminal, events).await?.to_storage();
    // store app state persistantly
    storage.save(app_storage)?;

    terminal::teardown()?;

    Ok(())
}
