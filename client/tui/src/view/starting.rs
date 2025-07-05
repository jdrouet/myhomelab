use ratatui::style::Stylize;
use ratatui::text::Text;

#[derive(Debug, Default)]
pub(crate) struct StartingView;

impl crate::prelude::Component for StartingView {
    fn digest(&mut self, _ctx: &crate::prelude::Context, _event: &crate::listener::Event) {}
}

impl ratatui::widgets::Widget for &StartingView {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Text::from("Loading...".bold()).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Style, Stylize};
    use ratatui::widgets::Widget;

    use super::StartingView;

    #[test]
    fn should_render() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
        let view = StartingView::default();
        view.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "Loading...                                        ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
        ]);
        let bold_style = Style::new().bold();
        let normal_style = Style::new().not_bold();
        expected.set_style(Rect::new(0, 0, 10, 1), bold_style);
        expected.set_style(Rect::new(11, 0, 40, 1), normal_style);

        assert_eq!(buf, expected);
    }
}
