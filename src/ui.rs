use fontdue;

use crate::layout::Layout;
use crate::rectangle::Rectangle;
use crate::render::Render;
use crate::texture::Texture;
use crate::tree::{NodeHandle, Tree};
use crate::widget::Widget;
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
    pub ui: &'a mut UI,
}

#[derive(Debug, Clone, Copy)]
pub enum UIEvent {
    PointerMoved,
    PointerDown,
    PointerUp,
    DoubleClick,
    /// Contains delta time in milliseconds since last animation frame
    AnimationFrame(f32),
}

#[derive(Debug)]
pub enum ElementType {
    Fill((f32, f32, f32, f32)),
    /// A rounded fill. The first f32s are corner radiuses, the second are colors.
    RoundedFill((f32, f32, f32, f32), (f32, f32, f32, f32)),
    /// A container that accepts a single element and constrains its width
    Width(f32),
    /// A container that accepts a single element and constrains its width by a percentage
    WidthPercentage(f32),
    /// A container that accepts a single element and constrains its height
    Height(f32),
    /// A container that accepts a single element and pads its width and height
    Padding(f32, f32),
    /// Rows lay out multiple elements in a row.
    /// Accepts a single value for spacing between elements
    Row(f32),
    /// Reverse row lay out multiple elements in a row with the opposite alignment.
    ReverseRow(f32),
    /// Moves an element by a percentage of the parent towards the end of the parent.
    PositionHorizontalPercentage(f32),
    /// Moves an element towards the horizontal end of the parent.
    PositionHorizontalPixels(f32),
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
    /// Takes up all horizontal space
    ExpanderHorizontal,
    /// Takes up all vertical space
    ExpanderVertical,
    /// Is the size of its children
    Fit,
}

pub struct Element {
    pub element_type: ElementType,
    pub rectangle: Rectangle,
}

pub struct UI {
    tree: Tree,
    root: NodeHandle,
    elements: Vec<Element>,
    width: f32,
    height: f32,
    drawing_info: DrawingInfo,
    fonts: Vec<fontdue::Font>,
    pointer_x: f32,
    pointer_y: f32,
    last_animation_timestamp: Option<std::time::Instant>,
    animation_frame_requested: bool,
}

impl UI {
    pub fn new() -> Self {
        let mut ui = Self {
            tree: Tree::new(),
            elements: Vec::new(),
            //widget_callbacks: Vec::new(),
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
            last_animation_timestamp: None,
            animation_frame_requested: false,
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
            //   self.widget_callbacks.push(None);
            self.elements.push(element)
        } else {
            //  self.widget_callbacks[new_handle.0] = None;
            self.elements[new_handle.0] = element;
        }
        new_handle
    }

    pub fn edit<'a>(&'a mut self) -> UIBuilder {
        let root = self.root;
        self.tree.remove(self.root);
        self.root = self.add(ElementType::Expander, None);
        UIBuilder {
            ui: Rc::new(RefCell::new(self)),
            parent: Some(root),
            current_widget: None,
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

    /// Move the pointer (mouse or touch) and trigger any potential events.
    pub fn pointer_move(&mut self, widget: &mut impl Widget, x: f32, y: f32) {
        // let old_x = self.pointer_x;
        // let old_y = self.pointer_y;
        self.pointer_x = x;
        self.pointer_y = y;
        widget.event(self, UIEvent::PointerMoved);
    }

    pub fn pointer_down(&mut self, widget: &mut impl Widget, x: f32, y: f32) {
        self.pointer_x = x;
        self.pointer_y = y;
        widget.event(self, UIEvent::PointerDown);
    }

    pub fn pointer_up(&mut self, widget: &mut impl Widget, x: f32, y: f32) {
        self.pointer_x = x;
        self.pointer_y = y;
        widget.event(self, UIEvent::PointerUp);
    }

    pub fn animate(&mut self, widget: &mut impl Widget) {
        self.animation_frame_requested = false;
        let elapsed = if let Some(last_time_stamp) = self.last_animation_timestamp {
            last_time_stamp.elapsed().as_secs_f32() * 1000.
        } else {
            0.
        };

        // Cap animation frames at 33 ms (30 fps) deltas.
        // This means that at lower framerates things will animate slower.
        // This also means that after a long period of time the animation will jump forward.
        // This probably requires a better solution.
        let elapsed = elapsed.min(33.);
        widget.event(self, UIEvent::AnimationFrame(elapsed));
        self.last_animation_timestamp = Some(std::time::Instant::now());
    }

    pub fn element_rectangle(&self, element: ElementHandle) -> Rectangle {
        self.elements[element.0].rectangle
    }

    pub fn pointer_position(&self) -> (f32, f32) {
        (self.pointer_x, self.pointer_y)
    }

    pub fn needs_redraw(&self) -> bool {
        self.animation_frame_requested
    }

    pub fn request_animation_frame(&mut self) {
        self.animation_frame_requested = true;
    }

    pub fn pointer_in_element(&self, element: ElementHandle) -> bool {
        self.elements[element.0]
            .rectangle
            .contains(self.pointer_x, self.pointer_y)
    }

    pub fn build(&mut self, widget: &mut impl Widget) {
        let editor = self.edit();
        widget.build(&editor);
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
    pub radiuses: Option<(f32, f32, f32, f32)>,
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
pub struct UIBuilder<'a> {
    ui: Rc<RefCell<&'a mut UI>>,
    parent: Option<NodeHandle>,
    current_widget: Option<usize>,
}

impl<'a> UIBuilder<'a> {
    pub fn add(&self, element_type: ElementType) -> Self {
        let new_container = self.ui.borrow_mut().add(element_type, self.parent);
        UIBuilder {
            ui: self.ui.clone(),
            parent: Some(new_container),
            current_widget: self.current_widget,
        }
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

    pub fn horizontal_expander(&self) -> Self {
        self.add(ElementType::ExpanderHorizontal)
    }

    pub fn vertical_expander(&self) -> Self {
        self.add(ElementType::ExpanderVertical)
    }

    pub fn width(&self, width_pixels: f32) -> Self {
        self.add(ElementType::Width(width_pixels))
    }

    /// Percentage of parent
    pub fn width_percentage(&self, width_percentage: f32) -> Self {
        self.add(ElementType::WidthPercentage(width_percentage))
    }

    pub fn padding(&self, padding: f32) -> Self {
        self.add(ElementType::Padding(padding, padding))
    }

    pub fn padding_horizontal(&self, padding: f32) -> Self {
        self.add(ElementType::Padding(padding, 0.))
    }

    pub fn padding_vertical(&self, padding: f32) -> Self {
        self.add(ElementType::Padding(0., padding))
    }

    pub fn height(&self, height_pixels: f32) -> Self {
        self.add(ElementType::Height(height_pixels))
    }

    /// Draw a rectangle that fills the entire available space
    pub fn fill(&self, color: (f32, f32, f32, f32)) -> Self {
        self.add(ElementType::Fill(color))
    }

    /// Draw a rectangle that fills the entire available space
    pub fn rounded_fill(&self, color: (f32, f32, f32, f32), radius: f32) -> Self {
        self.add(ElementType::RoundedFill(
            (radius, radius, radius, radius),
            color,
        ))
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

    pub fn position_horizontal_percentage(&self, percentage: f32) -> Self {
        self.add(ElementType::PositionHorizontalPercentage(percentage))
    }

    pub fn position_horizontal_pixels(&self, pixels: f32) -> Self {
        self.add(ElementType::PositionHorizontalPixels(pixels))
    }
}
