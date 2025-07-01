use tokio::sync::mpsc::UnboundedSender;

use crate::{listener::Event, worker::Action};

#[derive(Debug)]
pub(crate) struct StartingView {
    sender: UnboundedSender<Action>,
}

impl StartingView {
    pub(crate) fn new(sender: UnboundedSender<Action>) -> Self {
        Self { sender }
    }

    pub(crate) fn digest(&mut self, event: &crate::listener::Event) {
        match event {
            Event::Key(key) if key.code.as_char() == Some('Q') => {
                let _ = self.sender.send(Action::Shutdown);
            }
            _ => {}
        }
    }
}
