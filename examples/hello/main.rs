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

    struct UIData {
        font: FontHandle,
        button0: Button,
        button1: Button,
        button2: Button,
        button3: Button,
        button4: Button,
    }

    let mut ui_data = UIData {
        font: inter_medium,
        button0: Button::new("Click me!"),
        button1: Button::new("Click me too!"),
        button2: Button::new("Another button"),
        button3: Button::new("Yet Another"),
        button4: Button::new("Last Button"),
    };

    fn build_ui(ui: &mut UI<UIData>, data: &mut UIData) {
        let ui = ui.edit(data);
        let parent = ui
            .fit()
            .font(ui.data().font)
            .height(400. as f32)
            .expander()
            .fill(GRAY)
            .spaced_row(40.);

        parent
            .center_vertical()
            .add_widget(|data| &mut data.button0);

        parent
            .center_vertical()
            .add_widget(|data| &mut data.button1);
        parent
            .center_vertical()
            .add_widget(|data| &mut data.button2);
        parent
            .center_vertical()
            .add_widget(|data| &mut data.button3);
        parent
            .center_vertical()
            .add_widget(|data| &mut data.button4);
    }

    build_ui(&mut ui, &mut ui_data);

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
            ui.pointer_move(x, y, &mut ui_data);
            window.request_redraw();
        }
        Event::MouseButtonDown { x, y, .. } => {
            ui.pointer_down(x, y, &mut ui_data);
            window.request_redraw();
        }
        Event::MouseButtonUp { x, y, .. } => {
            ui.pointer_up(x, y, &mut ui_data);
            window.request_redraw();
        }
        Event::Draw { .. } => unsafe {
            gl.clear_color(0.945, 0.945, 0.945, 1.0);
            gl.disable(CULL_FACE);
            gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

            if ui_data.button0.pressed() {
                println!("Button pressed");
            }

            ui.animate(&mut ui_data);
            build_ui(&mut ui, &mut ui_data);
            let drawables = ui.render();
            gl_drawer.draw(&gl, &drawables);

            if ui.needs_redraw() {
                window.request_redraw();
            }
            /*
            for drawable in drawables {
                let width = (drawable.rectangle.2) as i32;
                let height = (drawable.rectangle.3) as i32;

                let x = (drawable.rectangle.0) as i32;
                let y = (window_height as f32 - drawable.rectangle.1 - drawable.rectangle.3) as i32;

                gl.scissor(x, y, width, height);

                gl.clear_color(
                    drawable.color.0,
                    drawable.color.1,
                    drawable.color.2,
                    drawable.color.3,
                );

                gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            }
            */

            gl_context.swap_buffers();
        },
        _ => {}
    });
}
