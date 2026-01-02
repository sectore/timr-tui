mod app;
mod common;
mod config;
mod constants;
mod event;
mod events;
mod logging;

mod args;
mod duration;
mod storage;
mod terminal;
mod widgets;

#[cfg(feature = "sound")]
mod sound;

use app::{App, FromAppArgs};
use args::{Args, LOG_DIRECTORY_DEFAULT_MISSING_VALUE};
use clap::Parser;
use color_eyre::Result;
use config::Config;
use std::path::PathBuf;
use storage::{AppStorage, Storage};

#[tokio::main]
async fn main() -> Result<()> {
    // init `Config`
    let cfg = Config::init()?;

    color_eyre::install()?;

    // get args given by CLI
    let args = Args::parse();
    // Note:
    // `log` arg can have three different values:
    // (1) not set => None
    // (2) set with path => Some(Some(path))
    // (3) set without path => Some(None)
    let custom_log_dir: Option<Option<&PathBuf>> = if let Some(path) = &args.log {
        if path.ne(PathBuf::from(LOG_DIRECTORY_DEFAULT_MISSING_VALUE).as_os_str()) {
            // (2)
            Some(Some(path))
        } else {
            // (3)
            Some(None)
        }
    } else {
        // (1)
        None
    };

    if let Some(log_dir) = custom_log_dir {
        let dir: PathBuf = log_dir.unwrap_or(&cfg.log_dir).to_path_buf();
        logging::Logger::new(dir).init()?;
    }

    let mut terminal = terminal::setup()?;
    let events = events::Events::new();

    // check persistant storage
    let storage = Storage::new(cfg.data_dir);
    // option to reset previous stored data to `default`
    let stg = if args.reset {
        AppStorage::default()
    } else {
        storage.load().unwrap_or_default()
    };

    let app_storage = App::from(FromAppArgs {
        args,
        stg,
        app_tx: events.get_app_event_tx(),
    })
    .run(&mut terminal, events)
    .await?
    .to_storage();
    // store app state persistantly
    storage.save(app_storage)?;

    terminal::teardown()?;

    Ok(())
}
