use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::{Stream, StreamExt};
use std::{pin::Pin, time::Duration};
use tokio::time::interval;
use tokio_stream::{wrappers::IntervalStream, StreamMap};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum StreamKey {
    Ticks,
    Render,
    Crossterm,
}
#[derive(Clone, Debug)]
pub enum Event {
    Init,
    Quit,
    Error,
    Tick,
    Render,
    Key(KeyEvent),
    Resize(u16, u16),
}

pub struct Events {
    streams: StreamMap<StreamKey, Pin<Box<dyn Stream<Item = Event>>>>,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            streams: StreamMap::from_iter([
                (StreamKey::Ticks, tick_stream()),
                (StreamKey::Render, render_stream()),
                (StreamKey::Crossterm, crossterm_stream()),
            ]),
        }
    }
}

impl Events {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.streams.next().await.map(|(_, event)| event)
    }
}

fn tick_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    let tick_interval = interval(Duration::from_secs_f64(1.0 / 10.0));
    Box::pin(IntervalStream::new(tick_interval).map(|_| Event::Tick))
}

fn render_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    let render_interval = interval(Duration::from_secs_f64(1.0 / 60.0)); // 60 FPS
    Box::pin(IntervalStream::new(render_interval).map(|_| Event::Render))
}

fn crossterm_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    Box::pin(
        EventStream::new()
            .fuse()
            // we are not interested in all events
            .filter_map(|event| async move {
                match event {
                    Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                        Some(Event::Key(key))
                    }
                    Ok(CrosstermEvent::Resize(x, y)) => Some(Event::Resize(x, y)),
                    Err(_) => Some(Event::Error),
                    _ => None,
                }
            }),
    )
}
