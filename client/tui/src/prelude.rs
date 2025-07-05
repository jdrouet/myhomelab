use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
pub struct Context {
    pub sender: UnboundedSender<crate::listener::Event>,
}

pub trait Component {
    fn digest(&mut self, ctx: &Context, event: &crate::listener::Event);
}
