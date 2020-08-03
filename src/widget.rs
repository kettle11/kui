use crate::ui::{UIBuilder, UIEvent, UI};
use std::any::Any;

pub trait Widget: ToAny {
    fn new() -> Self
    where
        Self: Sized;
    // fn build(&mut self, parent: &UIBuilder);
    fn event(&mut self, event: UIEvent) {}

    /*
    fn as_any(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }*/
    //  fn as_any(this: Box<Self>) -> Box<dyn Any>;
    // fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub trait ToAny {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Any> ToAny for T {
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

/*
struct WidgetContainer<T: Widget> {
    any: Box<dyn Any>,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Widget> WidgetContainer<T> {
    pub fn to_widget(self) -> Box<dyn Widget> {
        self.any.downcast::<T>().unwrap() as Box<dyn Widget>
    }
}*/
