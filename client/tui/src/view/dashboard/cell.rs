use myhomelab_adapter_http_client::AdapterHttpClient;
use myhomelab_dashboard::entity::DashboardCell;
use myhomelab_metric::query::Response;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::Block,
};

use crate::hook::query::{QueryHook, QueryState};

#[derive(Debug)]
pub(crate) struct DashboardLineCell {
    hook: QueryHook,
    inner: DashboardCell,
    loading: bool,
    responses: Vec<Response>,
    error: Option<anyhow::Error>,
}

impl DashboardLineCell {
    pub fn new(client: AdapterHttpClient, inner: DashboardCell) -> Self {
        let hook = QueryHook::new(client, inner.kind, inner.query.clone());
        Self {
            hook,
            loading: true,
            inner,
            responses: Vec::new(),
            error: None,
        }
    }

    pub fn inner(&self) -> &DashboardCell {
        &self.inner
    }
}

impl crate::prelude::Component for DashboardLineCell {
    fn digest(&mut self, _ctx: &crate::prelude::Context, _event: &crate::listener::Event) {
        match self.hook.pull() {
            Some(QueryState::Loading) => {
                self.loading = true;
            }
            Some(QueryState::Success(responses)) => {
                self.loading = false;
                self.responses = responses;
                self.error = None;
                self.hook.execute();
            }
            Some(QueryState::Error(error)) => {
                self.loading = false;
                self.error = Some(error);
            }
            None => {}
        }
    }
}

impl<'a> ratatui::widgets::Widget for &DashboardLineCell {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered();
        let block = match self.inner.title {
            Some(ref title) => {
                let title = Line::from_iter([format!(" {title} ").italic()]);
                block.title(title)
            }
            None => block,
        };
        let inner = block.inner(area);
        block.render(area, buf);

        if let Some(ref err) = self.error {
            Text::from(format!(" {err:?} ")).render(inner, buf);
        } else {
            Text::from(format!(" {:?} ", self.responses)).render(inner, buf);
        }
    }
}
