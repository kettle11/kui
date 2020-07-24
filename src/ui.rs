use fontdue;

use crate::layout::Layout;
use crate::rectangle::Rectangle;
use crate::render::Render;
use crate::texture::Texture;
use crate::tree::{NodeHandle, Tree};
use crate::widget::{Widget, WidgetCallback};
pub type ElementHandle = NodeHandle;

#[derive(Copy, Clone, Debug)]
pub struct FontHandle(pub(crate) usize);

pub(crate) struct TextProperties {
    pub(crate) size: f32,
    pub(crate) font: Option<FontHandle>,
}

impl TextProperties {
    pub fn new() -> Self {
        Self {
            // 17 * 2. When DPI scaling is added change this to 17.
            // 17 is the recommended size for buttons on iOS, so a bit arbitrary.
            size: 34.,
            font: None,
        }
    }
}

pub struct EventContext<'a, 'b, T> {
    pub data: &'b mut T,
    pub event: UIEvent,
    pub node: NodeHandle,
    pub ui: &'a mut UI<T>,
}

#[derive(Debug, Clone, Copy)]
pub enum UIEvent {
    PointerDown,
    PointerUp,
    DoubleClick,
    PointerHover,
    PointerExited,
    GlobalPointerUp,
    /// Contains delta time in milliseconds since last animation frame
    AnimationFrame(f32),
}

#[derive(Debug)]
pub enum ElementType {
    Fill((f32, f32, f32, f32)),
    /// A container that accepts a single element and constrains its width
    Width(f32),
    /// A container that accepts a single element and constrains its height
    Height(f32),
    /// A container that accepts a single element and pads its width and height
    Padding(f32),
    /// Rows lay out multiple elements in a row.
    /// Accepts a single value for spacing between elements
    Row(f32),
    /// Reverse row lay out multiple elements in a row with the opposite alignment.
    ReverseRow(f32),
    /// Columns lay out multiple elements in a column.
    /// Accepts a single value for spacing between elements
    Column(f32),
    /// Unstyled text
    Text(String),
    /// Specify text size for dependent elements,
    TextSize(f32),
    /// Specifies font to use for children. Defaults to 'None'.
    Font(FontHandle),
    /// Centers children vertically in available space
    CenterVertical,
    /// Always takes up maximum available space
    Expander,
    /// Is the size of its children
    Fit,
}

pub struct Element {
    pub element_type: ElementType,
    pub rectangle: Rectangle,
}

pub struct UI<T> {
    tree: Tree,
    root: NodeHandle,
    elements: Vec<Element>,
    widget_callbacks: Vec<Option<WidgetCallback<T>>>,
    width: f32,
    height: f32,
    drawing_info: DrawingInfo,
    fonts: Vec<fontdue::Font>,
    pointer_x: f32,
    pointer_y: f32,
    event_manager: EventManager,
    last_animation_timestamp: Option<std::time::Instant>,
}

impl<T> UI<T> {
    pub fn new() -> Self {
        let mut ui = Self {
            tree: Tree::new(),
            elements: Vec::new(),
            widget_callbacks: Vec::new(),
            root: NodeHandle(0),
            width: 0.0,
            height: 0.0,
            drawing_info: DrawingInfo {
                canvas_width: 0.0,
                canvas_height: 0.0,
                drawables: Vec::new(),
                texture: Texture::new(512),
            },
            fonts: Vec::new(),
            pointer_x: 0.0,
            pointer_y: 0.0,
            event_manager: EventManager::new(),
            last_animation_timestamp: None,
        };
        ui.add(ElementType::Expander, None);
        ui
    }

    pub fn font_from_bytes(&mut self, bytes: &[u8]) -> FontHandle {
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap();
        self.fonts.push(font);
        FontHandle(self.fonts.len() - 1)
    }

    /// Directly add an element to the UI tree.
    pub fn add(
        &mut self,
        element_type: ElementType,
        parent: Option<ElementHandle>,
    ) -> ElementHandle {
        let new_handle = self.tree.add(parent);
        let element = Element {
            element_type,
            rectangle: Rectangle::zero(),
        };
        // If the tree has allocated a new index, push the element there.
        if new_handle.0 >= self.elements.len() {
            self.widget_callbacks.push(None);
            self.elements.push(element)
        } else {
            self.widget_callbacks[new_handle.0] = None;
            self.elements[new_handle.0] = element;
        }
        new_handle
    }

