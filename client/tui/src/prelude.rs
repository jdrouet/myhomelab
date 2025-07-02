pub trait Component {
    fn digest(&mut self, event: crate::listener::Event);
}
