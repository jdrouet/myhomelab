use myhomelab_dashboard::entity::Size;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
};

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

// #[cfg(test)]
// mod tests {
//     use std::collections::HashSet;

//     use myhomelab_dashboard::entity::{DashboardCell, Size};
//     use myhomelab_metric::{
//         entity::MetricHeader,
//         query::{Aggregator, Query, RequestKind},
//     };

//     use crate::view::dashboard::cell::DashboardLineCell;

//     #[test]
//     fn should_give_2_lines_with_3_small_cells() {
//         let available = (0..5)
//             .map(|index| DashboardCell {
//                 title: Some(format!("cell {index}")),
//                 height: Size::Small,
//                 width: Size::Small,
//                 kind: RequestKind::Scalar,
//                 query: Query {
//                     header: MetricHeader::new("foo", Default::default()),
//                     aggregator: Aggregator::Average,
//                     group_by: HashSet::default(),
//                 },
//             })
//             .map(DashboardLineCell::from)
//             .collect::<Vec<_>>();
//         let result = super::DashboardLineIterator::new(&available, Size::Large).collect::<Vec<_>>();
//         assert_eq!(result.len(), 2);
//     }

//     #[test]
//     fn should_give_3_lines_with_2_small_cells() {
//         let available = (0..5)
//             .map(|index| DashboardCell {
//                 title: Some(format!("cell {index}")),
//                 height: Size::Small,
//                 width: Size::Small,
//                 kind: RequestKind::Scalar,
//                 query: Query {
//                     header: MetricHeader::new("foo", Default::default()),
//                     aggregator: Aggregator::Average,
//                     group_by: HashSet::default(),
//                 },
//             })
//             .map(DashboardLineCell::from)
//             .collect::<Vec<_>>();
//         let result =
//             super::DashboardLineIterator::new(&available, Size::Medium).collect::<Vec<_>>();
//         assert_eq!(result.len(), 3);
//     }

//     #[test]
//     fn should_give_5_lines_with_1_small_cells() {
//         let available = (0..5)
//             .map(|index| DashboardCell {
//                 title: Some(format!("cell {index}")),
//                 height: Size::Small,
//                 width: Size::Small,
//                 kind: RequestKind::Scalar,
//                 query: Query {
//                     header: MetricHeader::new("foo", Default::default()),
//                     aggregator: Aggregator::Average,
//                     group_by: HashSet::default(),
//                 },
//             })
//             .map(DashboardLineCell::from)
//             .collect::<Vec<_>>();
//         let result = super::DashboardLineIterator::new(&available, Size::Small).collect::<Vec<_>>();
//         assert_eq!(result.len(), 5);
//     }

//     #[test]
//     fn should_give_2_lines_with_1_medium_cell_and_1_small_cell() {
//         let available = (0..5)
//             .map(|index| DashboardCell {
//                 title: Some(format!("cell {index}")),
//                 height: Size::Small,
//                 width: if index % 2 == 0 {
//                     Size::Small
//                 } else {
//                     Size::Medium
//                 },
//                 kind: RequestKind::Scalar,
//                 query: Query {
//                     header: MetricHeader::new("foo", Default::default()),
//                     aggregator: Aggregator::Average,
//                     group_by: HashSet::default(),
//                 },
//             })
//             .map(DashboardLineCell::from)
//             .collect::<Vec<_>>();
//         let result = super::DashboardLineIterator::new(&available, Size::Large).collect::<Vec<_>>();
//         assert_eq!(result.len(), 3);
//         assert_eq!(result[0].cells.len(), 2);
//         assert_eq!(result[1].cells.len(), 2);
//         assert_eq!(result[2].cells.len(), 1);
//     }

//     #[test]
//     fn should_give_5_lines_with_1_medium_cell_and_1_small_cell() {
//         let available = (0..5)
//             .map(|index| DashboardCell {
//                 title: Some(format!("cell {index}")),
//                 height: Size::Small,
//                 width: if index % 2 == 0 {
//                     Size::Small
//                 } else {
//                     Size::Medium
//                 },
//                 kind: RequestKind::Scalar,
//                 query: Query {
//                     header: MetricHeader::new("foo", Default::default()),
//                     aggregator: Aggregator::Average,
//                     group_by: HashSet::default(),
//                 },
//             })
//             .map(DashboardLineCell::from)
//             .collect::<Vec<_>>();
//         let result =
//             super::DashboardLineIterator::new(&available, Size::Medium).collect::<Vec<_>>();
//         assert_eq!(result.len(), 5);
//     }
// }
