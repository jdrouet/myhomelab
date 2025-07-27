use myhomelab_metric::query::ScalarResponse;

pub struct ScalarTable<'a> {
    pub data: &'a [ScalarResponse],
}

impl<'a> ScalarTable<'a> {
    pub fn new(data: &'a [ScalarResponse]) -> Self {
        Self { data }
    }
}

impl<'a> crate::prelude::Component for ScalarTable<'a> {
    async fn render<C: crate::prelude::Context>(
        &self,
        _ctx: &C,
        buf: &mut String,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        buf.push_str("<table>");
        buf.push_str("<tbody>");
        for response in self.data {
            buf.push_str("<tr>");
            write!(
                buf,
                "<th>{:?}</th><td>{:.3}</td>",
                response.tags, response.value
            )?;
            buf.push_str("</tr>");
        }
        buf.push_str("</tbody>");
        buf.push_str("</table>");
        Ok(())
    }
}
