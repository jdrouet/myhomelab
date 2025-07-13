use myhomelab_prelude::time::{RelativeTimeRange, TimeRange};

fn render_relative(value: RelativeTimeRange, selected: bool, buf: &mut String) {
    buf.push_str("<option value=\"");
    buf.push_str(match value {
        RelativeTimeRange::LastHour => "last-hour",
        RelativeTimeRange::LastDay => "last-day",
        RelativeTimeRange::LastWeek => "last-week",
        RelativeTimeRange::LastMonth => "last-month",
    });
    buf.push_str("\"");
    if selected {
        buf.push_str(" selected");
    }
    buf.push('>');
    buf.push_str(match value {
        RelativeTimeRange::LastHour => "Last Hour",
        RelativeTimeRange::LastDay => "Last Day",
        RelativeTimeRange::LastWeek => "Last Week",
        RelativeTimeRange::LastMonth => "Last Month",
    });
    buf.push_str("</option>");
}

pub struct TimeRangeSelect {
    pub selected: TimeRange,
}

impl TimeRangeSelect {
    pub fn new(selected: TimeRange) -> Self {
        Self { selected }
    }
}

impl crate::prelude::Component for TimeRangeSelect {
    async fn render<C: crate::prelude::Context>(
        &self,
        _ctx: &C,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        buf.push_str("<form method=\"GET\">");
        buf.push_str("<select name=\"timerange\" onchange=\"this.form.submit()\">");
        render_relative(
            RelativeTimeRange::LastHour,
            matches!(
                self.selected,
                TimeRange::Relative(RelativeTimeRange::LastHour)
            ),
            buf,
        );
        render_relative(
            RelativeTimeRange::LastDay,
            matches!(
                self.selected,
                TimeRange::Relative(RelativeTimeRange::LastDay)
            ),
            buf,
        );
        render_relative(
            RelativeTimeRange::LastWeek,
            matches!(
                self.selected,
                TimeRange::Relative(RelativeTimeRange::LastWeek)
            ),
            buf,
        );
        render_relative(
            RelativeTimeRange::LastMonth,
            matches!(
                self.selected,
                TimeRange::Relative(RelativeTimeRange::LastMonth)
            ),
            buf,
        );
        buf.push_str("</select>");
        buf.push_str("<button title=\"Refresh\" type=\"submit\">♻</button>");
        buf.push_str("</form>");
        Ok(())
    }
}
