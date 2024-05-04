fn main() {
    use glfw::{Context, SwapInterval, WindowHint, WindowMode};

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    let (mut window, event_queue) = glfw
        .create_window(480, 320, "gl tutorial", WindowMode::Windowed)
        .unwrap();

    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol).cast());
    glfw.set_swap_interval(SwapInterval::Sync(1));
    if glfw.extension_supported("GL_ARB_gl_spirv") {
        println!("Extension 'GL_ARB_gl_spirv' is supported.");
    } else {
        println!("Extension 'GL_ARB_gl_spirv' is not supported.");
    }

    let mut my_app = MyApp::new();
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

struct MyApp {
    program: u32,
    vao: u32,
    vbo: u32,
    angle_y: f32,
}

impl MyApp {
    fn new() -> Self {
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
            let program = gl::CreateProgram();
            let shaders = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl::CreateShader(*shader_type);
                    let shader_source = format!("{shader_version}\n{shader_source}");
                    let _length = shader_source.len() as _;
                    gl::ShaderSource(shader, 1, [shader_source.as_ptr() as _].as_ptr(), &_length);
                    gl::CompileShader(shader);
                    {
                        let mut message = String::with_capacity(200);
                        gl::GetShaderInfoLog(
                            shader,
                            200,
                            std::ptr::null_mut(),
                            message.as_mut_ptr() as _,
                        );
                        assert!(
                            message.len() == 0,
                            "Failed to compile {shader_type}: {}",
                            message
                        );
                    }
                    gl::AttachShader(program, shader);
                    shader
                })
                .collect::<Vec<_>>();
            gl::LinkProgram(program);
            {
                let mut message = String::with_capacity(200);
                gl::GetProgramInfoLog(
                    program,
                    200,
                    std::ptr::null_mut(),
                    message.as_mut_ptr() as _,
                );
                assert!(message.len() == 0, "Failed to link: {}", message);
            }
            shaders.iter().for_each(|shader| {
                gl::DetachShader(program, *shader);
                gl::DeleteShader(*shader);
            });
            program
        };

        let (vbo, vao) = unsafe {
            let mut vbo = 0;
            gl::CreateBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let mut vao = 0;
            gl::CreateVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::EnableVertexAttribArray(MY_VERTEX_DATA_POS);
            gl::VertexAttribPointer(
                MY_VERTEX_DATA_POS,
                2,
                gl::FLOAT,
                gl::FALSE,
                5 * std::mem::size_of::<f32>() as i32,
                (0 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
            );
            gl::EnableVertexAttribArray(MY_VERTEX_DATA_COL);
            gl::VertexAttribPointer(
                MY_VERTEX_DATA_COL,
                3,
                gl::FLOAT,
                gl::FALSE,
                5 * std::mem::size_of::<f32>() as i32,
                (2 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
            );
            (vbo, vao)
        };

        Self {
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
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform1f(
                gl::GetUniformLocation(self.program, "u_angle_y".as_ptr() as _),
                self.angle_y,
            );
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (MY_VERTEX_DATA.len() * core::mem::size_of::<MyVertex>()) as isize,
                MY_VERTEX_DATA.as_ptr() as _,
                gl::STATIC_DRAW,
            );
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}

impl Drop for MyApp {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, [self.vao].as_ptr());
            gl::DeleteBuffers(1, [self.vbo].as_ptr());
        }
    }
}

struct MyVertex {
    pos: [f32; 2],
    col: [f32; 3],
}

static MY_VERTEX_DATA: [MyVertex; 3] = [
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
