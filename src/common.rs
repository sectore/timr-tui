use clap::ValueEnum;
use ratatui::symbols::shade;
use serde::{Deserialize, Serialize};
use strum::EnumString;
use time::{OffsetDateTime, format_description};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default, Serialize, Deserialize,
)]
pub enum Content {
    #[default]
    #[value(name = "countdown", alias = "c")]
    Countdown,
    #[value(name = "timer", alias = "t")]
    Timer,
    #[value(name = "pomodoro", alias = "p")]
    Pomodoro,
    #[value(name = "localtime", alias = "l")]
    LocalTime,
}

#[derive(Clone, Debug)]
pub enum ClockTypeId {
    Countdown,
    Timer,
}

#[derive(Debug, Copy, Clone, ValueEnum, Default, Serialize, Deserialize)]
pub enum Style {
    #[default]
    #[value(name = "full", alias = "f")]
    Full,
    #[value(name = "light", alias = "l")]
    Light,
    #[value(name = "medium", alias = "m")]
    Medium,
    #[value(name = "dark", alias = "d")]
    Dark,
    #[value(name = "thick", alias = "t")]
    Thick,
    #[value(name = "cross", alias = "c")]
    Cross,
    // https://en.wikipedia.org/wiki/Braille_Patterns
    // Note: Might not be supported in all terminals
    // see https://docs.rs/ratatui/latest/src/ratatui/symbols.rs.html#150
    #[value(name = "braille", alias = "b")]
    Braille,
}

impl Style {
    pub fn next(&self) -> Self {
        match self {
            Style::Full => Style::Dark,
            Style::Dark => Style::Medium,
            Style::Medium => Style::Light,
            Style::Light => Style::Braille,
            Style::Braille => Style::Thick,
            Style::Thick => Style::Cross,
            Style::Cross => Style::Full,
        }
    }

    pub fn get_digit_symbol(&self) -> &str {
        match &self {
            Style::Full => shade::FULL,
            Style::Light => shade::LIGHT,
            Style::Medium => shade::MEDIUM,
            Style::Dark => shade::DARK,
            Style::Cross => "╬",
            Style::Thick => "┃",
            Style::Braille => "⣿",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, EnumString, Serialize, Deserialize)]
pub enum AppTimeFormat {
    /// `hh:mm:ss`
    #[default]
    HhMmSs,
    /// `hh:mm`
    HhMm,
    /// `hh:mm AM` (or PM)
    Hh12Mm,
}

impl AppTimeFormat {
    pub const fn first() -> Self {
        Self::HhMmSs
    }

    pub const fn last() -> Self {
        Self::Hh12Mm
    }

    pub fn next(&self) -> Self {
        match self {
            AppTimeFormat::HhMmSs => AppTimeFormat::HhMm,
            AppTimeFormat::HhMm => AppTimeFormat::Hh12Mm,
            AppTimeFormat::Hh12Mm => AppTimeFormat::HhMmSs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AppTime {
    Local(OffsetDateTime),
    Utc(OffsetDateTime),
}

impl From<AppTime> for OffsetDateTime {
    fn from(app_time: AppTime) -> Self {
        match app_time {
            AppTime::Local(t) => t,
            AppTime::Utc(t) => t,
        }
    }
}

impl AppTime {
    pub fn format(&self, app_format: &AppTimeFormat) -> String {
        let parse_str = match app_format {
            AppTimeFormat::HhMmSs => "[hour]:[minute]:[second]",
            AppTimeFormat::HhMm => "[hour]:[minute]",
            AppTimeFormat::Hh12Mm => "[hour repr:12 padding:none]:[minute] [period]",
        };

        format_description::parse(parse_str)
            .map_err(|_| "parse error")
            .and_then(|fd| {
                OffsetDateTime::from(*self)
                    .format(&fd)
                    .map_err(|_| "format error")
            })
            .unwrap_or_else(|e| e.to_string())
    }

    pub fn get_period(&self) -> String {
        format_description::parse("[period]")
            .map_err(|_| "parse error")
            .and_then(|fd| {
                OffsetDateTime::from(*self)
                    .format(&fd)
                    .map_err(|_| "format error")
            })
            .unwrap_or_else(|e| e.to_string())
    }

    /// Converts `AppTime` into a `Duration` representing elapsed time since midnight (today).
    pub fn as_duration_of_today(&self) -> std::time::Duration {
        let dt = OffsetDateTime::from(*self);
        let time = dt.time();

        let total_nanos = u64::from(time.hour()) * 3_600_000_000_000
            + u64::from(time.minute()) * 60_000_000_000
            + u64::from(time.second()) * 1_000_000_000
            + u64::from(time.nanosecond());

        std::time::Duration::from_nanos(total_nanos)
    }
}

#[derive(Debug)]
pub enum AppEditMode {
    None,
    Clock,
    Time,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Toggle {
    #[value(name = "on")]
    On,
    #[default]
    #[value(name = "off")]
    Off,
}

impl From<bool> for Toggle {
    fn from(value: bool) -> Self {
        match value {
            true => Toggle::On,
            false => Toggle::Off,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use time::{Date, Month, PrimitiveDateTime, Time};

    #[test]
    fn test_format_app_time() {
        let dt = PrimitiveDateTime::new(
            Date::from_calendar_date(2025, Month::January, 6).unwrap(),
            Time::from_hms(18, 6, 10).unwrap(),
        )
        .assume_utc();
        // hh:mm:ss
        assert_eq!(
            AppTime::Utc(dt).format(&AppTimeFormat::HhMmSs),
            "18:06:10",
            "utc"
        );
        assert_eq!(
            AppTime::Local(dt).format(&AppTimeFormat::HhMmSs),
            "18:06:10",
            "local"
        );
        // hh:mm
        assert_eq!(
            AppTime::Utc(dt).format(&AppTimeFormat::HhMm),
            "18:06",
            "utc"
        );
        assert_eq!(
            AppTime::Local(dt).format(&AppTimeFormat::HhMm),
            "18:06",
            "local"
        );
        // hh:mm period
        assert_eq!(
            AppTime::Utc(dt).format(&AppTimeFormat::Hh12Mm),
            "6:06 PM",
            "utc"
        );
        assert_eq!(
            AppTime::Local(dt).format(&AppTimeFormat::Hh12Mm),
            "6:06 PM",
            "local"
        );
    }
}
