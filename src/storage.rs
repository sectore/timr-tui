use crate::constants::APP_NAME;
use crate::widgets::pomodoro::Mode as PomodoroMode;
use crate::{
    app::App,
    args::{ClockStyle, Content},
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
    pub clock_style: ClockStyle,
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
            show_menu: false,
            clock_style: ClockStyle::default(),
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
            // timer
            current_value_timer: Duration::ZERO,
        }
    }
}

impl From<App> for AppStorage {
    fn from(app: App) -> Self {
        AppStorage {
            content: app.content,
            show_menu: app.show_menu,
            clock_style: app.clock_style,
            with_decis: app.with_decis,
            pomodoro_mode: app.pomodoro.get_mode().clone(),
            inital_value_work: app.pomodoro.get_clock_work().initial_value,
            current_value_work: app.pomodoro.get_clock_work().current_value,
            inital_value_pause: app.pomodoro.get_clock_pause().initial_value,
            current_value_pause: app.pomodoro.get_clock_pause().current_value,
            inital_value_countdown: app.countdown.clock.initial_value,
            current_value_countdown: app.countdown.clock.current_value,
            current_value_timer: app.timer.clock.current_value,
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
        self.data_dir.join(format!("{}.data", APP_NAME))
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
