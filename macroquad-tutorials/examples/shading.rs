use macroquad::prelude::*;
use std::f32::consts;

fn window_conf() -> Conf {
    Conf {
        window_title: "macroquad tutorial / shading".to_owned(),
        fullscreen: false,
        window_width: 480,
        window_height: 320,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    set_pc_assets_folder("../assets");
    // let ferris = load_texture("ferris.png").await.unwrap();

    let material = load_material(
        ShaderSource::Glsl {
            vertex: &MY_VERTEX_SHADER_SOURCE,
            fragment: &MY_FRAGMENT_SHADER_SOURCE,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                ..Default::default()
            },
            uniforms: vec![
                UniformDesc::new("view", UniformType::Float3),
                UniformDesc::new("light", UniformType::Float3),
            ],
            ..Default::default()
        },
    )
    .unwrap();

    while !is_key_pressed(KeyCode::Escape) {
        clear_background(Color::new(0.2, 0.2, 0.2, 1.));

        // 右手系
        let camera_position = {
            let pos = mouse_position();
            let yaw = ((-pos.0 - screen_width() / 2.) * consts::PI / screen_width()).sin_cos();
            let pitch = ((pos.1 - screen_height() / 2.) * consts::PI / screen_height()).sin_cos();
            vec3(yaw.0 * pitch.1, pitch.0, yaw.1 * pitch.1) * 6.
        };
        let camera1 = Camera3D {
            position: camera_position,
            up: Vec3::Y,
            target: Vec3::ZERO,
            ..Default::default()
        };
        set_camera(&camera1);
        material.set_uniform("view", camera1.position);

        draw_grid(20, 1., BLACK, GRAY);

        let light1 = {
            let height = ((get_time() / 2.) as f32).cos();
            let theta = ((get_time() / 3.1) as f32).sin_cos();
            vec3(4. * theta.0, 1. * height + 2., 4. * theta.1)
        };
        draw_sphere(light1, 0.2, None, WHITE);
        draw_line_3d(light1, light1.with_y(0.), YELLOW);
        material.set_uniform("light", light1);

        let cube_mesh = Mesh {
            vertices: MY_VERTEX_DATA.to_vec(),
            indices: MY_INDEX_DATA.to_vec(),
            texture: None,
            // texture: Some(ferris.clone()),
        };
        gl_use_material(&material);
        draw_mesh(&cube_mesh);

        gl_use_default_material();
        set_default_camera();
        {
            let text = format!("FPS: {:?} / POS: {:?}", get_fps(), mouse_position());
            draw_text(&text, 0., 24. / 2., 24., WHITE);
        }

        next_frame().await
    }
}

