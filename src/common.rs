use clap::ValueEnum;
use ratatui::symbols::shade;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default)]
pub enum LocalTimeFormat {
    /// `hh:mm:ss`
    #[default]
    HhMmSs,
    /// `hh:mm`
    HhMm,
    /// `hh:mm AM` (or PM)
    Hh12Mm,
    /// empty
    Empty,
}

impl LocalTimeFormat {
    pub fn next(&self) -> Self {
        match self {
            LocalTimeFormat::HhMmSs => LocalTimeFormat::HhMm,
            LocalTimeFormat::HhMm => LocalTimeFormat::Hh12Mm,
            LocalTimeFormat::Hh12Mm => LocalTimeFormat::Empty,
            LocalTimeFormat::Empty => LocalTimeFormat::HhMmSs,
        }
    }

    pub fn fmt(&self) -> &str {
        match &self {
            LocalTimeFormat::HhMmSs => "%H:%M:%S",
            LocalTimeFormat::HhMm => "%H:%M",
            LocalTimeFormat::Hh12Mm => "%-I:%M %p",
            LocalTimeFormat::Empty => "",
        }
    }
}
