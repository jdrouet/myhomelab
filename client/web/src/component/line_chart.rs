use myhomelab_metric::query::TimeseriesResponse;
use myhomelab_prelude::time::{TimeRange, current_timestamp};
use plotters::prelude::*;

pub struct LineChart<'a> {
    pub data: &'a [TimeseriesResponse],
    pub timerange: TimeRange,
}

impl<'a> LineChart<'a> {
    pub fn new(data: &'a [TimeseriesResponse], timerange: TimeRange) -> Self {
        Self { data, timerange }
    }

    fn range_x(&self) -> (u64, u64) {
        let absolute = self.timerange.into_absolute();
        let min_x = absolute.start;
        let max_x = absolute.end.unwrap_or_else(current_timestamp);
        (min_x, max_x)
    }

    fn range_y(&self) -> (f64, f64) {
        self.data
            .iter()
            .flat_map(|serie| serie.values.iter().map(|(_, value)| *value))
            .fold((f64::MAX, f64::MIN), |(prev_min, prev_max), item| {
                (prev_min.min(item), prev_max.max(item))
            })
    }
}

impl<'a> crate::prelude::Component for LineChart<'a> {
    async fn render<C: crate::prelude::Context>(
        &self,
        _ctx: &C,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        let (min_x, max_x) = self.range_x();
        let (min_y, max_y) = self.range_y();

        let backend = SVGBackend::with_string(buf, (1200, 400));
        let drawing_area = backend.into_drawing_area();
        let mut chart = ChartBuilder::on(&drawing_area)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .build_cartesian_2d(min_x..(max_x + 1), min_y..(max_y + 1.0))?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        for response in self.data.iter() {
            chart.draw_series(LineSeries::new(response.values.iter().copied(), RED))?;
        }

        drawing_area.present()?;

        Ok(())
    }
}
