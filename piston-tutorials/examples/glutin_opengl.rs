// use piston::{*, Events, EventSettings};
// use piston_window::prelude::*;
use std::{env, f64, path};

#[pollster::main]
async fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("../assets");
        path
    } else {
        path::PathBuf::from("../assets")
    };

    use glutin_window::GlutinWindow;
    use piston::{Api, WindowSettings};
    use piston_window::PistonWindow;

    let mut window = WindowSettings::new("piston tutorial / glutin+gfx", [480, 320])
        .exit_on_esc(true)
        .vsync(true)
        .graphics_api(Api::opengl(3, 3))
        .build::<PistonWindow<GlutinWindow>>()
        .unwrap();

    let glyph_cache = window
        .load_font(resource_dir.join("LiberationMono-Regular.ttf"))
        .unwrap();

    let mut my_app = MyApp {
        text: "POS: ".into(),
        glyph_cache,
    };

    use piston::{Event, EventSettings, Events, Input, Loop, Motion};

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        match e {
            Event::Input(Input::Move(Motion::MouseCursor(args)), _) => {
                my_app.mouse_moved(&args);
            }
            Event::Loop(Loop::Update(args)) => {
                my_app.update(&args);
            }
            Event::Loop(Loop::Render(args)) => {
                window.draw_2d(&e, |c, g, device| {
                    my_app.render(&args, &c, g);
                    my_app.glyph_cache.factory.encoder.flush(device);
                });
            }
            _ => {}
        }
    }
}

use graphics::{context::Context, Graphics};
use piston::{RenderArgs, UpdateArgs};
use piston_window::{G2dTexture, Glyphs};

struct MyApp {
    text: String,
    glyph_cache: Glyphs,
}

impl MyApp {
    fn mouse_moved(self: &mut Self, args: &[f64; 2]) {
        self.text = format!("POS: {:?}", args);
    }

    fn update(self: &mut Self, _args: &UpdateArgs) {}

    fn render<G>(self: &mut Self, _args: &RenderArgs, c: &Context, g: &mut G)
    where
        G: Graphics<Texture = G2dTexture>,
    {
        use graphics::{clear, Transformed};

        clear([0.2; 4], g);

        let mut transform = c.transform.trans(100., 100.);
        let colors = [
            [1., 0., 0., 1.],
            [0., 1., 0., 1.],
            [0., 0., 1., 1.],
        ];
        colors.iter().for_each(|color| {
            use graphics::Ellipse;

            Ellipse::new(*color).draw([-200., -200., 400., 400.], &c.draw_state, transform, g);
            transform = transform.trans(200., 0.);
        });

        use graphics::Text;

        let transform = c.transform.trans(0., 32.);
        Text::new_color([1.; 4], 24)
            .draw(
                &self.text,
                &mut self.glyph_cache,
                &c.draw_state,
                transform,
                g,
            )
            .unwrap();
    }
}