    pub fn edit<'a>(&'a mut self, data: &'a mut T) -> UIBuilder<T> {
        let root = self.root;
        self.event_manager.clear();
        self.tree.remove(self.root);
        self.root = self.add(ElementType::Expander, None);
        UIBuilder {
            ui: Rc::new(RefCell::new(self)),
            data: Rc::new(RefCell::new(data)),
            parent: Some(root),
        }
    }

    /*

    pub fn edit_element(&mut self, node: NodeHandle) -> UIBuilder<T> {
        UIBuilder {
            ui: Rc::new(RefCell::new(self)),
            parent: Some(node),
        }
    }
    */

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn animate(&mut self, data: &mut T) {
        let elapsed = if let Some(last_time_stamp) = self.last_animation_timestamp {
            last_time_stamp.elapsed().as_secs_f32() * 1000.
        } else {
            0.
        };
        // Animation elements listening for animation frames.
        for node in &self.event_manager.animation_frame {
            self.send_event_to_node(data, *node, UIEvent::AnimationFrame(elapsed));
        }
        self.last_animation_timestamp = Some(std::time::Instant::now());
    }

    pub fn render(&mut self) -> &DrawingInfo {
        // First layout the elements.
        // Calculate the sizes for various elements.
        let mut layout = Layout {
            fonts: &self.fonts,
            tree: &self.tree,
            elements: &mut self.elements,
        };
        let text_properties = TextProperties::new();

        layout.layout(&text_properties, self.root);

        self.drawing_info.drawables.clear();

        // Then render the final outputs based on the previously calculated sizes.
        let mut render = Render {
            fonts: &self.fonts,
            tree: &self.tree,
            elements: &mut self.elements,
            drawing_info: &mut self.drawing_info,
        };

        render.render_element(
            &text_properties,
            Rectangle::new(0., 0., self.width, self.height),
            self.root,
        );

        //println!("Time: {:?}", now.elapsed().as_secs_f32());
        self.drawing_info.canvas_width = self.width;
        self.drawing_info.canvas_height = self.height;
        &self.drawing_info
    }

    fn find_touched_nodes_with_handlers(
        &self,
        event: UIEvent,
        data: &mut T,
        node: NodeHandle,
        x: f32,
        y: f32,
    ) {
        if self.elements[node.0].rectangle.contains(x, y) {
            self.send_event_to_node(data, node, event);

            for child in self.tree.child_iter(node) {
                self.find_touched_nodes_with_handlers(event, data, child, x, y);
            }
        }
    }

    fn deliver_pointer_move_event(
        &self,
        data: &mut T,
        node: NodeHandle,
        x: f32,
        y: f32,
        old_x: f32,
        old_y: f32,
    ) {
        if self.elements[node.0].rectangle.contains(x, y) {
            self.send_event_to_node(data, node, UIEvent::PointerHover);

            for child in self.tree.child_iter(node) {
                self.deliver_pointer_move_event(data, child, x, y, old_x, old_y);
            }
        } else if self.elements[node.0].rectangle.contains(old_x, old_y) {
            self.send_event_to_node(data, node, UIEvent::PointerExited);
            for child in self.tree.child_iter(node) {
                self.deliver_pointer_move_event(data, child, x, y, old_x, old_y);
            }
        }
    }

    pub fn send_event_to_node(&self, data: &mut T, node: NodeHandle, event: UIEvent) {
        if let Some(callback) = &self.widget_callbacks[node.0] {
            callback.event(data, event);
        }
    }
    /// Move the pointer (mouse or touch) and trigger any potential events.
    pub fn pointer_move(&mut self, x: f32, y: f32, data: &mut T) {
        let old_x = self.pointer_x;
        let old_y = self.pointer_y;
        self.pointer_x = x;
        self.pointer_y = y;
        self.deliver_pointer_move_event(data, self.root, x, y, old_x, old_y);
    }

    pub fn pointer_down(&mut self, x: f32, y: f32, data: &mut T) {
        self.pointer_x = x;
        self.pointer_y = y;
        self.find_touched_nodes_with_handlers(UIEvent::PointerDown, data, self.root, x, y);
    }

    pub fn pointer_up(&mut self, x: f32, y: f32, data: &mut T) {
        self.pointer_x = x;
        self.pointer_y = y;

        for node in &self.event_manager.global_pointer_up {
            self.send_event_to_node(data, *node, UIEvent::GlobalPointerUp);
        }
        self.find_touched_nodes_with_handlers(UIEvent::PointerUp, data, self.root, x, y);
    }

    pub fn log_tree(&self) {
        println!("Nodes: {:?}", self.tree.nodes);
    }

    pub fn needs_redraw(&self) -> bool {
        self.event_manager.animation_frame.len() > 0
    }
}

