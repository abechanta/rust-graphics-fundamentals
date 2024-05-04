fn main() {
    use glfw::{Context, SwapInterval, WindowHint, WindowMode};

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    let (mut window, event_queue) = glfw
        .create_window(480, 320, "glow tutorial", WindowMode::Windowed)
        .unwrap();

    window.make_current();
    let gl = unsafe {
        use glow::Context;

        Context::from_loader_function(|symbol| window.get_proc_address(symbol).cast())
    };
    glfw.set_swap_interval(SwapInterval::Sync(1));
    if glfw.extension_supported("GL_ARB_gl_spirv") {
        println!("Extension 'GL_ARB_gl_spirv' is supported.");
    } else {
        println!("Extension 'GL_ARB_gl_spirv' is not supported.");
    }

    let mut my_app = MyApp::new(gl);
    let mut i = 0;
    window.set_all_polling(true);
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&event_queue) {
            use glfw::{Action, Key, WindowEvent};

            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                _ => {}
            }
        }

        my_app.update();
        my_app.render();
        window.swap_buffers();

        if i % 60 == 0 {
            println!("loop {}", i as f32 / 60.0);
        }
        i += 1;
    }
}

use glow as gl;
use glow::{Buffer, Context, HasContext, Program, VertexArray};

struct MyApp {
    gl: Context,
    program: Program,
    vao: VertexArray,
    vbo: Buffer,
    angle_y: f32,
}

impl MyApp {
    fn new(gl: Context) -> Self {
        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };
        let shader_sources = [
            (gl::VERTEX_SHADER, MY_VERTEX_SHADER_SOURCE),
            (gl::FRAGMENT_SHADER, MY_FRAGMENT_SHADER_SOURCE),
        ];

        let program = unsafe {
            let program = gl.create_program().unwrap();
            let shaders = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl.create_shader(*shader_type).unwrap();
                    let shader_source = format!("{shader_version}\n{shader_source}");
                    gl.shader_source(shader, shader_source.as_str());
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect::<Vec<_>>();
            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "Failed to link: {}",
                gl.get_program_info_log(program)
            );
            shaders.iter().for_each(|shader| {
                gl.detach_shader(program, *shader);
                gl.delete_shader(*shader);
            });
            program
        };

        let (vbo, vao) = unsafe {
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(vbo));

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            gl.enable_vertex_attrib_array(MY_VERTEX_DATA_POS);
            gl.vertex_attrib_pointer_f32(
                MY_VERTEX_DATA_POS,
                2,
                gl::FLOAT,
                false,
                5 * std::mem::size_of::<f32>() as i32,
                0 * std::mem::size_of::<f32>() as i32,
            );
            gl.enable_vertex_attrib_array(MY_VERTEX_DATA_COL);
            gl.vertex_attrib_pointer_f32(
                MY_VERTEX_DATA_COL,
                3,
                gl::FLOAT,
                false,
                5 * std::mem::size_of::<f32>() as i32,
                2 * std::mem::size_of::<f32>() as i32,
            );
            (vbo, vao)
        };

        Self {
            gl,
            program,
            vao,
            vbo,
            angle_y: 0.0,
        }
    }

    fn update(&mut self) {
        self.angle_y += std::f32::consts::PI / 60.0;
    }

    fn render(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.clear_color(0.2, 0.2, 0.2, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT);
        }
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_1_f32(
                gl.get_uniform_location(self.program, "u_angle_y").as_ref(),
                self.angle_y,
            );
            gl.buffer_data_u8_slice(
                gl::ARRAY_BUFFER,
                bytemuck::cast_slice(MY_VERTEX_DATA),
                gl::STATIC_DRAW,
            );
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(gl::TRIANGLES, 0, 3);
        }
    }
}

impl Drop for MyApp {
    fn drop(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MyVertex {
    pos: [f32; 2],
    col: [f32; 3],
}

static MY_VERTEX_DATA: &[MyVertex] = &[
    MyVertex {
        pos: [0.8, 0.0],
        col: [1.0, 0.0, 0.0],
    },
    MyVertex {
        pos: [0.0, 0.8],
        col: [0.0, 1.0, 0.0],
    },
    MyVertex {
        pos: [-0.8, -0.8],
        col: [0.0, 0.0, 1.0],
    },
];

use const_format::formatcp;
const MY_VERTEX_DATA_POS: u32 = 0;
const MY_VERTEX_DATA_COL: u32 = 1;
const MY_VERTEX_SHADER_SOURCE: &str = formatcp!(
    r"
    precision mediump float;
    uniform float u_angle_y;
    layout(location = {}) in vec2 in_pos;
    layout(location = {}) in vec3 in_col;
    out vec3 v_color;

    void main() {{
        gl_Position = vec4(in_pos, 0.0, 1.0);
        gl_Position.x *= cos(u_angle_y);
        v_color = in_col;
    }}
    ",
    MY_VERTEX_DATA_POS,
    MY_VERTEX_DATA_COL,
);
const MY_FRAGMENT_SHADER_SOURCE: &str = formatcp!(
    r"
    precision mediump float;
    in vec3 v_color;
    out vec4 f_color;

    void main() {{
        f_color = vec4(v_color, 1.0);
    }}
    ",
);
