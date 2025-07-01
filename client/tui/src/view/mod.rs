use ratatui::{Terminal, prelude::Backend};
use ratatui::{
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::worker::Action;

mod starting;

#[derive(Debug)]
pub(crate) enum Route {
    Starting(starting::StartingView),
}

impl Route {
    pub fn digest(&mut self, event: &crate::listener::Event) {
        match self {
            Self::Starting(inner) => inner.digest(event),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Router {
    sender: UnboundedSender<Action>,
    route: Route,
}

impl Router {
    pub(crate) fn new(sender: UnboundedSender<Action>) -> Self {
        Self {
            sender: sender.clone(),
            route: Route::Starting(starting::StartingView::new(sender)),
        }
    }
}

impl Router {
    pub(crate) fn digest(&mut self, event: &crate::listener::Event) {
        self.route.digest(event);
    }

    pub(crate) fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
        Ok(())
    }
}

impl ratatui::widgets::Widget for &Router {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" MyHomeLab ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::default();

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
