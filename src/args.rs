use crate::{
    common::{Content, Style},
    duration,
};
use clap::Parser;
use std::time::Duration;

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

    #[arg(long, short = 'd', help = "Whether to show deciseconds or not.")]
    pub decis: bool,

    #[arg(long, short = 'm', value_enum, help = "Mode to start with.")]
    pub mode: Option<Content>,

    #[arg(long, short = 's', value_enum, help = "Style to display time with.")]
    pub style: Option<Style>,

    #[arg(long, value_enum, help = "Whether to open the menu or not.")]
    pub menu: bool,

    #[arg(long, short = 'r', help = "Reset stored values to default.")]
    pub reset: bool,
}
