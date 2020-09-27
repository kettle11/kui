use crate::{Drawable, DrawingInfo};
use glow::*;

struct RenderData {
    program: <Context as HasContext>::Program,
    vertex_array_object: <Context as HasContext>::VertexArray,
    vertex_buffer: <Context as HasContext>::Buffer,
    element_buffer: <Context as HasContext>::Buffer,
    texture_atlas_uniform: Option<<Context as HasContext>::UniformLocation>,
    texture: <Context as HasContext>::Texture,
}
pub struct GLDrawer {
    render_data: RenderData,
}
#[derive(Debug)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}
#[derive(Debug)]

struct Vec2 {
    x: f32,
    y: f32,
}
#[derive(Debug)]

struct Vertex {
    position: Vec4,
    uv: Vec2,
    color: Vec4,
}

impl Vertex {
    pub fn new(
        position: (f32, f32, f32, f32),
        uv: (f32, f32),
        color: (f32, f32, f32, f32),
    ) -> Self {
        Vertex {
            position: Vec4 {
                x: position.0,
                y: position.1,
                z: position.2,
                w: position.3,
            },
            uv: Vec2 { x: uv.0, y: uv.1 },
            color: Vec4 {
                x: color.0,
                y: color.1,
                z: color.2,
                w: color.3,
            },
        }
    }
}
impl GLDrawer {
    pub fn new(gl: &Context) -> Self {
        panic_if_error(gl);

        // Create the shader program
        let program = new_shader_program(
            gl,
            include_str!("shaders/vertex.vs"),
            include_str!("shaders/fragment.fs"),
        );

        unsafe {
            let texture_atlas_uniform = gl.get_uniform_location(program, "textureAtlas");

            // Setup and bind the vao
            let vertex_array_object = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vertex_array_object));

            // Create the vertex buffer holding vertex buffer data
            let vertex_buffer = gl.create_buffer().unwrap();

            // Bind buffer to vertex_array_object
            gl.bind_buffer(ARRAY_BUFFER, Some(vertex_buffer));

            // Setup vertex attributes

            // Position
            gl.vertex_attrib_pointer_f32(
                0, // Index
                4, // Number of components
                FLOAT, false, 40,       // Stride
                0 as i32, // Offset
            );

            gl.vertex_attrib_pointer_f32(
                1, // Index
                2, // Number of components
                FLOAT, false, 40,        // Stride
                16 as i32, // Offset
            );

            gl.vertex_attrib_pointer_f32(
                2, // Index
                4, // Number of components
                FLOAT, false, 40,        // Stride
                24 as i32, // Offset
            );

