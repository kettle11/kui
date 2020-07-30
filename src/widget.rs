use crate::ui::{UIBuilder, UIEvent, UI};

pub trait Widget {
    fn build(&mut self, parent: &UIBuilder);
    fn event(&mut self, ui: &mut UI, event: UIEvent);
}
