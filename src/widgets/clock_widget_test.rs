use crate::{
    common::Style,
    duration::{MAX_DURATION, parse_duration, parse_long_duration},
    widgets::{
        clock::{ClockState, ClockStateArgs, ClockWidget, Timer},
        clock_elements::DIGIT_HEIGHT,
        test_utils::{DrawArgs, draw},
    },
};
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};
use std::time::Duration;

fn w() -> ClockWidget<Timer> {
    ClockWidget::new(Style::default(), false)
}

fn args() -> ClockStateArgs {
    ClockStateArgs {
        initial_value: Duration::from_hours(21),
        current_value: Duration::from_mins(21),
        tick_value: Duration::from_millis(100),
        with_decis: false,
        app_tx: None,
    }
}

fn st_with_args(args: ClockStateArgs) -> ClockState<Timer> {
    ClockState::<Timer>::new(args)
}

fn _st() -> ClockState<Timer> {
    st_with_args(args())
}

fn terminal(w: ClockWidget<Timer>, st: ClockState<Timer>) -> Terminal<TestBackend> {
    draw(DrawArgs {
        widget: w,
        state: st,
        width: 120,
        height: DIGIT_HEIGHT,
    })
}

// Formats — longest to shortest

#[test]
fn test_clock_yyyydddhhmmss() {
    let st = st_with_args(ClockStateArgs {
        current_value: MAX_DURATION,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yyyydddhhmmss", t.backend());
}

#[test]
fn test_clock_yyyyddhhmmss() {
    let current_value = parse_long_duration("1000y 10d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yyyyddhhmmss", t.backend());
}

#[test]
fn test_clock_yyyydhhmmss() {
    let current_value = parse_long_duration("1000y 1d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yyyydhhmmss", t.backend());
}

#[test]
fn test_clock_yyydddhhmmss() {
    let current_value = parse_long_duration("100y 100d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yyydddhhmmss", t.backend());
}

#[test]
fn test_clock_yyyddhhmmss() {
    let current_value = parse_long_duration("100y 10d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yyyddhhmmss", t.backend());
}

#[test]
fn test_clock_yyydhhmmss() {
    let current_value = parse_long_duration("100y 1d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yyydhhmmss", t.backend());
}

#[test]
fn test_clock_yydddhhmmss() {
    let current_value = parse_long_duration("10y 100d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yydddhhmmss", t.backend());
}

#[test]
fn test_clock_yyddhhmmss() {
    let current_value = parse_long_duration("10y 10d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yyddhhmmss", t.backend());
}

#[test]
fn test_clock_yydhhmmss() {
    let current_value = parse_long_duration("10y 1d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yydhhmmss", t.backend());
}

#[test]
fn test_clock_ydddhhmmss() {
    let current_value = parse_long_duration("1y 100d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_ydddhhmmss", t.backend());
}

#[test]
fn test_clock_yddhhmmss() {
    let current_value = parse_long_duration("1y 10d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_yddhhmmss", t.backend());
}

#[test]
fn test_clock_ydhhmmss() {
    let current_value = parse_long_duration("1y 1d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_ydhhmmss", t.backend());
}

#[test]
fn test_clock_dddhhmmss() {
    let current_value = parse_long_duration("100d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_dddhhmmss", t.backend());
}

#[test]
fn test_clock_ddhhmmss() {
    let current_value = parse_long_duration("10d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_ddhhmmss", t.backend());
}

#[test]
fn test_clock_dhhmmss() {
    let current_value = parse_long_duration("1d 14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_dhhmmss", t.backend());
}

#[test]
fn test_clock_hhmmss() {
    let current_value = parse_duration("14:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_hhmmss", t.backend());
}

#[test]
fn test_clock_hmmss() {
    let current_value = parse_duration("1:30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_hmmss", t.backend());
}

#[test]
fn test_clock_mmss() {
    let current_value = parse_duration("30:05").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_mmss", t.backend());
}

#[test]
fn test_clock_mss() {
    let current_value = parse_duration("5:42").unwrap();
    let st = st_with_args(ClockStateArgs {
        current_value,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_mss", t.backend());
}

#[test]
fn test_clock_ss() {
    let st = st_with_args(ClockStateArgs {
        current_value: Duration::from_secs(42),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_ss", t.backend());
}

#[test]
fn test_clock_s() {
    let st = st_with_args(ClockStateArgs {
        current_value: Duration::from_secs(7),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_s", t.backend());
}

#[test]
fn test_clock_decis() {
    let st = st_with_args(ClockStateArgs {
        current_value: Duration::from_secs(7),
        with_decis: true,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("clock_decis", t.backend());
}

#[test]
fn test_clock_style() {
    let st = st_with_args(ClockStateArgs {
        current_value: MAX_DURATION,
        with_decis: true,
        ..args()
    });
    let t = terminal(ClockWidget::new(Style::Braille, false), st);
    assert_snapshot!("clock_style", t.backend());
}
