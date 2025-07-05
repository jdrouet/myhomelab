use cell::DashboardLineCell;
use myhomelab_adapter_http_client::AdapterHttpClient;
use myhomelab_dashboard::entity::{Dashboard, Size};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};

use crate::listener::Event;

mod cell;
mod line;

#[derive(Debug)]
pub(crate) struct DashboardView {
    title: String,
    cells: Vec<DashboardLineCell>,
}

impl DashboardView {
    pub(crate) fn new(client: AdapterHttpClient, dashboard: Dashboard) -> Self {
        Self {
            title: dashboard.title,
            cells: dashboard
                .cells
                .into_iter()
                .map(|cell| DashboardLineCell::new(client.clone(), cell))
                .collect(),
        }
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }
}

impl crate::prelude::Component for DashboardView {
    fn digest(&mut self, ctx: &crate::prelude::Context, event: &crate::listener::Event) {
        match event {
            Event::Key(key) if key.code.as_char() == Some('R') => {}
            other => {
                self.cells
                    .iter_mut()
                    .for_each(|cell| cell.digest(ctx, other));
            }
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
