pub static APP_NAME: &str = env!("CARGO_PKG_NAME");

pub static TICK_VALUE_MS: u64 = 1000 / 10; // 0.1 sec in milliseconds
pub static FPS_VALUE_MS: u64 = 1000 / 60; // 60 FPS in milliseconds

pub static LABEL_DAYS: &str = "d";
pub static LABEL_YEARS: &str = "y";
