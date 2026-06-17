use std::time::Duration;

pub static APP_NAME: &str = env!("CARGO_PKG_NAME");

pub static TICK_VALUE_MS: u64 = 1000 / 10; // 0.1 sec in milliseconds

pub static TABATA_WORK: Duration = Duration::from_secs(20);
pub static TABATA_PAUSE: Duration = Duration::from_secs(10);
pub static TABATA_MAX_ROUNDS: u64 = 8;
