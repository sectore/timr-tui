use crate::{
    common::Style,
    constants::TICK_VALUE_MS,
    duration::{ONE_MINUTE, ONE_SECOND},
    events::TuiEventHandler,
    widgets::{
        clock::{ClockState, ClockStateArgs, Timer as ClockTimer},
        test_utils::{DrawArgs, Key, draw},
        timer::{Timer, TimerState},
    },
};
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};
use std::time::Duration;

struct Args {
    current_value: Duration,
    with_decis: bool,
    vim_motions: bool,
}

fn args() -> Args {
    Args {
        current_value: Duration::ZERO,
        with_decis: false,
        vim_motions: false,
    }
}

fn st_with_args(a: Args) -> TimerState {
    TimerState::new(
        ClockState::<ClockTimer>::new(ClockStateArgs {
            initial_value: Duration::ZERO,
            current_value: a.current_value,
            tick_value: Duration::from_millis(TICK_VALUE_MS),
            with_decis: a.with_decis,
            app_tx: None,
        }),
        a.vim_motions,
    )
}

fn st() -> TimerState {
    st_with_args(args())
}

fn w() -> Timer {
    Timer {
        style: Style::default(),
        blink: false,
    }
}

fn terminal(w: Timer, st: TimerState) -> Terminal<TestBackend> {
    draw(DrawArgs {
        widget: w,
        state: st,
        width: 70,
        height: 16,
    })
}

#[test]
fn test_timer_initial() {
    let t = terminal(w(), st());
    assert_snapshot!("timer_initial", t.backend());
}

#[test]
fn test_timer_run() {
    let mut st = st_with_args(Args {
        current_value: ONE_MINUTE.saturating_mul(5),
        ..args()
    });
    st.update(Key::StartStop.into());
    let t = terminal(w(), st);
    assert_snapshot!("timer_run", t.backend());
}

#[test]
fn test_timer_pause() {
    let mut st = st_with_args(Args {
        current_value: ONE_MINUTE.saturating_mul(5),
        ..args()
    });
    st.update(Key::StartStop.into());
    st.update(Key::StartStop.into());
    let t = terminal(w(), st);
    assert_snapshot!("timer_pause", t.backend());
}

#[test]
fn test_timer_edit_minutes() {
    let mut st = st_with_args(Args {
        current_value: ONE_MINUTE.saturating_mul(5),
        ..args()
    });
    st.update(Key::Edit.into());
    let t = terminal(w(), st);
    assert_snapshot!("timer_edit_minutes", t.backend());
}

#[test]
fn test_timer_edit_seconds() {
    let mut st = st_with_args(Args {
        current_value: ONE_SECOND.saturating_mul(12),
        ..args()
    });
    st.update(Key::Edit.into());
    let t = terminal(w(), st);
    assert_snapshot!("timer_edit_seconds", t.backend());
}
