use ratatui::{Terminal, backend::TestBackend, widgets::StatefulWidget};
use time::{OffsetDateTime, macros::datetime};

pub const FIXED_TIME: OffsetDateTime = datetime!(2024-06-10 14:30:00 UTC);

pub struct AssertSnapshotArgs<W>
where
    W: StatefulWidget,
    W::State: Sized,
{
    pub widget: W,
    pub state: W::State,
    pub width: u16,
    pub height: u16,
}

/// Renders a stateful widget into a `TestBackend` and asserts the output matches the stored snapshot.
pub fn assert_snapshot<W>(args: AssertSnapshotArgs<W>)
where
    W: StatefulWidget,
    W::State: Sized,
{
    let AssertSnapshotArgs {
        widget,
        mut state,
        width,
        height,
    } = args;
    let mut t = Terminal::new(TestBackend::new(width, height)).unwrap();
    t.draw(|frame| frame.render_stateful_widget(widget, frame.area(), &mut state))
        .unwrap();
    // derive snapshot name from the test thread name to preserve the original module path naming
    let snapshot_name = std::thread::current()
        .name()
        .unwrap_or("unknown")
        .replace("::", "__");
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(snapshot_name, t.backend());
    });
}
