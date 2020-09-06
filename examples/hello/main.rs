use glow::*;
use kapp::*;
use kui::widgets::*;
use kui::*;

mod gl_drawer;
fn main() {
    let (app, event_loop) = initialize();
    let mut window_width = 1200;
    let mut window_height = 800;
    let window = app
        .new_window()
        //  .without_titlebar()
        .title("Hello")
        .size(window_width, window_height)
        .build()
        .unwrap();
    println!("Hello, world!");

    // Create a GLContext
    let mut gl_context = GLContext::new().samples(4).build().unwrap();
    gl_context.set_window(Some(&window)).unwrap();

    // gl_context.set_vsync(VSync::Adaptive);

    #[cfg(target_arch = "wasm32")]
    let gl = glow::Context::from_webgl2_context(gl_context.webgl2_context().unwrap());
    #[cfg(not(target_arch = "wasm32"))]
    let gl = glow::Context::from_loader_function(|s| gl_context.get_proc_address(s));

    unsafe {
        // gl.enable(SCISSOR_TEST);
        gl.viewport(0, 0, window_width as i32, window_height as i32);
    }

    let mut ui = UI::new();
    let inter_medium = ui.font_from_bytes(include_bytes!("../../resources/Inter-Medium.ttf")); //&std::fs::read("resources/Inter-Medium.ttf").unwrap());
                                                                                               /*let material_icons =
                                                                                               ui.font_from_bytes(include_bytes!("../../resources/MaterialIcons-Regular.ttf")); //&std::fs::read("resources/MaterialIcons-Regular.ttf").unwrap());
                                                                                               */
    ui.resize(window_width as f32, window_height as f32);

    const GRAY: (f32, f32, f32, f32) = (0.6, 0.6, 0.6, 1.0);

    let mut gl_drawer = gl_drawer::GLDrawer::new(&gl);

    let mut letter = "A".to_string();
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
        Event::PointerMoved { x, y, .. } => {
            ui.pointer_move(x as f32, y as f32);
            window.request_redraw();
        }
        Event::PointerDown { x, y, .. } => {
            ui.pointer_down(x as f32, y as f32);
            window.request_redraw();
        }
        Event::PointerUp { x, y, .. } => {
            ui.pointer_up(x as f32, y as f32);
            window.request_redraw();
        }
        Event::Scroll { delta_y, .. } => {
            ui.scroll(delta_y as f32);
            window.request_redraw();
        }
        Event::Draw { .. } => unsafe {
            // ----------- Build UI ---------------
            let body = ui.edit().font(inter_medium).fill((0., 0., 0., 1.));

            let (first_section, second_section) = vertical_divider(&body, id!(), 600.);

            first_section
                .center()
                .scale_to_fit()
                .padding(50.)
                .text_size(500.)
                .text(&letter);

            let scroll = scroll_view(&second_section, id!());
            let scroll = scroll.flexible().text_size(40.).spaced_column(20.);

            for (i, c) in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().enumerate() {
                if button(
                    &scroll.center_horizontal(),
                    id!() + i as u64,
                    &c.to_string(),
                ) {
                    letter = c.to_string();
                    window.request_redraw();
                }
            }

            /*
            let body = ui.edit().font(inter_medium);
            let toolbar = body.height(200.).horizontal_expander().spaced_row(30.);
            if button(&toolbar, id!(), "Add Cube") {
                println!("Added a cube");
            }
            if button(&toolbar, id!(), "Add Sphere") {
                println!("Added a sphere");
            }*/

            // ----------- End Build UI ---------------

            gl.clear_color(0.945, 0.945, 0.945, 1.0);
            gl.disable(CULL_FACE);
            gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

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
