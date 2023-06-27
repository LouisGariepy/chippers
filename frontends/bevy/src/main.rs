use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::WindowMode};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::hex("58505D").unwrap()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chippers (CHIP-8)".into(),
                mode: WindowMode::Fullscreen,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let screen_width = 1920.;
    commands.spawn(Camera2dBundle::default());
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::new(screen_width, screen_width / 2., 0.)),
        material: materials.add(ColorMaterial::from(Color::DARK_GREEN)),
        ..default()
    });
}
