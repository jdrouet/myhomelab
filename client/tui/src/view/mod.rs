use dashboard::DashboardView;
use ratatui::Terminal;
use ratatui::prelude::Backend;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use starting::StartingView;
use tokio::sync::mpsc::UnboundedSender;

use crate::worker::Action;

mod dashboard;
mod starting;

#[derive(Debug)]
pub(crate) struct Router {
    current: usize,
    loading_dashboard: bool,
    dashboards: Vec<DashboardView>,
    sender: UnboundedSender<Action>,
}

impl Router {
    pub(crate) fn new(sender: UnboundedSender<Action>) -> Self {
        let _ = sender.send(Action::FetchDashboardList);
        Self {
            current: 0,
            dashboards: Vec::new(),
            loading_dashboard: true,
            sender: sender.clone(),
        }
    }
}

impl crate::prelude::Component for Router {
    fn digest(&mut self, event: crate::listener::Event) {
        match event {
            crate::listener::Event::Key(key) if key.code.as_char() == Some('Q') => {
                let _ = self.sender.send(Action::Shutdown);
            }
            crate::listener::Event::DashboardList(crate::listener::AsyncEvent::Init) => {
                self.loading_dashboard = true;
            }
            crate::listener::Event::DashboardList(crate::listener::AsyncEvent::Success(list)) => {
                self.current = 0;
                self.dashboards = list
                    .into_iter()
                    .map(|dashboard| DashboardView::new(dashboard, self.sender.clone()))
                    .collect();
                self.loading_dashboard = false;
            }
            crate::listener::Event::DashboardList(crate::listener::AsyncEvent::Error(_)) => {
                self.current = 0;
                self.dashboards = Default::default();
                self.loading_dashboard = false;
            }
            other => {
                if let Some(board) = self.dashboards.get_mut(self.current) {
                    board.digest(other);
                }
            }
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
        let title = if self.loading_dashboard {
            Line::from(" MyHomeLab (loading...) ".bold()).left_aligned()
        } else if let Some(dash) = self.dashboards.get(self.current) {
            let title = format!(
                " MyHomeLab - {} ({}/{}) ",
                dash.title(),
                (self.current + 1).min(self.dashboards.len()),
                self.dashboards.len()
            );
            Line::from(title.bold()).left_aligned()
        } else {
            Line::from(" MyHomeLab ".bold()).left_aligned()
        };
        let instructions = Line::from_iter([
            " Refresh ".into(),
            "<R> ".blue().bold(),
            "- Quit ".into(),
            "<Q> ".blue().bold(),
        ])
        .centered();
        let block = Block::bordered().title(title).title_bottom(instructions);
        let inner = block.inner(area);
        block.render(area, buf);

        if let Some(board) = self.dashboards.get(self.current) {
            board.render(inner, buf);
        } else {
            StartingView::new(self.sender.clone()).render(inner, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    #[test]
    fn should_render_with_starting() {
        let (action_tx, _action_rx) = tokio::sync::mpsc::unbounded_channel();
        let view = super::Router {
            current: 0,
            dashboards: Vec::default(),
            loading_dashboard: false,
            sender: action_tx.clone(),
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
