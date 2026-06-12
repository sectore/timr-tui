use crate::{
    common::{AppTime, AppTimeFormat, Style},
    duration::{ONE_MINUTE, ONE_SECOND},
    events::TuiEventHandler,
    widgets::{
        countdown::{Countdown, CountdownState, CountdownStateArgs},
        test_utils::{DrawArgs, FIXED_TIME, Key, draw},
    },
};
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};
use std::time::Duration;

const INITIAL: Duration = ONE_MINUTE.saturating_mul(30);

fn app_tx() -> crate::events::AppEventTx {
    tokio::sync::mpsc::unbounded_channel().0
}

fn w() -> Countdown {
    Countdown {
        style: Style::default(),
        blink: false,
    }
}

fn args() -> CountdownStateArgs {
    CountdownStateArgs {
        initial_value: INITIAL,
        current_value: INITIAL,
        elapsed_value: Duration::ZERO,
        app_time: AppTime::Utc(FIXED_TIME),
        target_time_format: None,
        with_decis: false,
        app_tx: app_tx(),
        vim_motions: false,
    }
}

fn st_with_args(args: CountdownStateArgs) -> CountdownState {
    CountdownState::new(args)
}

fn st() -> CountdownState {
    st_with_args(args())
}

fn terminal(w: Countdown, st: CountdownState) -> Terminal<TestBackend> {
    draw(DrawArgs {
        widget: w,
        state: st,
        width: 70,
        height: 16,
    })
}

#[test]
fn test_countdown_initial() {
    let t = terminal(w(), st());
    assert_snapshot!("countdown_initial", t.backend());
}

#[test]
fn test_countdown_run() {
    let mut st = st_with_args(CountdownStateArgs {
        current_value: INITIAL - ONE_MINUTE,
        ..args()
    });
    st.update(Key::StartStop.into());
    let t = terminal(w(), st);
    assert_snapshot!("countdown_run", t.backend());
}

#[test]
fn test_countdown_pause() {
    let mut st = st_with_args(CountdownStateArgs {
        current_value: INITIAL - ONE_MINUTE.saturating_mul(5),
        ..args()
    });
    st.update(Key::StartStop.into());
    st.update(Key::StartStop.into());
    let t = terminal(w(), st);
    assert_snapshot!("countdown_pause", t.backend());
}

#[test]
fn test_countdown_done() {
    let st = st_with_args(CountdownStateArgs {
        current_value: Duration::ZERO,
        elapsed_value: ONE_SECOND.saturating_mul(2),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("countdown_done", t.backend());
}

#[test]
fn test_countdown_edit_minutes() {
    let mut st = st();
    st.update(Key::Edit.into());
    let t = terminal(w(), st);
    assert_snapshot!("countdown_edit_minutes", t.backend());
}

#[test]
fn test_countdown_edit_seconds() {
    let mut st = st_with_args(CountdownStateArgs {
        current_value: ONE_SECOND.saturating_mul(12),
        ..args()
    });
    st.update(Key::Edit.into());
    let t = terminal(w(), st);
    assert_snapshot!("countdown_edit_seconds", t.backend());
}

#[test]
fn test_countdown_until_hhmmss() {
    let st = st_with_args(CountdownStateArgs {
        target_time_format: Some(AppTimeFormat::HhMmSs),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("countdown_until_hhmmss", t.backend());
}

#[test]
fn test_countdown_until_hhmm() {
    let st = st_with_args(CountdownStateArgs {
        target_time_format: Some(AppTimeFormat::HhMm),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("countdown_until_hhmm", t.backend());
}

#[test]
fn test_countdown_until_hh12mm() {
    let st = st_with_args(CountdownStateArgs {
        target_time_format: Some(AppTimeFormat::Hh12Mm),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("countdown_until_hh12mm", t.backend());
}
