use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use rand::{thread_rng, Rng};

const SPRITES_PER_SECOND: u32 = 10000;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "AtlasStress".to_string(),
            width: 800.,
            height: 600.,
            present_mode: PresentMode::Immediate,
            resizable: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .insert_resource(Counter(0))
        .add_system(counter_system)
        .add_system(mouse_handler)
        .add_system(animate_sprite)
        .run();
}

#[derive(Component, Deref)]
struct AtlasHandle(Handle<TextureAtlas>);

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct StatsText;

#[derive(Deref, DerefMut)]
struct Counter(usize);

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    atlas_handle: Res<AtlasHandle>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite)>,
) {
    let atlas = texture_atlases.get(&atlas_handle).unwrap();
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = (sprite.index + 1) % atlas.len();
        }
    }
}

fn mouse_handler(
    mut commands: Commands,
    time: Res<Time>,
    mouse_button_input: Res<Input<MouseButton>>,
    atlas_handle: Res<AtlasHandle>,
    mut counter: ResMut<Counter>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let spawn_count = (SPRITES_PER_SECOND as f64 * time.delta_seconds_f64()) as usize;
        spawn_sprites(
            &mut commands,
            &mut counter,
            spawn_count,
            atlas_handle,
            texture_atlases,
        );
    }
}

fn spawn_sprites(
    commands: &mut Commands,
    counter: &mut Counter,
    spawn_count: usize,
    atlas_handle: Res<AtlasHandle>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) {
    let mut rng = thread_rng();
    **counter += spawn_count;

    let atlas = texture_atlases.get(&atlas_handle).unwrap();

    for _ in 0..spawn_count {
        // draw a sprite from the atlas
        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform {
                    translation: Vec3::new(
                        rng.gen_range(-300.0..300.0),
                        rng.gen_range(-300.0..300.0),
                        0.0,
                    ),
                    scale: Vec3::splat(0.1),
                    ..default()
                },
                sprite: TextureAtlasSprite::new(rng.gen_range(0..atlas.len())),
                texture_atlas: atlas_handle.0.clone_weak(),
                ..default()
            })
            .insert(AnimationTimer(Timer::from_seconds(0.2, true)));
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/array_texture.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(250.0, 250.0), 1, 4);
    let atlas_handle = texture_atlases.add(texture_atlas);

    // set up a scene to display our texture atlas
    commands.spawn_bundle(Camera2dBundle::default());

    commands.insert_resource(AtlasHandle(atlas_handle));

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Sprite Count: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.0, 1.0, 0.0),
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.0, 1.0, 1.0),
                        },
                    },
                    TextSection {
                        value: "\nAverage FPS: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.0, 1.0, 0.0),
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.0, 1.0, 1.0),
                        },
                    },
                ],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(StatsText);
}

fn counter_system(
    diagnostics: Res<Diagnostics>,
    counter: Res<Counter>,
    mut query: Query<&mut Text, With<StatsText>>,
) {
    let mut text = query.single_mut();

    if counter.is_changed() {
        text.sections[1].value = format!("{}", **counter);
    }

    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            text.sections[3].value = format!("{:.2}", average);
        }
    };
}
