use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend, widgets::StatefulWidget};
use time::{OffsetDateTime, macros::datetime};

use crate::events::TuiEvent;

pub const FIXED_TIME: OffsetDateTime = datetime!(2024-06-10 14:30:00 UTC);

pub enum Action {
    StartStop,
    Edit,
}

impl From<Action> for TuiEvent {
    fn from(action: Action) -> Self {
        let code = match action {
            Action::StartStop => KeyCode::Char(' '),
            Action::Edit => KeyCode::Char('e'),
        };
        TuiEvent::Crossterm(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)))
    }
}

pub struct DrawArgs<W>
where
    W: StatefulWidget,
    W::State: Sized,
{
    pub widget: W,
    pub state: W::State,
    pub width: u16,
    pub height: u16,
}

/// Draws a stateful widget into a `TestBackend`.
pub fn draw<W>(args: DrawArgs<W>) -> Terminal<TestBackend>
where
    W: StatefulWidget,
    W::State: Sized,
{
    let DrawArgs {
        widget,
        mut state,
        width,
        height,
    } = args;
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| frame.render_stateful_widget(widget, frame.area(), &mut state))
        .unwrap();
    terminal
}
