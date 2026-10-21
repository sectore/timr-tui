use crate::{
    common::{Content, Style, Toggle},
    duration,
    event::{Event, parse_event},
    widgets::pomodoro::PauseDuration,
};
#[cfg(feature = "sound")]
use crate::{sound, sound::SoundError};
use clap::Parser;
#[cfg(feature = "sound")]
use clap::ValueEnum;
use std::path::PathBuf;
use std::time::Duration;

pub const LOG_DIRECTORY_DEFAULT_MISSING_VALUE: &str = " "; // empty string

#[derive(Parser)]
#[command(version)]
pub struct Args {
    #[arg(long, short, value_parser = duration::parse_long_duration,
        help = "Countdown time to start from. Formats: 'Yy Dd hh:mm:ss', 'Dd hh:mm:ss', 'Yy mm:ss', 'Dd mm:ss', 'Yy ss', 'Dd ss', 'hh:mm:ss', 'mm:ss', 'ss'. Examples: '1y 5d 10:30:00', '2d 4:00', '1d 10', '5:03'."
    )]
    pub countdown: Option<Duration>,

    #[arg(long, short, value_parser = duration::parse_duration,
        help = "Work time to count down from. Formats: 'ss', 'mm:ss', 'hh:mm:ss'"
    )]
    pub work: Option<Duration>,

    #[arg(long, short, value_parser = pause_duration_parser,
        help = "Pause duration. Single value (every round): '5:00'. Variable: 'regular,special[,every_n_rounds]' - special pause every N rounds, default every 4. Examples: '5:00,25:00' or '5:00,30:00,5'. Duration formats: 'ss', 'mm:ss', 'hh:mm:ss'."
    )]
    pub pause: Option<PauseDuration>,

    #[arg(long, help = "Maximum number of pomodoro rounds. 0 = unlimited.")]
    pub max_rounds: Option<u64>,

    #[arg(
        long,
        help = "Tabata regimen: work 20s, pause 10s, 8 rounds, auto-switch enabled."
    )]
    pub tabata: bool,

    #[arg(long, help = "Enable auto-switch between `work` and `pause` screens.")]
    pub auto_switch: bool,

    #[arg(
        long,
        short = 'e',
        value_parser = parse_event,
        help = "Event date time and title (optional). Format: 'YYYY-MM-DD HH:MM:SS' or 'time=YYYY-MM-DD HH:MM:SS[,title=...]'. Examples: '2025-10-10 14:30:00' or 'time=2025-10-10 14:30:00,title=My Event'."
    )]
    pub event: Option<Event>,

    #[arg(long, short = 'd', help = "Show deciseconds.")]
    pub decis: bool,

    #[arg(long, short = 'm', value_enum, help = "Mode to start with.")]
    pub mode: Option<Content>,

    #[arg(long, short = 's', value_enum, help = "Style to display time with.")]
    pub style: Option<Style>,

    #[arg(long, value_enum, help = "Open menu.")]
    pub menu: bool,

    #[arg(long, short = 'v', value_enum, help = "Enable/disable Vim motions.")]
    pub vim: Option<Toggle>,

    #[arg(long, short = 'r', help = "Reset stored values to defaults.")]
    pub reset: bool,

    #[arg(
        long,
        short,
        value_enum,
        help = "Enable/disable desktop notifications. Experimental."
    )]
    pub notification: Option<Toggle>,

    #[arg(
        long,
        value_enum,
        help = "Enable/disable blink mode to animate a clock when it reaches its finished mode."
    )]
    pub blink: Option<Toggle>,

    #[cfg(feature = "sound")]
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "on",
        help = "Sound notification: 'on', 'off' or path to a custom sound file (.mp3 or .wav).",
        value_hint = clap::ValueHint::FilePath,
        value_parser = sound_arg_parser,
    )]
    pub sound: Option<SoundArg>,

    #[arg(
        long,
        // allows both --log=path and --log path syntax
        num_args = 0..=1,
        // Note: If no value is passed, use a " " by default,
        // this value will be checked later in `main`
        // to use another (default) log directory instead
        default_missing_value=LOG_DIRECTORY_DEFAULT_MISSING_VALUE,
        help = "Directory for log file. If not set, standard application log directory is used (check README for details).",
        value_hint = clap::ValueHint::DirPath,
    )]
    pub log: Option<PathBuf>,
}

