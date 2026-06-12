use crate::{
    common::Style,
    duration::{MAX_DURATION, ONE_DAY, ONE_HOUR, ONE_MINUTE, ONE_SECOND, ONE_YEAR},
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

fn st(current_value: Duration) -> ClockState<Timer> {
    ClockState::<Timer>::new(ClockStateArgs {
        initial_value: current_value,
        current_value,
        tick_value: Duration::from_millis(100),
        with_decis: false,
        app_tx: None,
    })
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
    let t = terminal(w(), st(MAX_DURATION));
    assert_snapshot!("clock_yyyydddhhmmss", t.backend());
}

#[test]
fn test_clock_yyyyddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR.saturating_mul(1000)
            + ONE_DAY.saturating_mul(10)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yyyyddhhmmss", t.backend());
}

#[test]
fn test_clock_yyyydhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR.saturating_mul(1000)
            + ONE_DAY
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yyyydhhmmss", t.backend());
}

#[test]
fn test_clock_yyydddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR.saturating_mul(100)
            + ONE_DAY.saturating_mul(100)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yyydddhhmmss", t.backend());
}

#[test]
fn test_clock_yyyddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR.saturating_mul(100)
            + ONE_DAY.saturating_mul(10)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yyyddhhmmss", t.backend());
}

#[test]
fn test_clock_yyydhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR.saturating_mul(100)
            + ONE_DAY
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yyydhhmmss", t.backend());
}

#[test]
fn test_clock_yydddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR.saturating_mul(10)
            + ONE_DAY.saturating_mul(100)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yydddhhmmss", t.backend());
}

#[test]
fn test_clock_yyddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR.saturating_mul(10)
            + ONE_DAY.saturating_mul(10)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yyddhhmmss", t.backend());
}

#[test]
fn test_clock_yydhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR.saturating_mul(10)
            + ONE_DAY
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yydhhmmss", t.backend());
}

#[test]
fn test_clock_ydddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR
            + ONE_DAY.saturating_mul(100)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_ydddhhmmss", t.backend());
}

#[test]
fn test_clock_yddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR
            + ONE_DAY.saturating_mul(10)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_yddhhmmss", t.backend());
}

#[test]
fn test_clock_ydhhmmss() {
    let t = terminal(
        w(),
        st(ONE_YEAR
            + ONE_DAY
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_ydhhmmss", t.backend());
}

#[test]
fn test_clock_dddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_DAY.saturating_mul(100)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_dddhhmmss", t.backend());
}

#[test]
fn test_clock_ddhhmmss() {
    let t = terminal(
        w(),
        st(ONE_DAY.saturating_mul(10)
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_ddhhmmss", t.backend());
}

#[test]
fn test_clock_dhhmmss() {
    let t = terminal(
        w(),
        st(ONE_DAY
            + ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_dhhmmss", t.backend());
}

#[test]
fn test_clock_hhmmss() {
    let t = terminal(
        w(),
        st(ONE_HOUR.saturating_mul(14)
            + ONE_MINUTE.saturating_mul(30)
            + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_hhmmss", t.backend());
}

#[test]
fn test_clock_hmmss() {
    let t = terminal(
        w(),
        st(ONE_HOUR + ONE_MINUTE.saturating_mul(30) + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_hmmss", t.backend());
}

#[test]
fn test_clock_mmss() {
    let t = terminal(
        w(),
        st(ONE_MINUTE.saturating_mul(30) + ONE_SECOND.saturating_mul(5)),
    );
    assert_snapshot!("clock_mmss", t.backend());
}

#[test]
fn test_clock_mss() {
    let t = terminal(
        w(),
        st(ONE_MINUTE.saturating_mul(5) + ONE_SECOND.saturating_mul(42)),
    );
    assert_snapshot!("clock_mss", t.backend());
}

#[test]
fn test_clock_ss() {
    let t = terminal(w(), st(ONE_SECOND.saturating_mul(42)));
    assert_snapshot!("clock_ss", t.backend());
}

#[test]
fn test_clock_s() {
    let t = terminal(w(), st(ONE_SECOND.saturating_mul(7)));
    assert_snapshot!("clock_s", t.backend());
}

#[test]
fn test_clock_decis() {
    let t = terminal(
        w(),
        ClockState::<Timer>::new(ClockStateArgs {
            initial_value: ONE_SECOND.saturating_mul(7),
            current_value: ONE_SECOND.saturating_mul(7),
            tick_value: Duration::from_millis(100),
            with_decis: true,
            app_tx: None,
        }),
    );
    assert_snapshot!("clock_decis", t.backend());
}

#[test]
fn test_clock_style() {
    let t = terminal(
        ClockWidget::new(Style::Braille, false),
        ClockState::<Timer>::new(ClockStateArgs {
            initial_value: MAX_DURATION,
            current_value: MAX_DURATION,
            tick_value: Duration::from_millis(100),
            with_decis: true,
            app_tx: None,
        }),
    );
    assert_snapshot!("clock_style", t.backend());
}
