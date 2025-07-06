use std::collections::HashMap;

use myhomelab_adapter_http_client::AdapterHttpClient;
use myhomelab_dashboard::entity::DashboardCell;
use myhomelab_metric::query::{Response, TimeseriesResponse};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
};

use crate::hook::query::{QueryHook, QueryState};

#[derive(Debug)]
pub(crate) struct DashboardLineCell {
    hook: QueryHook,
    inner: DashboardCell,
    loading: bool,
    responses: HashMap<Box<str>, Response>,
    error: Option<anyhow::Error>,
}

impl DashboardLineCell {
    pub fn new(client: AdapterHttpClient, inner: DashboardCell) -> Self {
        let hook = QueryHook::new(client, inner.kind, inner.query.clone());
        Self {
            hook,
            loading: true,
            inner,
            responses: HashMap::default(),
            error: None,
        }
    }

    pub fn inner(&self) -> &DashboardCell {
        &self.inner
    }
}

#[derive(Debug, Default)]
struct Subset {
    points: Vec<(f64, f64)>,
}

impl Subset {
    pub fn with(mut self, ts: f64, value: f64) -> Self {
        self.points.push((ts, value));
        self
    }
}

fn render_timeseries(
    block: Block<'_>,
    timeseries: &[TimeseriesResponse],
    area: Rect,
    buf: &mut Buffer,
) {
    use ratatui::widgets::Widget;

    let min_x = timeseries
        .iter()
        .flat_map(|item| item.values.iter().map(|(ts, _)| *ts))
        .min()
        .unwrap_or(0) as f64;
    let max_x = timeseries
        .iter()
        .flat_map(|item| item.values.iter().map(|(ts, _)| *ts))
        .max()
        .unwrap_or(0) as f64;

    let data = timeseries
        .iter()
        .map(|item| {
            item.values
                .iter()
                .fold(Subset::default(), |sub, (ts, value)| {
                    sub.with(*ts as f64, *value)
                })
        })
        .collect::<Vec<_>>();
    let dataset = data
        .iter()
        .enumerate()
        .map(|(index, subset)| {
            Dataset::default()
                .name(format!("Set {index}"))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&subset.points)
        })
        .collect();
    let chart = Chart::new(dataset)
        .block(block)
        .x_axis(
            Axis::default()
                .title("Time")
                .bounds([min_x, max_x])
                .style(Style::default().gray()),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 30.0])
                .style(Style::default().gray()),
        );
    chart.render(area, buf);
}

impl crate::prelude::Component for DashboardLineCell {
    fn digest(&mut self, _ctx: &crate::prelude::Context, event: &crate::listener::Event) {
        match self.hook.pull() {
            Some(QueryState::Loading) => {
                self.loading = true;
            }
            Some(QueryState::Success(responses)) => {
                self.loading = false;
                self.responses = responses;
                self.error = None;
            }
            Some(QueryState::Error(error)) => {
                self.loading = false;
                self.error = Some(error);
            }
            None => {}
        }
        match event {
            crate::listener::Event::Key(key)
                if key.code.as_char() == Some('R') && !self.loading =>
            {
                self.hook.execute();
            }
            _ => {}
        }
    }
}

impl ratatui::widgets::Widget for &DashboardLineCell {
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

        if let Some(response) = self.responses.get("default") {
            match response {
                Response::Timeseries(values) => {
                    render_timeseries(block, values, area, buf);
                }
                Response::Scalar(_values) => {
                    todo!()
                }
            }
        }
    }
}
