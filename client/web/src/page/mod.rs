use crate::prelude::{Context, Page};

pub mod dashboard;
pub mod home;

#[derive(Debug)]
pub struct PageWrapper<P: Page> {
    page: P,
}

impl<P: Page> PageWrapper<P> {
    pub fn new(page: P) -> Self {
        Self { page }
    }

    pub async fn render<C>(&self, context: &C, buf: &mut String) -> anyhow::Result<()>
    where
        C: Context,
    {
        buf.push_str("<!DOCTYPE html>");
        buf.push_str("<html>");
        buf.push_str("<head>");
        buf.push_str("<title>");
        buf.push_str(self.page.title());
        buf.push_str("</title>");
        buf.push_str("<style>");
        buf.push_str(include_str!("./style.css"));
        buf.push_str("</style>");
        buf.push_str("</head>");
        buf.push_str("<body>");
        self.page.render_body(context, buf).await?;
        buf.push_str("</body>");
        buf.push_str("</html>");
        Ok(())
    }
}
