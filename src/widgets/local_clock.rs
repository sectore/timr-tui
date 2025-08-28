use crate::common::{AppTime, AppTimeFormat, Style};

/// State for LocalClock Widget
pub struct LocalClockState {
    app_time: AppTime,
    app_time_format: AppTimeFormat,
}

impl LocalClockState {
    pub const fn new(app_time: AppTime, app_time_format: AppTimeFormat) -> Self {
        Self {
            app_time,
            app_time_format,
        }
    }

    pub const fn set_app_time_format(&mut self, value: AppTimeFormat) {
        self.app_time_format = value;
    }
}

#[derive(Debug)]
pub struct LocalClock {
    pub style: Style,
}
