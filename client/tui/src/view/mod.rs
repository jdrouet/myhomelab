use dashboard::DashboardView;
use myhomelab_adapter_http_client::AdapterHttpClient;
use ratatui::Terminal;
use ratatui::prelude::Backend;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use starting::StartingView;

use crate::hook::dashboard::{DashboardListHook, DashboardListState};

mod dashboard;
mod starting;

#[derive(Debug)]
pub(crate) struct Router {
    client: AdapterHttpClient,
    dashboard_list_hook: DashboardListHook,
    current: usize,
    loading: bool,
    dashboards: Vec<DashboardView>,
}

impl Router {
    pub(crate) fn new(client: AdapterHttpClient) -> Self {
        Self {
            client: client.clone(),
            dashboard_list_hook: DashboardListHook::new(client),
            current: 0,
            dashboards: Vec::new(),
            loading: true,
        }
    }
}

impl crate::prelude::Component for Router {
    fn digest(&mut self, ctx: &crate::prelude::Context, event: &crate::listener::Event) {
        if let Some(state) = self.dashboard_list_hook.pull() {
            match state {
                DashboardListState::Loading => {
                    self.loading = true;
                }
                DashboardListState::Success(list) => {
                    self.loading = false;
                    self.current = 0;
                    self.dashboards = list
                        .into_iter()
                        .map(|board| DashboardView::new(self.client.clone(), board))
                        .collect();
                }
                DashboardListState::Error(_) => {
                    self.loading = false;
                }
            }
        }
        match event {
            crate::listener::Event::Key(key) if key.code.as_char() == Some('Q') => {
                let _ = ctx.sender.send(crate::listener::Event::Shutdown);
            }
            crate::listener::Event::Key(key)
                if key.code.as_char() == Some('R')
                    && !self.loading
                    && self.dashboards.is_empty() =>
            {
                self.dashboard_list_hook.execute();
            }
            other => {
                if let Some(board) = self.dashboards.get_mut(self.current) {
                    board.digest(ctx, other);
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
        let title = if self.loading {
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
            StartingView::default().render(inner, buf);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use ratatui::buffer::Buffer;
//     use ratatui::layout::Rect;
//     use ratatui::widgets::Widget;

//     use crate::hook::dashboard::DashboardListHook;

//     #[test]
//     fn should_render_with_starting() {
//         let (action_tx, _action_rx) = tokio::sync::mpsc::unbounded_channel();
//         let view = super::Router {
//             dashboard_list_hook: DashboardListHook::new(client.clone()),
//             current: 0,
//             dashboards: Vec::default(),
//             loading_dashboard: false,
//         };

//         let mut buf = Buffer::empty(Rect::new(0, 0, 50, 5));
//         view.render(buf.area, &mut buf);

//         let expected = Buffer::with_lines(vec![
//             "┌───────────────────────────────────── MyHomeLab ┐",
//             "│Loading...                                      │",
//             "│                                                │",
//             "│                                                │",
//             "└ Quit <Q> ──────────────────────────────────────┘",
//         ]);

//         similar_asserts::assert_eq!(buf.area, expected.area);
//     }
// }
