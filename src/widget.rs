use crate::ui::{ElementHandle, UIBuilder, UIEvent};

pub trait Widget<D> {
    fn build(&self, parent: &UIBuilder<D>) -> ElementHandle;
    fn event(&mut self, event: UIEvent) {}
}

pub struct WidgetCallback<D>(pub fn(&mut D) -> &mut dyn Widget<D>);

impl<D> WidgetCallback<D> {
    pub fn event(&self, data: &mut D, event: UIEvent) {
        ((self.0)(data)).event(event)
    }
}
