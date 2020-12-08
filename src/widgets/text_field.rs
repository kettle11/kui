use crate::drawing_info::*;
use crate::rectangle::Rectangle;
use crate::render::Render;
use crate::ui::{ElementHandle, TextProperties, UIBuilder, Widget, WidgetHandle};

pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);

pub struct TextField {
    element: Option<ElementHandle>,
}

impl TextField {
    fn new() -> Self {
        Self { element: None }
    }

    /// Returns true if pressed
    fn build(&mut self, parent: &UIBuilder, text: &str, widget: WidgetHandle) {
        let top = parent.fit();
        let root = parent.flexible().custom_draw(widget);

        root.fill(DEFAULT_COLOR)
            .padding(2.)
            .fill((0., 0., 0., 1.))
            .padding(20.)
            .center_vertical()
            .text(text);
        self.element = Some(top.handle());
    }
}

impl Widget for TextField {
    // A custom draw implementation to draw the cursor.
    fn draw(
        &mut self,
        context: &mut Render,
        element: ElementHandle,
        rectangle: Rectangle,
        text_properties: &TextProperties,
    ) {
        // First draw all children
        for child in context.tree.child_iter(element) {
            context.render_element(text_properties, rectangle, child);
        }

        // Then draw cursor on top.
        context.drawing_info.drawables.push(Drawable {
            rectangle: (
                rectangle.x + rectangle.width - 18.,
                rectangle.y + 25.,
                4.,
                text_properties.size,
            ),
            texture_rectangle: (0., 0., 0., 0.),
            color: (1.0, 1., 1., 1.),
            radiuses: Some((0., 0., 0., 0.)),
        });
    }
}

pub fn text_field_with_id(parent: &UIBuilder, id: u64, placeholder_text: &str) {
    let (handle, widget) = parent.get_widget(id);
    let mut widget = widget.unwrap_or(Box::new(TextField::new()));
    widget.build(parent, placeholder_text, handle);
    parent.add_widget(id, widget);
}

/// Create a button
/// Returns true if the button is pressed.
/// Uses button text for ID calculation.
#[track_caller]
pub fn text_field(parent: &UIBuilder, placeholder_text: &str) {
    let id = super::calculate_id(placeholder_text);
    text_field_with_id(parent, id, placeholder_text)
}
