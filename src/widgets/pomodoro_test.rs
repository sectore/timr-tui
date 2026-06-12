use crate::{
    common::Style,
    duration::{ONE_MINUTE, ONE_SECOND},
    events::{TuiEvent, TuiEventHandler},
    widgets::{
        pomodoro::{Mode, PauseDuration, PomodoroState, PomodoroStateArgs, PomodoroWidget},
        test_utils::{DrawArgs, Key, draw},
    },
};
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};
use std::time::Duration;

const WORK: Duration = ONE_MINUTE.saturating_mul(25); /* 25min */
const PAUSE: Duration = ONE_MINUTE.saturating_mul(5); /* 5min */

fn app_tx() -> crate::events::AppEventTx {
    tokio::sync::mpsc::unbounded_channel().0
}

fn w() -> PomodoroWidget {
    PomodoroWidget {
        style: Style::default(),
        blink: false,
    }
}

fn args() -> PomodoroStateArgs {
    PomodoroStateArgs {
        mode: Mode::Work,
        initial_value_work: WORK,
        current_value_work: WORK,
        pause_duration: PauseDuration::Fixed(PAUSE),
        current_value_pause: PAUSE,
        with_decis: false,
        app_tx: app_tx(),
        round: 1,
        vim_motions: false,
        auto_switch: false,
    }
}

fn st_with_args(args: PomodoroStateArgs) -> PomodoroState {
    PomodoroState::new(args)
}

fn st() -> PomodoroState {
    st_with_args(args())
}

fn terminal(w: PomodoroWidget, st: PomodoroState) -> Terminal<TestBackend> {
    draw(DrawArgs {
        widget: w,
        state: st,
        width: 70,
        height: 16,
    })
}

// work

#[test]
fn test_work_pause() {
    let t = terminal(w(), st());
    assert_snapshot!("work_pause", t.backend());
}

#[test]
fn test_work_pause_decis() {
    let st = st_with_args(PomodoroStateArgs {
        with_decis: true,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("work_pause_decis", t.backend());
}

#[test]
fn test_work_play() {
    let mut st = st_with_args(PomodoroStateArgs {
        current_value_work: WORK - ONE_MINUTE,
        ..args()
    });
    st.update(Key::StartStop.into());
    let t = terminal(w(), st);
    assert_snapshot!("work_play", t.backend());
}

#[test]
fn test_work_done() {
    let mut st = st_with_args(PomodoroStateArgs {
        current_value_work: Duration::ZERO,
        ..args()
    });
    st.update(Key::StartStop.into());
    st.update(TuiEvent::Tick);
    let t = terminal(w(), st);
    assert_snapshot!("work_done", t.backend());
}

#[test]
fn test_work_edit_minutes() {
    let mut st = st();
    st.update(Key::Edit.into());
    let t = terminal(w(), st);
    assert_snapshot!("work_edit_minutes", t.backend());
}

#[test]
fn test_work_edit_seconds() {
    let mut st = st_with_args(PomodoroStateArgs {
        current_value_work: ONE_SECOND.saturating_mul(12),
        ..args()
    });
    st.update(Key::Edit.into());
    let t = terminal(w(), st);
    assert_snapshot!("work_edit_seconds", t.backend());
}
