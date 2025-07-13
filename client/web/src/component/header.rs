use myhomelab_prelude::time::TimeRange;

pub struct Header {
    pub timerange: TimeRange,
}

impl Header {
    pub fn new(timerange: TimeRange) -> Self {
        Self { timerange }
    }
}

impl crate::prelude::Component for Header {
    async fn render<C: crate::prelude::Context>(
        &self,
        ctx: &C,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        buf.push_str("<header>");
        buf.push_str("<div class=\"align-center flex-row\">");
        buf.push_str("<div class=\"flex-grow\">");
        buf.push_str("<a href=\"/\">MyHomeLab</a>");
        buf.push_str("</div>");
        crate::component::timerange_select::TimeRangeSelect::new(self.timerange)
            .render(ctx, buf)
            .await?;
        buf.push_str("</div>");
        buf.push_str("</header>");
        Ok(())
    }
}