const MY_VERTEX_DATA: [Vertex; 24] = [
    // 0: top
    Vertex {
        position: Vec3::new(-1., 1., 1.),
        uv: Vec2::new(0., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 1., 0., 0.),
    },
    Vertex {
        position: Vec3::new(1., 1., 1.),
        uv: Vec2::new(1., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 1., 0., 0.),
    },
    Vertex {
        position: Vec3::new(-1., 1., -1.),
        uv: Vec2::new(0., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 1., 0., 0.),
    },
    Vertex {
        position: Vec3::new(1., 1., -1.),
        uv: Vec2::new(1., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 1., 0., 0.),
    },
    // 4: bottom
    Vertex {
        position: Vec3::new(-1., -1., 1.),
        uv: Vec2::new(0., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., -1., 0., 0.),
    },
    Vertex {
        position: Vec3::new(1., -1., 1.),
        uv: Vec2::new(1., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., -1., 0., 0.),
    },
    Vertex {
        position: Vec3::new(-1., -1., -1.),
        uv: Vec2::new(0., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., -1., 0., 0.),
    },
    Vertex {
        position: Vec3::new(1., -1., -1.),
        uv: Vec2::new(1., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., -1., 0., 0.),
    },
    // 8: front
    Vertex {
        position: Vec3::new(-1., 1., 1.),
        uv: Vec2::new(0., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 0., 1., 0.),
    },
    Vertex {
        position: Vec3::new(1., 1., 1.),
        uv: Vec2::new(1., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 0., 1., 0.),
    },
    Vertex {
        position: Vec3::new(-1., -1., 1.),
        uv: Vec2::new(0., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 0., 1., 0.),
    },
    Vertex {
        position: Vec3::new(1., -1., 1.),
        uv: Vec2::new(1., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 0., 1., 0.),
    },
    // 12: back
    Vertex {
        position: Vec3::new(-1., 1., -1.),
        uv: Vec2::new(0., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 0., -1., 0.),
    },
    Vertex {
        position: Vec3::new(1., 1., -1.),
        uv: Vec2::new(1., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 0., -1., 0.),
    },
    Vertex {
        position: Vec3::new(-1., -1., -1.),
        uv: Vec2::new(0., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 0., -1., 0.),
    },
    Vertex {
        position: Vec3::new(1., -1., -1.),
        uv: Vec2::new(1., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(0., 0., -1., 0.),
    },
    // 16: left
    Vertex {
        position: Vec3::new(-1., 1., -1.),
        uv: Vec2::new(0., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(-1., 0., 0., 0.),
    },
    Vertex {
        position: Vec3::new(-1., 1., 1.),
        uv: Vec2::new(1., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(-1., 0., 0., 0.),
    },
    Vertex {
        position: Vec3::new(-1., -1., -1.),
        uv: Vec2::new(0., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(-1., 0., 0., 0.),
    },
    Vertex {
        position: Vec3::new(-1., -1., 1.),
        uv: Vec2::new(1., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(-1., 0., 0., 0.),
    },
    // 20: right
    Vertex {
        position: Vec3::new(1., 1., -1.),
        uv: Vec2::new(0., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(1., 0., 0., 0.),
    },
    Vertex {
        position: Vec3::new(1., 1., 1.),
        uv: Vec2::new(1., 0.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(1., 0., 0., 0.),
    },
    Vertex {
        position: Vec3::new(1., -1., -1.),
        uv: Vec2::new(0., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(1., 0., 0., 0.),
    },
    Vertex {
        position: Vec3::new(1., -1., 1.),
        uv: Vec2::new(1., 1.),
        color: [255, 255, 255, 255],
        normal: Vec4::new(1., 0., 0., 0.),
    },
];

const MY_INDEX_DATA: [u16; 36] = [
    0, 1, 2, 1, 2, 3,
    4, 5, 6, 5, 6, 7,
    8, 9, 10, 9, 10, 11,
    12, 13, 14, 13, 14, 15,
    16, 17, 18, 17, 18, 19,
    20, 21, 22, 21, 22, 23,
];

const MY_VERTEX_SHADER_SOURCE: &'static str = r"
#version 330
precision lowp float;

uniform mat4 Model;
uniform mat4 Projection;
layout(location=0) in vec3 position;
layout(location=1) in vec2 texcoord;
layout(location=2) in vec4 color0;
layout(location=3) in vec3 normal;
layout(location=0) out vec3 v_position;
layout(location=1) out vec2 v_uv;
layout(location=2) out vec4 v_color;
layout(location=3) out vec3 v_normal;

void main() {
    vec4 pos = Model * vec4(position, 1);
    gl_Position = Projection * pos;
    v_position = vec3(pos);
    v_uv = texcoord;
    v_color = color0 / 255.0;
    v_normal = normalize(mat3(transpose(inverse(Model))) * normal);
}
";

const MY_FRAGMENT_SHADER_SOURCE: &'static str = r"
#version 330
precision lowp float;

uniform sampler2D Texture;
uniform vec3 view;
uniform vec3 light;
layout(location=0) in vec3 v_position;
layout(location=1) in vec2 v_uv;
layout(location=2) in vec4 v_color;
layout(location=3) in vec3 v_normal;
out vec4 FragColor;

void main() {
    vec3 lightDir = normalize(light - v_position);
    float diff = max(dot(lightDir, v_normal), 0.3);
    vec3 viewDir = normalize(v_position - view);
    float spec = 0.5 * pow(max(dot(viewDir, reflect(lightDir, v_normal)), 0.0), 64);
    vec4 intensity = vec4(diff + spec, diff + spec, diff + spec, 1.0);
    FragColor = v_color * texture(Texture, v_uv) * intensity;
}
";