            let element_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(element_buffer));

            let texture = gl.create_texture().unwrap();
            gl.bind_texture(TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as i32); // set texture wrapping to GL_REPEAT (default wrapping method)
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as i32);
            // set texture filtering parameters
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as i32);

            let render_data = RenderData {
                program,
                vertex_array_object,
                vertex_buffer,
                element_buffer,
                texture_atlas_uniform,
                texture,
            };
            panic_if_error(gl);

            GLDrawer { render_data }
        }
    }

    fn screen_to_gl(x: f32, y: f32, width: f32, height: f32) -> (f32, f32) {
        ((x / width) * 2.0 - 1.0, ((y / height) * 2.0 - 1.0) * -1.0)
    }

    fn rounded_rectangle(
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        drawable: &Drawable,
        r0: f32,
        r1: f32,
        r2: f32,
        r3: f32,
        width: f32,
        height: f32,
    ) {
        let rectangle = drawable.rectangle;
        let center = Self::screen_to_gl(
            rectangle.0 + rectangle.2 / 2.,
            rectangle.1 + rectangle.3 / 2.,
            width,
            height,
        );

        let min_radius = (rectangle.2 / 2.).min(rectangle.3 / 2.);
        let r0 = r0.min(min_radius);
        let r1 = r1.min(min_radius);
        let r2 = r2.min(min_radius);
        let r3 = r3.min(min_radius);

        let c = drawable.color;
        let center_index = vertices.len() as u32;

        vertices.push(Vertex::new((center.0, center.1, 0., 0.), (0., 0.), c));
        corner(
            r0,
            center_index,
            (rectangle.0 + r0, rectangle.1 + r0),
            std::f32::consts::PI,
            vertices,
            indices,
            &drawable.color,
            width,
            height,
        );
        corner(
            r1,
            center_index,
            (rectangle.0 - r1 + rectangle.2, rectangle.1 + r1),
            std::f32::consts::PI * 1.5,
            vertices,
            indices,
            &drawable.color,
            width,
            height,
        );
        corner(
            r2,
            center_index,
            (
                rectangle.0 - r2 + rectangle.2,
                rectangle.1 - r2 + rectangle.3,
            ),
            std::f32::consts::PI * 2.0,
            vertices,
            indices,
            &drawable.color,
            width,
            height,
        );
        corner(
            r3,
            center_index,
            (rectangle.0 + r3, rectangle.1 - r3 + rectangle.3),
            std::f32::consts::PI * 0.5,
            vertices,
            indices,
            &drawable.color,
            width,
            height,
        );

        // Push closing triangle
        indices.push(center_index);
        indices.push(vertices.len() as u32 - 1);
        indices.push(center_index + 1);

        fn corner(
            radius: f32,
            center_index: u32,
            corner_center: (f32, f32),
            start_angle: f32,
            vertices: &mut Vec<Vertex>,
            indices: &mut Vec<u32>,
            color: &(f32, f32, f32, f32),
            width: f32,
            height: f32,
        ) {
            let mut angle = start_angle;
            let steps = 20;
            let step_amount = (std::f32::consts::PI / 2.0) / steps as f32;
            for _ in 0..steps {
                let len = vertices.len() as u32;
                indices.push(center_index);
                indices.push(len - 1);
                indices.push(len);

                let position = GLDrawer::screen_to_gl(
                    corner_center.0 + angle.cos() * radius,
                    corner_center.1 + angle.sin() * radius,
                    width,
                    height,
                );

                vertices.push(Vertex::new(
                    (position.0, position.1, 0., 0.),
                    (0., 0.),
                    *color,
                ));
                angle += step_amount;
            }
        }
    }

    // Does not update the texture yet.
    fn update_data(&mut self, gl: &Context, drawing_info: &DrawingInfo) -> usize {
        panic_if_error(gl);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for drawable in &drawing_info.drawables {
            let vertices_len = vertices.len() as u32;
            let r0 = drawable.rectangle;

            if let Some((r0, r1, r2, r3)) = drawable.radiuses {
                Self::rounded_rectangle(
                    &mut vertices,
                    &mut indices,
                    &drawable,
                    r0,
                    r1,
                    r2,
                    r3,
                    drawing_info.canvas_width,
                    drawing_info.canvas_height,
                );
            } else {
                let r = (
                    (r0.0 / drawing_info.canvas_width) * 2.0 - 1.0,
                    ((r0.1 / drawing_info.canvas_height) * 2.0 - 1.0) * -1.0,
                    (r0.2 / drawing_info.canvas_width) * 2.0,
                    (r0.3 / drawing_info.canvas_height) * -2.0,
                );

                let t = drawable.texture_rectangle;

                let t0 = (t.0, t.1);
                let t1 = (t.0, t.1 + t.3);
                let t2 = (t.0 + t.2, t.1 + t.3);
                let t3 = (t.0 + t.2, t.1);

                // println!("t0: {:?}", t0);
                //  println!("t1: {:?}", t1);
                //  println!("t2: {:?}", t2);
                // println!("t3: {:?}", t3);

                let c = drawable.color;
                vertices.push(Vertex::new((r.0, r.1, 0., 0.), t0, c));
                vertices.push(Vertex::new((r.0, r.1 + r.3, 0., 0.), t1, c));
                vertices.push(Vertex::new((r.0 + r.2, r.1 + r.3, 0., 0.), t2, c));
                vertices.push(Vertex::new((r.0 + r.2, r.1, 0., 0.), t3, c));

                indices.push(vertices_len + 0);
                indices.push(vertices_len + 1);
                indices.push(vertices_len + 2);
                indices.push(vertices_len + 0);
                indices.push(vertices_len + 2);
                indices.push(vertices_len + 3);
            }
        }

        unsafe {
            // Update vertex data
            gl.bind_buffer(ARRAY_BUFFER, Some(self.render_data.vertex_buffer));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, slice_to_bytes(&vertices), STATIC_DRAW);

            // Update element buffer data
            gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(self.render_data.element_buffer));
            gl.buffer_data_u8_slice(ELEMENT_ARRAY_BUFFER, slice_to_bytes(&indices), STATIC_DRAW);
        }
        indices.len()
    }

    /// This does too much setup per call.
    pub fn draw(&mut self, gl: &Context, drawing_info: &DrawingInfo) {
        let index_count = self.update_data(gl, drawing_info);

        //  println!("DRAWABLE VERTICES: {:?}", drawable.vertices);
        unsafe {
            // These attributes need to be enabled.
            // In a normal GL program they're probably already enabled,
            // but just in case turn them on here.
            gl.enable_vertex_attrib_array(0); // Position
            gl.enable_vertex_attrib_array(1); // UV coordinates
            gl.enable_vertex_attrib_array(2); // Color

            // Alpha blending is required so that images can be transparent
            gl.enable(BLEND);
            gl.blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);

            // Setup the texture here.
            gl.bind_texture(TEXTURE_2D, Some(self.render_data.texture));

            gl.tex_image_2d(
                TEXTURE_2D,
                0,
                RED as i32,
                drawing_info.texture.width as i32,
                drawing_info.texture.height as i32,
                0,
                RED,
                UNSIGNED_BYTE,
                Some(&drawing_info.texture.data),
            );

            gl.generate_mipmap(TEXTURE_2D);
            panic_if_error(gl);

            // Finally draw!
            gl.use_program(Some(self.render_data.program));
            panic_if_error(gl);

            gl.bind_vertex_array(Some(self.render_data.vertex_array_object));

            gl.active_texture(TEXTURE0);
            panic_if_error(gl);

            gl.bind_texture(TEXTURE_2D, Some(self.render_data.texture));
            panic_if_error(gl);

            // Bind the uniform to the first slot.
            gl.uniform_1_i32(self.render_data.texture_atlas_uniform.as_ref(), 0);
            panic_if_error(gl);

            //s  println!("Drawing: {:?}", index_count);
            gl.draw_elements(TRIANGLES, index_count as i32, UNSIGNED_INT, 0);
            panic_if_error(gl);
        }
    }
}

