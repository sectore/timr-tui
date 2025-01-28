use crate::{
    common::{AppTimeFormat, Content, Notification, Style},
    widgets::pomodoro::Mode as PomodoroMode,
};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppStorage {
    pub content: Content,
    pub show_menu: bool,
    pub notification: Notification,
    pub app_time_format: AppTimeFormat,
    pub style: Style,
    pub with_decis: bool,
    pub pomodoro_mode: PomodoroMode,
    // pomodoro -> work
    pub inital_value_work: Duration,
    pub current_value_work: Duration,
    // pomodoro -> pause
    pub inital_value_pause: Duration,
    pub current_value_pause: Duration,
    // countdown
    pub inital_value_countdown: Duration,
    pub current_value_countdown: Duration,
    pub elapsed_value_countdown: Duration,
    // timer
    pub current_value_timer: Duration,
}

impl Default for AppStorage {
    fn default() -> Self {
        const DEFAULT_WORK: Duration = Duration::from_secs(60 * 25); /* 25min */
        const DEFAULT_PAUSE: Duration = Duration::from_secs(60 * 5); /* 5min */
        const DEFAULT_COUNTDOWN: Duration = Duration::from_secs(60 * 10); /* 10min */
        AppStorage {
            content: Content::default(),
            show_menu: true,
            notification: Notification::Off,
            app_time_format: AppTimeFormat::default(),
            style: Style::default(),
            with_decis: false,
            pomodoro_mode: PomodoroMode::Work,
            // pomodoro -> work
            inital_value_work: DEFAULT_WORK,
            current_value_work: DEFAULT_WORK,
            // pomodoro -> pause
            inital_value_pause: DEFAULT_PAUSE,
            current_value_pause: DEFAULT_PAUSE,
            // countdown
            inital_value_countdown: DEFAULT_COUNTDOWN,
            current_value_countdown: DEFAULT_COUNTDOWN,
            elapsed_value_countdown: Duration::ZERO,
            // timer
            current_value_timer: Duration::ZERO,
        }
    }
}

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn get_storage_path(&self) -> PathBuf {
        self.data_dir.join("app.data")
    }

    pub fn save(&self, data: AppStorage) -> Result<()> {
        let file = fs::File::create(self.get_storage_path())?;
        serde_json::to_writer(file, &data)?;
        Ok(())
    }

    pub fn load(&self) -> Result<AppStorage> {
        let file = fs::File::open(self.get_storage_path())?;
        let data = serde_json::from_reader(file)?;
        Ok(data)
    }
}
