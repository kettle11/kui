use crate::drawing_info::*;
use crate::rectangle::Rectangle;
use crate::render::Render;
use crate::ui::{ElementHandle, TextProperties, UIBuilder, Widget, WidgetHandle};

pub const HANDLE_COLOR: (f32, f32, f32, f32) = (0.2, 0.2, 0.2, 1.0);

pub struct ScrollView {
    offset_y: f32,
    view_and_content: Option<(ElementHandle, ElementHandle)>,
    handle_rectangle: Option<(f32, f32, f32, f32)>,
}

impl ScrollView {
    fn new() -> Self {
        Self {
            offset_y: 0.,
            view_and_content: None,
            handle_rectangle: None,
        }
    }

    /// Returns true if pressed
    fn build<'a>(&mut self, parent: &UIBuilder<'a>, widget: WidgetHandle) -> UIBuilder<'a> {
        self.offset_y += parent.scroll_delta();

        // The root should take up as much vertical space as the children needs,
        // but no more than the parent.
        let root = parent.flexible().custom_draw(widget);
        let inner = root.horizontal_expander().fit();

        self.view_and_content = Some((root.handle(), inner.handle()));
        inner
    }
}

impl Widget for ScrollView {
    // The scrollbar must use a custom draw callback because it is drawn
    // based on the layout.
    // The regular view must go here as well to ensure that the scrolled element is
    // properly constrained to its parent region.
    fn draw(
        &mut self,
        context: &mut Render,
        element: ElementHandle,
        rectangle: Rectangle,
        text_properties: &TextProperties,
    ) {
        let (view, content) = self.view_and_content.unwrap();
        let view_height = context.elements[view.0].rectangle.height;
        let content_height = context.elements[content.0].rectangle.height;

        // Constrain the scroll.
        self.offset_y = self.offset_y.min(0.).max(-content_height + view_height);

        // The rectangle passed to the content of the scrollview.
        let child_rectangle = Rectangle::new(
            rectangle.x,
            self.offset_y,
            rectangle.width,
            rectangle.height,
        );

        // Scrollbar settings.
        let scrollbar_width = 10.;
        let scrollbar_right_margin = 10.;

        // First draw all content
        for child in context.tree.child_iter(element) {
            context.render_element(text_properties, child_rectangle, child);
        }

        // Then draw the scrollbar on top

        // Don't draw the scrollbar if it's not needed.
        if view_height < content_height {
            let handle_height = (view_height / content_height) * view_height;
            let handle_offset = view_height * (self.offset_y / content_height);
            let fill_rectangle = (
                rectangle.x + rectangle.width - scrollbar_width - scrollbar_right_margin,
                rectangle.y - handle_offset,
                scrollbar_width,
                handle_height,
            );
            context.drawing_info.drawables.push(Drawable {
                rectangle: fill_rectangle,
                texture_rectangle: (0., 0., 0., 0.),
                color: HANDLE_COLOR,
                radiuses: Some((10., 10., 10., 10.)),
            });
            self.handle_rectangle = Some(fill_rectangle);
        } else {
            self.handle_rectangle = None;
        }
    }
}

/// Returns the two sections divided by the divider
pub fn scroll_view<'a>(parent: &UIBuilder<'a>, id: u64) -> UIBuilder<'a> {
    let (handle, item) = parent.get_widget(id);
    let mut item = item.unwrap_or(Box::new(ScrollView::new()));
    let child_container = item.build(parent, handle);
    parent.add_widget(id, item);
    child_container
}
