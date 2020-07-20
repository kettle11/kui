use fontdue;

use crate::layout::Layout;
use crate::render::Render;
use crate::texture::Texture;
use crate::tree::{NodeHandle, Tree};

pub type ElementHandle = NodeHandle;

#[derive(Debug)]
pub enum ElementType {
    /// Draw a rectangle equivalent to the area of its children
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
    /// Lays out multiple elements evenly spaced
    EvenlySpacedRow,
    /// Columns lay out multiple elements in a column.
    /// Accepts a single value for spacing between elements
    Column(f32),
    /// Unstyled text
    Text(String),
    /// Specify text size for dependent elements,
    TextSize(f32),
    /// Centers children vertically in available space
    CenterVertical,
    /// Always takes up maximum available space
    Expander,
}

pub struct Element {
    pub element_type: ElementType,
    pub size: (f32, f32),
}

pub struct UI {
    tree: Tree,
    root: NodeHandle,
    elements: Vec<Element>,
    width: f32,
    height: f32,
    drawing_info: DrawingInfo,
    fonts: Vec<fontdue::Font>,
}

impl UI {
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
        };
        ui.add(ElementType::Expander, None);
        ui
    }

    pub fn font_from_bytes(&mut self, bytes: &[u8]) {
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap();
        self.fonts.push(font);
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
            size: (0., 0.),
        };
        // If the tree has allocated a new index, push the element there.
        if new_handle.0 >= self.elements.len() {
            self.elements.push(element)
        } else {
            self.elements[new_handle.0] = element;
        }
        new_handle
    }

    pub fn edit(&mut self) -> UIBuilder {
        let root = self.root;
        UIBuilder {
            ui: Rc::new(RefCell::new(self)),
            current_container: Some(root),
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
            (0., 0., self.width, self.height),
            self.root,
        );

        println!("Time: {:?}", now.elapsed().as_secs_f32());
        self.drawing_info.canvas_width = self.width;
        self.drawing_info.canvas_height = self.height;
        &self.drawing_info
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
pub struct UIBuilder<'a> {
    ui: Rc<RefCell<&'a mut UI>>,
    current_container: Option<NodeHandle>,
}

impl<'a> UIBuilder<'a> {
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

    /// The spacing value specifies spacing between elements.
    /// Use an empty width container to spacing at the start or end.
    pub fn row(&self, spacing: f32) -> Self {
        self.add(ElementType::Row(spacing))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty height container to spacing at the start or end.
    pub fn column(&self, spacing: f32) -> Self {
        self.add(ElementType::Column(spacing))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty height container to spacing at the start or end.
    pub fn reverse_row(&self, spacing: f32) -> Self {
        self.add(ElementType::ReverseRow(spacing))
    }

    pub fn evenly_spaced_row(&self) -> Self {
        self.add(ElementType::EvenlySpacedRow)
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
}

pub(crate) struct TextProperties {
    pub(crate) size: f32,
}

impl TextProperties {
    pub fn new() -> Self {
        Self {
            // 17 * 2. When DPI scaling is added change this to 17.
            // 17 is the recommended size for buttons on iOS, so a bit arbitrary.
            size: 34.,
        }
    }
}
