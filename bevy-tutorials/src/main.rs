use bevy::prelude::*;
use ops::FloatPow;
use std::time::Duration;

fn main() {
    use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
    use bevy::window::{EnabledButtons, PresentMode};

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy tutorial".into(),
                        resolution: (480., 320.).into(),
                        present_mode: PresentMode::AutoVsync,
                        resizable: false,
                        enabled_buttons: EnabledButtons {
                            minimize: false,
                            maximize: false,
                            ..Default::default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "..\\assets".into(),
                    ..default()
                }),
            // LogDiagnosticsPlugin::default(),
            // FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_plugins(MyPlugin)
        .run();
}

pub struct MyPlugin;

#[derive(Resource)]
struct MyChains(u32);

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_system)
            .add_systems(PreUpdate, (window_close_system, user_input_system).chain())
            .add_systems(
                Update,
                (
                    my_explosion_system,
                    my_breakable_system,
                    my_chain_explosion_system,
                    my_chains_display_system,
                )
                    .chain(),
            )
            .insert_resource(ClearColor(Color::srgba(0.1, 0.1, 0.1, 1.)))
            .insert_resource(MyChains(0));
    }
}

#[derive(Component)]
struct ChainsDisplay;

fn setup_system(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2d);

    let font = asset_server.load("LiberationMono-Regular.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 16.,
        ..default()
    };

    use bevy::sprite::Anchor;

    cmd.spawn((
        Text2d::new("Mouse L: Spawn Explosion"),
        text_font.clone(),
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0., 160., 0.)),
        Anchor::TopCenter,
    ));
    cmd.spawn((
        Text2d::new("Mouse M: Spawn Bomb"),
        text_font.clone(),
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0., 140., 0.)),
        Anchor::TopCenter,
    ));
    cmd.spawn((
        Text2d::new("Mouse R: Pause/Resume"),
        text_font.clone(),
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0., 120., 0.)),
        Anchor::TopCenter,
    ));
    cmd.spawn((
        Text2d::new(""),
        text_font.clone(),
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0., -140., 0.)),
        Anchor::TopCenter,
        ChainsDisplay,
    ));
}

fn window_close_system(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        once!({
            info!("Escape!");
            app_exit.send(AppExit::Success);
        });
    }
}

fn user_input_system(
    camera_props: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cmd: Commands,
    mut time: ResMut<Time<Virtual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (camera, camera_transform) = *camera_props;
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };
    if mouse_button.just_pressed(MouseButton::Left) {
        let entity = cmd
            .spawn(MyExplosion::new(
                point,
                0,
                (&time, &mut meshes, &mut materials),
            ))
            .id();
        println!("spawn {entity}");
    }
    if mouse_button.just_pressed(MouseButton::Middle) {
        let entity = cmd
            .spawn(MyBomb::new(point, (&mut meshes, &mut materials)))
            .id();
        println!("spawn {entity}");
    }
    if mouse_button.just_pressed(MouseButton::Right) {
        if time.is_paused() {
            time.unpause();
            println!("resume");
        } else {
            time.pause();
            println!("pause");
        }
    }
}

#[derive(Component)]
struct MyExplosion {
    timer: Timer,
    radius: f32,
    chain_value: u32,
}

const EXPLOSION_RADIUS: f32 = 40.;

impl MyExplosion {
    fn new(
        point: Vec2,
        chain_value: u32,
        components: (
            &Time<Virtual>,
            &mut ResMut<Assets<Mesh>>,
            &mut ResMut<Assets<ColorMaterial>>,
        ),
    ) -> (
        MyExplosion,
        Transform,
        Mesh2d,
        MeshMaterial2d<ColorMaterial>,
    ) {
        let (time, meshes, materials) = components;
        (
            MyExplosion {
                timer: Timer::from_seconds(1.2, TimerMode::Once),
                radius: 0.,
                chain_value,
            },
            Transform::from_translation(point.extend(1.)).with_scale(Vec2::splat(0.).extend(1.)),
            Mesh2d(meshes.add(Circle::new(EXPLOSION_RADIUS))),
            MeshMaterial2d(materials.add(Self::color(&time))),
        )
    }

    fn color(time: &Time<Virtual>) -> Color {
        let hue = time.elapsed().as_secs_f32().fract();
        Color::hsl(360. * hue, 0.9, 0.9)
    }
}

