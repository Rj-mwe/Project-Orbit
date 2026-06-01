use bevy::prelude::*;

const MOVE_SPEED: f32 = 300.0;
const PLAYER_RADIUS: f32 = 50.0;

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
    commands.spawn(Camera2d::default());
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
    windows: Query<&Window>,
) {
    let window = match windows.single() {
        Ok(window) => window,
        Err(_) => return,
    };
    let half_height = window.height() / 2.0;
    let top_boundary = half_height - PLAYER_RADIUS;
    let bottom_boundary = -half_height + PLAYER_RADIUS;

    let half_width = window.width() / 2.0;
    let wrap_threshold = half_width + PLAYER_RADIUS;

    for mut transform in transforms.iter_mut() {
        let mut direction = Vec3::ZERO;
        
        if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
        if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }
        
        if direction.length_squared() > 0.0 {
            transform.translation += MOVE_SPEED * direction.normalize() * time.delta_secs();
        }

        transform.translation.y = transform.translation.y.clamp(bottom_boundary, top_boundary);

        if transform.translation.x > wrap_threshold {
            transform.translation.x = -wrap_threshold;
        } else if transform.translation.x < -wrap_threshold {
            transform.translation.x = wrap_threshold;
        }
    }
}