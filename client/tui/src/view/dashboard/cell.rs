use myhomelab_dashboard::entity::DashboardCell;
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Line, widgets::Block};

pub(crate) struct DashboardLineCell<'a>(&'a DashboardCell);

impl<'a> From<&'a DashboardCell> for DashboardLineCell<'a> {
    fn from(value: &'a DashboardCell) -> Self {
        Self(value)
    }
}

impl<'a> ratatui::widgets::Widget for &DashboardLineCell<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered();
        let block = match self.0.title {
            Some(ref title) => {
                let title = Line::from_iter([format!(" {title} ").italic()]);
                block.title(title)
            }
            None => block,
        };
        block.render(area, buf);
    }
}
