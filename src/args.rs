use crate::{
    common::{Content, Style, Toggle},
    duration,
};
#[cfg(feature = "sound")]
use crate::{sound, sound::SoundError};
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

pub const LOG_DIRECTORY_DEFAULT_MISSING_VALUE: &str = " "; // empty string

#[derive(Parser)]
#[command(version)]
pub struct Args {
    #[arg(long, short, value_parser = duration::parse_duration,
        help = "Countdown time to start from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss'"
    )]
    pub countdown: Option<Duration>,

    #[arg(long, short, value_parser = duration::parse_duration,
        help = "Work time to count down from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss'"
    )]
    pub work: Option<Duration>,

    #[arg(long, short, value_parser = duration::parse_duration,
        help = "Pause time to count down from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss'"
    )]
    pub pause: Option<Duration>,

    #[arg(long, short = 'd', help = "Show deciseconds.")]
    pub decis: bool,

    #[arg(long, short = 'm', value_enum, help = "Mode to start with.")]
    pub mode: Option<Content>,

    #[arg(long, short = 's', value_enum, help = "Style to display time with.")]
    pub style: Option<Style>,

    #[arg(long, value_enum, help = "Open the menu.")]
    pub menu: bool,

    #[arg(long, short = 'r', help = "Reset stored values to default values.")]
    pub reset: bool,

    #[arg(
        long,
        short,
        value_enum,
        help = "Toggle desktop notifications. Experimental."
    )]
    pub notification: Option<Toggle>,

    #[arg(
        long,
        value_enum,
        help = "Toggle blink mode to animate a clock when it reaches its finished mode."
    )]
    pub blink: Option<Toggle>,

    #[cfg(feature = "sound")]
    #[arg(
        long,
        value_enum,
        help = "Path to sound file (.mp3 or .wav) to play as notification. Experimental.",
        value_hint = clap::ValueHint::FilePath,
        value_parser = sound_file_parser,
    )]
    pub sound: Option<PathBuf>,

    #[arg(
        long,
        // allows both --log=path and --log path syntax
        num_args = 0..=1,
        // Note: If no value is passed, use a " " by default,
        // this value will be checked later in `main`
        // to use another (default) log directory instead
        default_missing_value=LOG_DIRECTORY_DEFAULT_MISSING_VALUE,
        help = "Directory to store log file. If not set, standard application log directory is used (check README for details).",
        value_hint = clap::ValueHint::DirPath,
    )]
    pub log: Option<PathBuf>,
}

#[cfg(feature = "sound")]
/// Custom parser for sound file
fn sound_file_parser(s: &str) -> Result<PathBuf, SoundError> {
    let path = PathBuf::from(s);
    sound::validate_sound_file(&path)?;
    Ok(path)
}
