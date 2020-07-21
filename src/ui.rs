use fontdue;

use crate::layout::Layout;
use crate::rectangle::Rectangle;
use crate::render::Render;
use crate::texture::Texture;
use crate::tree::{NodeHandle, Tree};

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

#[derive(Debug, Clone, Copy)]
pub enum UIEvent {
    Press,
    Release,
    DoublePress,
    Hover,
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

pub struct Element<T> {
    pub element_type: ElementType,
    pub event_handler: Option<Box<dyn FnMut(UIEvent, &mut T, NodeHandle, &mut UI<T>)>>,
    pub rectangle: Rectangle,
    phantom: std::marker::PhantomData<T>,
}

pub struct UI<T> {
    tree: Tree,
    root: NodeHandle,
    elements: Vec<Element<T>>,
    width: f32,
    height: f32,
    drawing_info: DrawingInfo,
    fonts: Vec<fontdue::Font>,
    mouse_x: f32,
    mouse_y: f32,
}

impl<T> UI<T> {
    pub fn new() -> Self {
        let mut ui = Self {
            tree: Tree::new(),
            elements: Vec::new(),
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
            mouse_x: 0.0,
            mouse_y: 0.0,
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
            event_handler: None,
            rectangle: Rectangle::zero(),
            phantom: std::marker::PhantomData,
        };
        // If the tree has allocated a new index, push the element there.
        if new_handle.0 >= self.elements.len() {
            self.elements.push(element)
        } else {
            self.elements[new_handle.0] = element;
        }
        new_handle
    }

    pub fn edit(&mut self) -> UIBuilder<T> {
        let root = self.root;
        UIBuilder {
            ui: Rc::new(RefCell::new(self)),
            current_container: Some(root),
        }
    }

    pub fn edit_element(&mut self, node: NodeHandle) -> UIBuilder<T> {
        UIBuilder {
            ui: Rc::new(RefCell::new(self)),
            current_container: Some(node),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn render(&mut self) -> &DrawingInfo {
        let now = std::time::Instant::now();

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

        println!("Time: {:?}", now.elapsed().as_secs_f32());
        self.drawing_info.canvas_width = self.width;
        self.drawing_info.canvas_height = self.height;
        &self.drawing_info
    }

    fn find_touched_nodes_with_handlers(
        &self,
        contained_nodes: &mut Vec<NodeHandle>,
        node: NodeHandle,
        x: f32,
        y: f32,
    ) {
        if self.elements[node.0].rectangle.contains(x, y) {
            if self.elements[node.0].event_handler.is_some() {
                contained_nodes.push(node);
            }
            for child in self.tree.child_iter(node) {
                self.find_touched_nodes_with_handlers(contained_nodes, child, x, y);
            }
        }
    }

    /// Move the mouse and trigger any potential events.
    pub fn move_mouse(&mut self, x: f32, y: f32, data: &mut T) {
        let mut nodes = Vec::new();
        self.find_touched_nodes_with_handlers(&mut nodes, self.root, x, y);
        for node in nodes {
            let mut handler = self.elements[node.0].event_handler.take().unwrap();
            (handler)(UIEvent::Hover, data, node, self);
            if self.elements[node.0].event_handler.is_none() {
                self.elements[node.0].event_handler = Some(handler);
            }
        }
    }

    pub fn log_tree(&self) {
        println!("Nodes: {:?}", self.tree.nodes);
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
    current_container: Option<NodeHandle>,
}

impl<'a, T> UIBuilder<'a, T> {
    pub fn add(&self, element_type: ElementType) -> Self {
        let new_container = self
            .ui
            .borrow_mut()
            .add(element_type, self.current_container);
        UIBuilder {
            ui: self.ui.clone(),
            current_container: Some(new_container),
        }
    }

    pub fn event_handler<F>(&self, event_handler: F)
    where
        F: FnMut(UIEvent, &mut T, NodeHandle, &mut UI<T>) + 'static,
    {
        let e = &mut self.ui.borrow_mut().elements[self.current_container.unwrap().0].event_handler;
        *e = Some(Box::new(event_handler));
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
}