fn panic_if_error(gl: &Context) {
    unsafe {
        let error = gl.get_error();
        if error != 0 {
            panic!("GL ERROR: {:?}", error)
        }
    }
}
unsafe fn slice_to_bytes<T>(t: &[T]) -> &[u8] {
    let ptr = t.as_ptr() as *const u8;
    let size = std::mem::size_of::<T>() * t.len();
    std::slice::from_raw_parts(ptr, size)
}

fn compile_shader(gl: &Context, shader_type: u32, source: &str) -> <Context as HasContext>::Shader {
    #[cfg(all(target_arch = "wasm32"))]
    let version = "#version 300 es";
    #[cfg(all(not(target_arch = "wasm32")))]
    let version = "#version 410";

    let source = &format!("{}\n{}", version, source);
    unsafe {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            println!("Type: {:?}", shader_type);
            println!("{}", source);
            println!("{}", gl.get_shader_info_log(shader));
            panic!();
        }

        shader
    }
}

pub fn new_shader_program(
    gl: &Context,
    vertex_source: &str,
    fragment_source: &str,
) -> <Context as HasContext>::Program {
    let vertex_shader = compile_shader(gl, VERTEX_SHADER, vertex_source);
    let fragment_shader = compile_shader(gl, FRAGMENT_SHADER, fragment_source);

    unsafe {
        let program = gl.create_program().unwrap();
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            println!("{}", gl.get_program_info_log(program));
            panic!();
        }

        program
    }
}
