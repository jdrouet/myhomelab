use dashboard::DashboardView;
use myhomelab_dashboard::entity::Dashboard;
use ratatui::Terminal;
use ratatui::prelude::Backend;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Tabs};
use tokio::sync::mpsc::UnboundedSender;

use crate::prelude::Component;
use crate::worker::Action;

mod dashboard;
mod starting;

#[derive(Debug)]
pub(crate) enum Route {
    Starting(starting::StartingView),
    Dashboard(dashboard::DashboardView),
}

impl Route {
    pub fn digest(&mut self, event: crate::listener::Event) {
        match self {
            Self::Starting(inner) => inner.digest(event),
            Self::Dashboard(inner) => inner.digest(event),
        }
    }
}

impl ratatui::widgets::Widget for &Route {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self {
            Route::Starting(inner) => inner.render(area, buf),
            Route::Dashboard(inner) => inner.render(area, buf),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Router {
    sender: UnboundedSender<Action>,
    route: Route,
    dashboards: Vec<Dashboard>,
}

impl Router {
    pub(crate) fn new(sender: UnboundedSender<Action>) -> Self {
        let _ = sender.send(Action::FetchDashboardList);
        Self {
            sender: sender.clone(),
            route: Route::Starting(starting::StartingView::new(sender)),
            dashboards: Vec::new(),
        }
    }
}

impl crate::prelude::Component for Router {
    fn digest(&mut self, event: crate::listener::Event) {
        match event {
            crate::listener::Event::DashboardList(crate::listener::AsyncEvent::Success(list)) => {
                self.dashboards = list;
                self.route = Route::Dashboard(DashboardView::new(self.sender.clone()));
            }
            other => self.route.digest(other),
        }
    }
}

impl Router {
    pub(crate) fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
        Ok(())
    }
}

impl ratatui::widgets::Widget for &Router {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" MyHomeLab ".bold()).right_aligned();
        let instructions = Line::from_iter([" Quit ".into(), "<Q> ".blue().bold()]);
        let block = Block::bordered().title(title).title_bottom(instructions);
        let inner = block.inner(area);
        let tabs = Tabs::new(self.dashboards.iter().map(|dash| dash.title.as_str()))
            .block(block)
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(0)
            .divider(" ")
            .padding("", "");
        tabs.render(area, buf);
        self.route.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    use super::starting::StartingView;

    #[test]
    fn should_render_with_starting() {
        let (action_tx, _action_rx) = tokio::sync::mpsc::unbounded_channel();
        let view = super::Router {
            sender: action_tx.clone(),
            route: super::Route::Starting(StartingView::new(action_tx)),
            dashboards: Vec::default(),
        };

        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 5));
        view.render(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "┌───────────────────────────────────── MyHomeLab ┐",
            "│Loading...                                      │",
            "│                                                │",
            "│                                                │",
            "└ Quit <Q> ──────────────────────────────────────┘",
        ]);

        similar_asserts::assert_eq!(buf.area, expected.area);
    }
}