fn pause_duration_parser(s: &str) -> Result<PauseDuration, String> {
    let parse = |s| duration::parse_duration(s).map_err(|e| e.to_string());
    let parts: Vec<&str> = s.splitn(3, ',').collect();
    match parts.as_slice() {
        [single] => Ok(PauseDuration::Fixed(parse(single)?)),
        [regular, special] => Ok(PauseDuration::Variable {
            regular: parse(regular)?,
            special: parse(special)?,
            special_every: 4,
        }),
        [regular, special, every] => Ok(PauseDuration::Variable {
            regular: parse(regular)?,
            special: parse(special)?,
            special_every: every.parse::<u64>().map_err(|e| e.to_string())?,
        }),
        _ => Err("expected 'duration' or 'regular,special[,every_n_rounds]'".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use crate::duration::ONE_MINUTE;

    use super::*;
    use std::time::Duration;

    const FIVE_MIN: Duration = ONE_MINUTE.saturating_mul(5);
    const TEN_MIN: Duration = ONE_MINUTE.saturating_mul(10);

    #[test]
    fn pause_parser_fixed() {
        assert_eq!(
            pause_duration_parser("5:00").unwrap(),
            PauseDuration::Fixed(FIVE_MIN)
        );
    }

    #[test]
    fn pause_parser_variable() {
        assert_eq!(
            pause_duration_parser("5:00,10:00,4").unwrap(),
            PauseDuration::Variable {
                regular: FIVE_MIN,
                special: TEN_MIN,
                special_every: 4,
            }
        );
    }

    #[test]
    fn pause_parser_variable_default_every() {
        assert_eq!(
            pause_duration_parser("5:00,10:00").unwrap(),
            PauseDuration::Variable {
                regular: FIVE_MIN,
                special: TEN_MIN,
                special_every: 4,
            }
        );
    }

    #[test]
    fn pause_parser_invalid() {
        assert!(pause_duration_parser("invalid-duration").is_err());
    }
}

#[cfg(feature = "sound")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundArg {
    Toggle(Toggle),
    File(PathBuf),
}

#[cfg(feature = "sound")]
impl SoundArg {
    /// A custom sound file implies `on`
    pub fn toggle(&self) -> Toggle {
        match self {
            Self::Toggle(toggle) => *toggle,
            Self::File(_) => Toggle::On,
        }
    }
}

#[cfg(feature = "sound")]
fn sound_arg_parser(s: &str) -> Result<SoundArg, SoundError> {
    if let Ok(toggle) = Toggle::from_str(s, false) {
        return Ok(SoundArg::Toggle(toggle));
    }
    let path = PathBuf::from(s);
    sound::validate_sound_file(&path)?;
    Ok(SoundArg::File(path))
}

#[cfg(all(test, feature = "sound"))]
mod sound_tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Option<SoundArg>, clap::Error> {
        Args::try_parse_from([&["timr-tui"], args].concat()).map(|a| a.sound)
    }

    #[test]
    fn sound_not_set() {
        assert_eq!(parse(&[]).unwrap(), None);
    }

    #[test]
    fn sound_bare() {
        assert_eq!(
            parse(&["--sound"]).unwrap(),
            Some(SoundArg::Toggle(Toggle::On))
        );
    }

    #[test]
    fn sound_on_off() {
        assert_eq!(
            parse(&["--sound", "on"]).unwrap(),
            Some(SoundArg::Toggle(Toggle::On))
        );
        assert_eq!(
            parse(&["--sound=off"]).unwrap(),
            Some(SoundArg::Toggle(Toggle::Off))
        );
    }

    #[test]
    fn sound_file() {
        let path = "assets/confirmation-004.mp3";
        assert_eq!(
            parse(&["--sound", path]).unwrap(),
            Some(SoundArg::File(PathBuf::from(path)))
        );
    }

    #[test]
    fn sound_toggle() {
        assert_eq!(SoundArg::Toggle(Toggle::Off).toggle(), Toggle::Off);
        assert_eq!(SoundArg::File(PathBuf::new()).toggle(), Toggle::On);
    }

    #[test]
    fn sound_invalid() {
        assert!(parse(&["--sound", ""]).is_err());
        assert!(parse(&["--sound", "missing.mp3"]).is_err());
        assert!(parse(&["--sound", "Cargo.toml"]).is_err());
    }
}
