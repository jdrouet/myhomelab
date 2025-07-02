use ratatui::style::Stylize;
use ratatui::text::Text;
use tokio::sync::mpsc::UnboundedSender;

use crate::listener::Event;
use crate::worker::Action;

#[derive(Debug)]
pub(crate) struct StartingView {
    sender: UnboundedSender<Action>,
}

impl StartingView {
    pub(crate) fn new(sender: UnboundedSender<Action>) -> Self {
        Self { sender }
    }
}

impl crate::prelude::Component for StartingView {
    fn digest(&mut self, event: crate::listener::Event) {
        match event {
            Event::Key(key) if key.code.as_char() == Some('Q') => {
                let _ = self.sender.send(Action::Shutdown);
            }
            _ => {}
        }
    }
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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Style, Stylize};
    use ratatui::widgets::Widget;

    use super::StartingView;
    use crate::prelude::Component;
    use crate::worker::Action;

    #[test]
    fn should_render() {
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
        let view = StartingView::new(tx);
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

    #[test]
    fn should_trigger_shutdown() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut view = StartingView::new(tx);
        view.digest(crate::listener::Event::Key(KeyEvent::new(
            KeyCode::Char('Q'),
            KeyModifiers::empty(),
        )));
        let action = rx.blocking_recv().unwrap();
        assert!(matches!(action, Action::Shutdown));
    }
}
