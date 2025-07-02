use myhomelab_dashboard::entity::Dashboard;
use tokio_stream::StreamExt;

pub struct Listener {
    timer: tokio::time::Interval,
    term_events: crossterm::event::EventStream,
    app_events: tokio::sync::mpsc::UnboundedReceiver<crate::listener::Event>,
}

impl Listener {
    pub fn new(
        interval: std::time::Duration,
        app_events: tokio::sync::mpsc::UnboundedReceiver<crate::listener::Event>,
    ) -> Self {
        Self {
            timer: tokio::time::interval(interval),
            term_events: crossterm::event::EventStream::new(),
            app_events,
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        loop {
            tokio::select! {
                maybe_event = self.term_events.next() => {
                    match maybe_event {
                        Some(Ok(crossterm::event::Event::Key(inner))) => {
                            self.timer.reset();
                            return Some(Event::Key(inner));
                        }
                        Some(Err(error)) => {
                            self.timer.reset();
                            return Some(Event::Error(error.into()));
                        }
                        _ => {}
                    }
                }
                app_event = self.app_events.recv() => {
                    return app_event;
                }
                _ = self.timer.tick() => {
                    return Some(Event::Tick)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Error(anyhow::Error),
    Key(crossterm::event::KeyEvent),
    Shutdown,
    Tick,
    DashboardList(AsyncEvent<Vec<Dashboard>>),
}

#[derive(Debug)]
pub enum AsyncEvent<T> {
    Init,
    Success(T),
    Error(anyhow::Error),
}
