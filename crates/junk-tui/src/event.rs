use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::{
    Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind, MouseEvent,
};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::app::AppResult;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

/// Terminal event-handler.
///
/// See [ratatui
/// example](https://github.com/ratatui/templates/blob/main/simple-async/src/event.rs);
#[derive(Debug)]
pub struct EventHandler {
    sx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sx, rx) = mpsc::unbounded_channel();
        let _sx = sx.clone();

        let handler = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);

            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    // break when sender closes
                    _ = _sx.closed() => {
                        break;
                    }

                    // continue to tick
                    _ = tick_delay => {
                        _sx.send(Event::Tick).expect("Event Sender failed to tick");
                    }

                    // handle user input
                    Some(Ok(event)) = crossterm_event => {
                        match event {
                            CrosstermEvent::Key(key) => {
                                if key.kind == KeyEventKind::Press {
                                    _sx.send(Event::Key(key)).expect("failed to send key input");
                                }
                            },
                            CrosstermEvent::Mouse(mouse) => {
                                _sx.send(Event::Mouse(mouse)).expect("failed to send mouse input");
                            },
                            CrosstermEvent::Resize(x, y) => {
                                _sx.send(Event::Resize(x, y)).expect("failed to send resize input");
                            },
                            CrosstermEvent::FocusLost => {},
                            CrosstermEvent::FocusGained => {},
                            CrosstermEvent::Paste(_) => {},
                        }
                    }
                }
            }
        });

        Self { sx, rx, handler }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> AppResult<Event> {
        self.rx.recv().await.ok_or(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "IO Error",
        )))
    }
}
