use ggez::{Context, GameResult};
use glam::*;
use std::{env, path};

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("../assets");
        path
    } else {
        path::PathBuf::from("../assets")
    };

    use ggez::conf::{WindowMode, WindowSetup};

    let context_builder = ggez::ContextBuilder::new("appname", "yourname")
        .window_setup(WindowSetup {
            title: "ggez tutorial".to_owned(),
            ..Default::default()
        })
        .window_mode(WindowMode {
            width: 480.0,
            height: 320.0,
            ..Default::default()
        })
        .add_resource_path(resource_dir);
    let (mut ggez_context, event_queue) = context_builder.build()?;

    let my_app = MyApp::new(&mut ggez_context)?;
    ggez::event::run(ggez_context, event_queue, my_app)
}

struct MyApp {}

impl MyApp {
    fn new(ctx: &mut Context) -> GameResult<MyApp> {
        use ggez::graphics::FontData;

        ctx.gfx.add_font(
            "LiberationMono",
            FontData::from_path(ctx, "/LiberationMono-Regular.ttf")?,
        );
        let my_app = MyApp {};
        Ok(my_app)
    }
}

use ggez::{event::EventHandler, GameError};

impl EventHandler<GameError> for MyApp {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Drawable, Mesh, Text};

        let mut canvas = Canvas::from_frame(ctx, Color::from([0.2, 0.2, 0.2, 1.0]));

        let colors = [Color::RED, Color::GREEN, Color::BLUE];
        [100.0, 300.0, 500.0]
            .iter()
            .zip(colors.iter())
            .for_each(|(x, c)| {
                let circle =
                    Mesh::new_circle(ctx, DrawMode::fill(), Vec2::new(0., 0.), 100.0, 1.0, *c)
                        .unwrap();
                circle.draw(&mut canvas, Vec2::new(*x, 100.0));

                // circle.draw(
                //     &mut canvas,
                //     DrawParam::new()
                //         .dest(Vec2 { x: 0.0, y: 0.0 })
                //         .rotation(std::f32::consts::PI / 6.0)
                //         .scale(Vec2 { x: 1.0, y: 1.0 })
                //         .offset(Vec2 { x: -*x, y: -100.0 })
                // );

                // let mat = Mat4::from_translation(Vec3 {
                //     x: *x,
                //     y: 100.0,
                //     z: 0.0,
                // });
                // circle.draw(&mut canvas, DrawParam::new().transform(mat));
            });

        let fps = format!("FPS: {:.2}", ctx.time.fps());
        let scales = [48.0, 32.0, 24.0, 16.0];
        [0.0, 48.0, 80.0, 104.0]
            .iter()
            .zip(scales.iter())
            .for_each(|(y, s)| {
                let mut text = Text::new(&fps);
                text.set_font("LiberationMono")
                    .set_scale(*s)
                    .draw(&mut canvas, Vec2::new(0.0, *y));
            });

        // let fps = format!("FPS: {:.2}", ctx.time.fps());
        // let mut text = Text::new("");
        // [48.0, 32.0, 24.0, 16.0].iter().for_each(|s| {
        //     let fragment = TextFragment::new(format!("{}\n", fps))
        //         .font("LiberationMono")
        //         .scale(*s);
        //     text.add(fragment);
        // });
        // text.draw(&mut canvas, Vec2::new(0.0, 0.0));

        canvas.finish(ctx)?;
        Ok(())
    }
}
