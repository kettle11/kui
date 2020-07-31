use glow::*;
use kapp::*;
use kui::widgets::*;
use kui::*;

mod gl_drawer;
fn main() {
    let (app, event_loop) = initialize();
    let mut window_width = 800;
    let mut window_height = 800;
    let window = app
        .new_window()
        .title("Mail")
        .dimensions(window_width, window_height)
        .build()
        .unwrap();
    println!("Hello, world!");

    // Create a GLContext
    let mut gl_context = GLContext::new().build().unwrap();
    gl_context.set_window(Some(&window)).unwrap();

    #[cfg(target_arch = "wasm32")]
    let gl = glow::Context::from_webgl2_context(gl_context.webgl2_context().unwrap());
    #[cfg(not(target_arch = "wasm32"))]
    let gl = glow::Context::from_loader_function(|s| gl_context.get_proc_address(s));

    unsafe {
        // gl.enable(SCISSOR_TEST);
        gl.viewport(0, 0, 800, 800);
    }

    let mut ui = UI::new();
    let inter_medium = ui.font_from_bytes(include_bytes!("../../resources/Inter-Medium.ttf")); //&std::fs::read("resources/Inter-Medium.ttf").unwrap());
    let material_icons =
        ui.font_from_bytes(include_bytes!("../../resources/MaterialIcons-Regular.ttf")); //&std::fs::read("resources/MaterialIcons-Regular.ttf").unwrap());
    ui.resize(window_width as f32, window_height as f32);

    const GRAY: (f32, f32, f32, f32) = (0.6, 0.6, 0.6, 1.0);

    struct MainView {
        font: FontHandle,
        button: Button,
        slider: Slider,
        element: Option<ElementHandle>,
    }

    let mut main_view = MainView {
        font: inter_medium,
        button: Button::new("Click me"),
        slider: Slider::new(),
        element: None,
    };

    impl Widget for MainView {
        fn build(&mut self, parent: &UIBuilder) {
            let top = parent
                .font(self.font)
                .height(100.)
                .expander()
                .fill(GRAY)
                .spaced_row(20.);

            top.width(0.); // For spacing
            self.button.build(&top.center_vertical());
            self.slider.build(&top.center_vertical());
            // self.element = Some(top.handle());
            top.width(0.); // For spacing
        }

        fn event(&mut self, ui: &mut UI, event: UIEvent) {
            self.button.event(ui, event);
            self.slider.event(ui, event);
        }
    }

    struct ToolBar<T: Widget> {
        children: Vec<T>,
        element: Option<ElementHandle>,
    }

    impl<T: Widget> ToolBar<T> {
        pub fn new(children: Vec<T>) -> Self {
            Self {
                children,
                element: None,
            }
        }
    }

    impl<T: Widget> Widget for ToolBar<T> {
        fn build(&mut self, parent: &UIBuilder) {
            let top = parent.height(100.).expander().fill(GRAY).row();
            for child in &mut self.children {
                child.build(&top);
            }
            self.element = Some(top.handle());
        }

        fn event(&mut self, ui: &mut UI, event: UIEvent) {
            for child in &mut self.children {
                child.event(ui, event);
            }
        }
    }

    ui.build(&mut main_view);

    let mut gl_drawer = gl_drawer::GLDrawer::new(&gl);

    event_loop.run(move |event| match event {
        Event::WindowCloseRequested { .. } => app.quit(),
        Event::WindowResized { width, height, .. } => {
            window_width = width;
            window_height = height;
            gl_context.resize();
            unsafe {
                gl_context.resize(); // Resizes the window buffer
                gl.viewport(0, 0, width as i32, height as i32);
            }
            ui.resize(width as f32, height as f32);
            window.request_redraw();
        }
        Event::MouseMoved { x, y, .. } => {
            ui.pointer_move(&mut main_view, x, y);
            window.request_redraw();
        }
        Event::MouseButtonDown { x, y, .. } => {
            ui.pointer_down(&mut main_view, x, y);
            window.request_redraw();
        }
        Event::MouseButtonUp { x, y, .. } => {
            ui.pointer_up(&mut main_view, x, y);
            window.request_redraw();
        }
        Event::Draw { .. } => unsafe {
            gl.clear_color(0.945, 0.945, 0.945, 1.0);
            gl.disable(CULL_FACE);
            gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

            ui.animate(&mut main_view);

            ui.build(&mut main_view);
            let drawables = ui.render();
            gl_drawer.draw(&gl, &drawables);

            if ui.needs_redraw() {
                window.request_redraw();
            }

            gl_context.swap_buffers();
        },
        _ => {}
    });
}
