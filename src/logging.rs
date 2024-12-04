use color_eyre::eyre::Result;
use std::fs;
use std::path::PathBuf;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    self, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

use crate::constants::APP_NAME;

pub struct Logger {
    log_dir: PathBuf,
}

impl Logger {
    pub fn new(log_dir: PathBuf) -> Self {
        Self { log_dir }
    }

    pub fn init(&self) -> Result<()> {
        let log_path = self.log_dir.join(format!("{}.log", APP_NAME));
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
}
