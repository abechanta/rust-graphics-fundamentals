use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "macroquad tutorial / 3D".to_owned(),
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
    let ferris = load_texture("ferris.png").await.unwrap();

    while !is_key_pressed(KeyCode::Escape) {
        clear_background(Color::new(0.2, 0.2, 0.2, 1.));

        // 右手系
        let camera1 = Camera3D {
            position: vec3(0., 15., 15.),
            up: Vec3::Y,
            target: Vec3::ZERO,
            ..Default::default()
        };
        set_camera(&camera1);

        draw_grid(20, 1., BLACK, GRAY);

        let cubes = &[
            (Vec3::new(-6., 0., -6.), Color::new(1., 0., 0., 1.)),
            (Vec3::new(0., 0., -6.), Color::new(0., 1., 0., 1.)),
            (Vec3::new(6., 0., -6.), Color::new(0., 0., 1., 1.)),
        ];
        let spheres = &[
            (Vec3::new(-6., 0., 0.), Color::new(1., 0., 0., 1.)),
            (Vec3::new(0., 0., 0.), Color::new(0., 1., 0., 1.)),
            (Vec3::new(6., 0., 0.), Color::new(0., 0., 1., 1.)),
        ];
        cubes.iter().for_each(|(pos, color)| {
            draw_cube(*pos, Vec3::splat(4.), None, *color);
        });
        spheres.iter().for_each(|(pos, color)| {
            draw_sphere(*pos, 1.2, None, *color);
        });
        {
            draw_plane(vec3(0., 0.1, 5.), vec2(5., 3.), Some(&ferris), WHITE);
        }

        set_default_camera();
        {
            let text = format!("FPS: {:?} / POS: {:?}", get_fps(), mouse_position());
            draw_text(&text, 0., 24. / 2., 24., WHITE);
        }

        next_frame().await
    }
}