#[derive(Debug)]
pub struct Drawable {
    pub rectangle: (f32, f32, f32, f32),
    pub texture_rectangle: (f32, f32, f32, f32),
    pub color: (f32, f32, f32, f32),
}

pub struct DrawingInfo {
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub texture: crate::texture::Texture,
    pub drawables: Vec<Drawable>,
}

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct UIBuilder<'a, T> {
    ui: Rc<RefCell<&'a mut UI<T>>>,
    data: Rc<RefCell<&'a mut T>>,
    parent: Option<NodeHandle>,
}

impl<'a, T> UIBuilder<'a, T> {
    pub fn add(&self, element_type: ElementType) -> Self {
        let new_container = self.ui.borrow_mut().add(element_type, self.parent);
        UIBuilder {
            ui: self.ui.clone(),
            data: self.data.clone(),
            parent: Some(new_container),
        }
    }

    pub fn data(&self) -> std::cell::Ref<'_, T> {
        std::cell::Ref::map(self.data.borrow(), |d| &**d)
    }

    pub fn handle(&self) -> NodeHandle {
        self.parent.unwrap()
    }

    pub fn row(&self) -> Self {
        self.add(ElementType::Row(0.))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty width container to spacing at the start or end.
    pub fn spaced_row(&self, spacing: f32) -> Self {
        self.add(ElementType::Row(spacing))
    }

    pub fn column(&self) -> Self {
        self.add(ElementType::Column(0.))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty height container to spacing at the start or end.
    pub fn spaced_column(&self, spacing: f32) -> Self {
        self.add(ElementType::Column(spacing))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty height container to spacing at the start or end.
    pub fn reverse_row(&self) -> Self {
        self.add(ElementType::ReverseRow(0.))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty height container to spacing at the start or end.
    pub fn spaced_reverse_row(&self, spacing: f32) -> Self {
        self.add(ElementType::ReverseRow(spacing))
    }

    pub fn expander(&self) -> Self {
        self.add(ElementType::Expander)
    }

    pub fn width(&self, width_pixels: f32) -> Self {
        self.add(ElementType::Width(width_pixels))
    }

    pub fn padding(&self, padding: f32) -> Self {
        self.add(ElementType::Padding(padding))
    }

    pub fn height(&self, height_pixels: f32) -> Self {
        self.add(ElementType::Height(height_pixels))
    }

    /// Draw a rectangle that fills the entire available space
    pub fn fill(&self, color: (f32, f32, f32, f32)) -> Self {
        self.add(ElementType::Fill(color))
    }

    pub fn center_vertical(&self) -> Self {
        self.add(ElementType::CenterVertical)
    }

    pub fn text(&self, text: &str) -> Self {
        self.add(ElementType::Text(text.to_owned()))
    }

    pub fn text_size(&self, size: f32) -> Self {
        self.add(ElementType::TextSize(size))
    }

    pub fn font(&self, font: FontHandle) -> Self {
        self.add(ElementType::Font(font))
    }

    pub fn fit(&self) -> Self {
        self.add(ElementType::Fit)
    }

    /// Passed in widget_path and the widget must refer to the same widget.
    pub fn add_widget(&mut self, widget_path: fn(&mut T) -> &mut dyn Widget<T>) {
        let data = &mut self.data.borrow_mut();
        let widget = (widget_path)(data);
        let (node, event_subscriptions) = widget.build(self);

        if event_subscriptions.local {
            self.ui.borrow_mut().widget_callbacks[node.0] = Some(WidgetCallback(widget_path));
        }

        if event_subscriptions.global_pointer_up {
            self.ui
                .borrow_mut()
                .event_manager
                .global_pointer_up
                .push(node);
        }

        if event_subscriptions.animation_frame {
            self.ui
                .borrow_mut()
                .event_manager
                .animation_frame
                .push(node);
        }
    }
}

struct EventManager {
    global_pointer_up: Vec<NodeHandle>,
    animation_frame: Vec<NodeHandle>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            global_pointer_up: Vec::new(),
            animation_frame: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.global_pointer_up.clear();
        self.animation_frame.clear();
    }
}
