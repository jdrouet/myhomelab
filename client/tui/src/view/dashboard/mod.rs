use myhomelab_dashboard::entity::{Dashboard, DashboardCell, Size};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use tokio::sync::mpsc::UnboundedSender;

use crate::listener::Event;
use crate::worker::Action;

mod cell;
mod line;

#[derive(Debug)]
pub(crate) struct DashboardView {
    title: String,
    cells: Vec<DashboardCell>,
    sender: UnboundedSender<Action>,
}

impl DashboardView {
    pub(crate) fn new(dashboard: Dashboard, sender: UnboundedSender<Action>) -> Self {
        Self {
            title: dashboard.title,
            cells: dashboard.cells,
            sender,
        }
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }
}

impl crate::prelude::Component for DashboardView {
    fn digest(&mut self, event: crate::listener::Event) {
        match event {
            Event::Key(key) if key.code.as_char() == Some('Q') => {
                let _ = self.sender.send(Action::Shutdown);
            }
            Event::Key(key) if key.code.as_char() == Some('R') => {
                let _ = self.sender.send(Action::FetchDashboardList);
            }
            _ => {}
        }
    }
}

impl ratatui::widgets::Widget for &DashboardView {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let lines = line::DashboardLineIterator::new(&self.cells, Size::Large).collect::<Vec<_>>();
        let vertical = Layout::vertical(lines.iter().map(|_| Constraint::Min(10))).spacing(1);
        let areas = vertical.split(area);
        for (area, line) in areas.into_iter().zip(lines.into_iter()) {
            line.render(*area, buf);
        }
    }
}
