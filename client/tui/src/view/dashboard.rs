use ratatui::style::Stylize;
use ratatui::text::Text;
use tokio::sync::mpsc::UnboundedSender;

use crate::listener::Event;
use crate::worker::Action;

#[derive(Debug)]
pub(crate) struct DashboardView {
    sender: UnboundedSender<Action>,
}

impl DashboardView {
    pub(crate) fn new(sender: UnboundedSender<Action>) -> Self {
        Self { sender }
    }
}

impl crate::prelude::Component for DashboardView {
    fn digest(&mut self, event: crate::listener::Event) {
        match event {
            Event::Key(key) if key.code.as_char() == Some('Q') => {
                let _ = self.sender.send(Action::Shutdown);
            }
            _ => {}
        }
    }
}

impl ratatui::widgets::Widget for &DashboardView {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Text::from("DASHBOARD LIST".bold()).render(area, buf);
    }
}