fn my_explosion_system(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut MyExplosion, &mut Transform)>,
    time: Res<Time<Virtual>>,
    mut chains: ResMut<MyChains>,
) {
    for (entity, mut explosion, mut transform) in &mut query {
        if explosion.timer.tick(time.delta()).just_finished() {
            // 爆発おわり
            println!("despawn {entity}");
            _ = cmd.entity(entity).despawn();
        } else {
            // 爆発している
            let timer = explosion.timer.clone();
            let t = 2.
                * Duration::min(timer.elapsed(), timer.remaining())
                    .div_duration_f32(timer.duration());
            explosion.radius = t;
            *transform = transform.with_scale(Vec2::splat(t).extend(1.));
        }
    }

    let chain_value_max = query
        .iter()
        .map(|(_, explosion, _)| explosion.chain_value)
        .reduce(|chain_value1, chain_value2| u32::max(chain_value1, chain_value2));
    chains.0 = chain_value_max.unwrap_or_default();
}

#[derive(Default)]
enum MyBreakableEvent {
    #[default]
    None,
    Damaged(u32),
}

#[derive(Component)]
struct MyBreakable {
    will_explode: bool,
    incoming: MyBreakableEvent,
}

impl Default for MyBreakable {
    fn default() -> MyBreakable {
        Self {
            will_explode: true,
            incoming: MyBreakableEvent::None,
        }
    }
}

#[derive(Component)]
#[require(MyBreakable)]
struct MyBomb;

const BOMB_RADIUS: f32 = 4.;
const BOMB_COLOR: Color = Color::WHITE;

impl MyBomb {
    fn new(
        point: Vec2,
        components: (
            &mut ResMut<Assets<Mesh>>,
            &mut ResMut<Assets<ColorMaterial>>,
        ),
    ) -> (MyBomb, Transform, Mesh2d, MeshMaterial2d<ColorMaterial>) {
        let (meshes, materials) = components;
        (
            MyBomb,
            Transform::from_translation(point.extend(1.)),
            Mesh2d(meshes.add(Rectangle::new(BOMB_RADIUS, BOMB_RADIUS))),
            MeshMaterial2d(materials.add(BOMB_COLOR)),
        )
    }
}

fn my_breakable_system(
    mut cmd: Commands,
    query: Query<(Entity, &MyBreakable, &Transform)>,
    time: Res<Time<Virtual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, breakable, transform) in &query {
        match breakable.incoming {
            MyBreakableEvent::Damaged(chain_value) => {
                cmd.entity(entity).despawn();
                if breakable.will_explode {
                    // 誘爆する
                    let entity = cmd
                        .spawn(MyExplosion::new(
                            transform.translation.xy(),
                            chain_value + 1,
                            (&time, &mut meshes, &mut materials),
                        ))
                        .id();
                    println!("spawn {entity}");
                }
            }
            _ => {}
        }
    }
}

fn my_chain_explosion_system(
    mut query_breakables: Query<(&mut MyBreakable, &Transform)>,
    query_explosions: Query<(&MyExplosion, &Transform)>,
) {
    for (mut breakable, transform) in &mut query_breakables {
        let point = transform.translation.truncate();
        let collided_opponent = {
            let mut opponent = None;
            for (explosion, explosion_transform) in &query_explosions {
                let point2 = explosion_transform.translation.truncate();
                if get_collision(
                    (&point, explosion.radius * EXPLOSION_RADIUS),
                    (&point2, BOMB_RADIUS),
                ) {
                    opponent = Some(explosion.chain_value);
                    break;
                }
            }
            opponent
        };
        if let Some(chain_value) = collided_opponent {
            breakable.incoming = MyBreakableEvent::Damaged(chain_value);
        }
    }
}

fn get_collision(c1: (&Vec2, f32), c2: (&Vec2, f32)) -> bool {
    let distance_squared = c1.0.distance_squared(*c2.0);
    distance_squared < (c1.1 + c2.1).squared()
}

fn my_chains_display_system(
    mut text2d: Single<&mut Text2d, With<ChainsDisplay>>,
    chains: Res<MyChains>,
) {
    let nbchains = if chains.0 > 0 {
        format!("{} Chain(s)", chains.0)
    } else {
        "".into()
    };
    text2d.0 = nbchains;
}
