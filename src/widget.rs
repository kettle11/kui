use crate::ui::{ElementHandle, UIBuilder, UIEvent};

pub struct EventSubscriptions {
    pub local: bool,
    pub global_pointer_up: bool,
    pub animation_frame: bool,
}

impl Default for EventSubscriptions {
    fn default() -> Self {
        Self {
            local: true,
            global_pointer_up: false,
            animation_frame: false,
        }
    }
}

pub trait Widget<D> {
    fn build(&self, parent: &UIBuilder<D>) -> (ElementHandle, EventSubscriptions);
    fn event(&mut self, _event: UIEvent) {}
}

pub struct WidgetCallback<D>(pub fn(&mut D) -> &mut dyn Widget<D>);

impl<D> WidgetCallback<D> {
    pub fn event(&self, data: &mut D, event: UIEvent) {
        ((self.0)(data)).event(event)
    }
}
