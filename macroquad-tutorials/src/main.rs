use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "macroquad tutorial".to_owned(),
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

        let circles = &[
            (Vec3::new(100., 100., 200.), Color::new(1., 0., 0., 1.)),
            (Vec3::new(300., 100., 200.), Color::new(0., 1., 0., 1.)),
            (Vec3::new(500., 100., 200.), Color::new(0., 0., 1., 1.)),
        ];
        circles.iter().for_each(|(pos, color)| {
            draw_circle(pos.x, pos.y, pos.z, *color);
        });
        {
            let pos = Vec2::new(screen_width() / 2., screen_height() / 2.) - ferris.size() / 2.;
            draw_texture(&ferris, pos.x, pos.y, WHITE);
        }
        {
            let text = format!("FPS: {:?} / POS: {:?}", get_fps(), mouse_position());
            draw_text(&text, 0., 24. / 2., 24., WHITE);
        }

        next_frame().await
    }
}
