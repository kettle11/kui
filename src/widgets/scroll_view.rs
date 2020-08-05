use crate::drawing_info::*;
use crate::rectangle::Rectangle;
use crate::ui::{Element, ElementHandle, UIBuilder, Widget, WidgetHandle};

pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.0, 0.2, 1.0);

pub struct ScrollView {
    offset_y: f32,
    view_and_content: Option<(ElementHandle, ElementHandle)>,
}

impl ScrollView {
    fn new() -> Self {
        Self {
            offset_y: 0.,
            view_and_content: None,
        }
    }

    /// Returns true if pressed
    fn build<'a>(&mut self, parent: &UIBuilder<'a>, widget: WidgetHandle) -> UIBuilder<'a> {
        self.offset_y += parent.scroll_delta();
        //let parent_height = parent.element_rectangle(parent.handle()).y;

        // The root should take up as much vertical space as the children needs,
        // but no more than the parent.
        let root = parent.flexible();
        let inner = root
            .position_vertical_pixels(self.offset_y)
            .horizontal_expander()
            .fit()
            .fill((0., 0., 1., 1.));

        root.fill((0., 1., 0., 1.));

        // Scrollbar
        root.reverse_row().width(20.).custom_draw(widget);
        self.view_and_content = Some((root.handle(), inner.handle()));
        inner
    }
}

impl Widget for ScrollView {
    // The scrollbar must use a custom draw callback because it is drawn
    // based on the layout.
    fn draw(
        &mut self,
        rectangle: Rectangle,
        drawing_info: &mut DrawingInfo,
        elements: &Vec<Element>,
    ) {
        if let Some((view, content)) = self.view_and_content {
            let view_height = elements[view.0].rectangle.height;
            let content_height = elements[content.0].rectangle.height;

            let handle_height = (view_height / content_height) * view_height;
            let handle_offset = view_height * (self.offset_y / content_height);
            let fill_rectangle = (
                rectangle.x,
                rectangle.y - handle_offset,
                rectangle.width,
                handle_height,
            );
            drawing_info.drawables.push(Drawable {
                rectangle: fill_rectangle,
                texture_rectangle: (0., 0., 0., 0.),
                color: (1., 0., 0., 1.),
                radiuses: Some((10., 10., 10., 10.)),
            });
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
