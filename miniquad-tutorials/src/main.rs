fn main() {
    use miniquad::conf::{AppleGfxApi, Conf};

    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    let conf = Conf {
        window_title: "miniquad tutorial".to_string(),
        window_width: 480,
        window_height: 320,
        platform: miniquad::conf::Platform {
            apple_gfx_api: if metal {
                AppleGfxApi::Metal
            } else {
                AppleGfxApi::OpenGl
            },
            ..Default::default()
        },
        ..Default::default()
    };

    miniquad::start(conf, move || Box::new(MyApp::new()));
}

use miniquad::{Bindings, Pipeline, RenderingBackend};

struct MyApp {
    pipeline: Pipeline,
    bindings: Bindings,
    ctx: Box<dyn RenderingBackend>,
    angle_y: f32,
}

impl MyApp {
    pub fn new() -> MyApp {
        use miniquad::{BufferSource, BufferType, BufferUsage};

        let mut ctx: Box<dyn RenderingBackend> = miniquad::window::new_rendering_backend();

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&MY_VERTICES),
        );
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&MY_INDICES),
        );
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };

        use miniquad::{Backend, ShaderSource};

        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: my_shader::MY_VERTEX_SHADER_SOURCE,
                        fragment: my_shader::MY_FRAGMENT_SHADER_SOURCE,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: my_shader::MY_METAL_SHADER_SOURCE,
                    },
                },
                my_shader::meta(),
            )
            .unwrap();

        use miniquad::{BufferLayout, PipelineParams, VertexAttribute, VertexFormat};

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_col", VertexFormat::Float3),
            ],
            shader,
            PipelineParams {
                ..Default::default()
            },
        );

        MyApp {
            pipeline,
            bindings,
            ctx,
            angle_y: 0.0,
        }
    }
}

use miniquad::{EventHandler, KeyCode, KeyMods};

impl EventHandler for MyApp {
    fn update(&mut self) {
        self.angle_y += std::f32::consts::PI / 60.0;
    }

    fn draw(&mut self) {
        use miniquad::UniformsSource;

        self.ctx.begin_default_pass(Default::default());
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx.apply_uniforms(UniformsSource::table(&MyUniforms {
            u_angle_y: self.angle_y,
        }));
        self.ctx.draw(0, 3, 1);
        self.ctx.end_render_pass();

        self.ctx.commit_frame();
    }

    fn key_down_event(&mut self, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if keycode == KeyCode::Escape {
            miniquad::window::quit();
        }
    }
}

#[repr(C)]
pub struct MyUniforms {
    pub u_angle_y: f32,
}

#[repr(C)]
struct MyVertex {
    pos: [f32; 2],
    col: [f32; 3],
}

#[rustfmt::skip]
const MY_VERTICES: [MyVertex; 3] = [
    MyVertex { pos: [0.8, 0.0], col: [1.0, 0.0, 0.0] },
    MyVertex { pos: [0.0, 0.8], col: [0.0, 1.0, 0.0] },
    MyVertex { pos: [-0.8, -0.8], col: [0.0, 0.0, 1.0] },
];

const MY_INDICES: [u16; 3] = [0, 1, 2];

mod my_shader {
    pub const MY_VERTEX_SHADER_SOURCE: &str = r#"
        #version 330
        precision mediump float;
        uniform float u_angle_y;
        layout(location = 0) in vec2 in_pos;
        layout(location = 1) in vec3 in_col;
        layout(location = 0) out vec4 v_color;

        void main() {
            gl_Position = vec4(in_pos, 0.0, 1.0);
            gl_Position.x *= cos(u_angle_y);
            v_color = vec4(in_col, 1.0);
        }
    "#;

    pub const MY_FRAGMENT_SHADER_SOURCE: &str = r#"
        #version 330
        precision mediump float;
        layout(location = 0) in vec4 v_color;
        out vec4 color;

        void main() {
            color = v_color;
        }
    "#;

    pub const MY_METAL_SHADER_SOURCE: &str = r#"
        #include <metal_stdlib>

        using namespace metal;

        struct Vertex
        {
            float2 in_pos [[attribute(0)]];
            float4 in_col [[attribute(1)]];
        };

        struct RasterizerData
        {
            float4 position [[position]];
            float4 color [[user(locn0)]];
        };

        vertex RasterizerData vertexShader(Vertex v [[stage_in]])
        {
            RasterizerData out;

            out.position = float4(v.in_pos.xy, 0.0, 1.0);
            out.color = v.in_col;
            return out;
        }

        fragment float4 fragmentShader(RasterizerData in [[stage_in]])
        {
            return in.color;
        }
    "#;

    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc};

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("u_angle_y", miniquad::UniformType::Float1)],
            },
        }
    }
}
