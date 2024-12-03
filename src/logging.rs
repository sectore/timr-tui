use color_eyre::eyre::{eyre, Result};
use directories::ProjectDirs;
use std::fs;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    self, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

pub fn init() -> Result<()> {
    let app_name = env!("CARGO_PKG_NAME");
    let proj_dirs = ProjectDirs::from("io", "jkrause", app_name)
        .ok_or_else(|| eyre!("Failed to get project directories"))?;
    let directory = proj_dirs
        .state_dir()
        .ok_or_else(|| eyre!("Failed to get state directory"))?
        .join("logs");
    fs::create_dir_all(&directory)?;
    let log_path = directory.join(format!("{}.log", app_name));
    let log_file = fs::File::create(log_path)?;
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false);
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive(LevelFilter::DEBUG.into());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter)
        .init();
    Ok(())
}
