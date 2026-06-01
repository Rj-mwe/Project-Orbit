use bevy::prelude::*;

const MOVE_SPEED: f32 = 300.0;

#[derive(Component)] 
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

fn setup(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    // 1. Spawn the modern 2D camera component
    commands.spawn(Camera2d::default());

    // 2. Spawn the player using modern required components instead of a bundle
    commands.spawn((
        Player, 
        Mesh2d(meshes.add(Circle { radius: 50.0 })),
        MeshMaterial2d(materials.add(Color::from(bevy::color::palettes::css::WHITE))),
    ));
}

fn move_player(
    mut transforms: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in transforms.iter_mut() {
        let mut direction = Vec3::ZERO;
        
        if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
        if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }
        
        if direction.length_squared() > 0.0 {
            transform.translation += MOVE_SPEED * direction.normalize() * time.delta_secs();
        }
    }
}