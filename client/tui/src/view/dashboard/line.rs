use myhomelab_dashboard::entity::Size;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};

use super::cell::DashboardLineCell;

fn column_count(size: Size) -> u8 {
    match size {
        Size::Small => 1,
        Size::Medium => 2,
        Size::Large => 3,
    }
}

fn view_width(size: Size) -> u8 {
    match size {
        Size::Small => 1,
        Size::Medium => 2,
        Size::Large => 4,
    }
}

pub(crate) struct DashboardLineIterator<'a> {
    cells: &'a [DashboardLineCell],
    view_size: Size,
}

impl<'a> DashboardLineIterator<'a> {
    pub fn new(cells: &'a [DashboardLineCell], view_size: Size) -> Self {
        Self { cells, view_size }
    }
}

impl<'a> Iterator for DashboardLineIterator<'a> {
    type Item = DashboardLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut space_left: u8 = view_width(self.view_size);
        let mut take = 0;
        while let Some(next) = self.cells.get(take) {
            let width = column_count(next.inner().width);
            if take == 0 || width <= space_left {
                take += 1;
                space_left -= width.min(space_left);
            } else {
                break;
            }
        }
        if take == 0 {
            return None;
        }
        let subset: &'a [DashboardLineCell] = &self.cells[..take];
        self.cells = &self.cells[take..];
        Some(DashboardLine {
            cells: subset,
            view_size: self.view_size,
        })
    }
}

pub(crate) struct DashboardLine<'a> {
    cells: &'a [DashboardLineCell],
    view_size: Size,
}

impl<'a> ratatui::widgets::Widget for &DashboardLine<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let total = column_count(self.view_size) as u32;
        let segments = Layout::horizontal(
            self.cells
                .iter()
                .map(|cell| Constraint::Ratio(column_count(cell.inner().width) as u32, total)),
        )
        .split(area);
        for (area, cell) in segments.iter().zip(self.cells) {
            cell.render(*area, buf);
        }
    }
}
