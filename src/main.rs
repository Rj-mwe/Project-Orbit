use bevy::prelude::*;

const MOVE_SPEED: f32 = 300.0;
const DASH_SPEED: f32 = 1500.0;
const DASH_DURATION: f32 = 0.15;
const DASH_COOLDOWN: f32 = 1.0;
const PLAYER_RADIUS: f32 = 50.0;

#[derive(Component)] 
struct Player;

#[derive(Component)]
struct Dash {
    duration_timer: Timer,
    cooldown_timer: Timer,
    direction: Vec3,
    is_dashing: bool,
}

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
        Dash {
            duration_timer: Timer::from_seconds(DASH_DURATION, TimerMode::Once),
            cooldown_timer: {
                let mut timer = Timer::from_seconds(DASH_COOLDOWN, TimerMode::Once);
                timer.tick(std::time::Duration::from_secs_f32(DASH_COOLDOWN));
                timer
            },
            direction: Vec3::ZERO,
            is_dashing: false,
        },
        Mesh2d(meshes.add(Circle { radius: 50.0 })),
        MeshMaterial2d(materials.add(Color::from(bevy::color::palettes::css::WHITE))),
    ));
}

fn move_player(
    mut query: Query<(&mut Transform, &mut Dash, &MeshMaterial2d<ColorMaterial>), With<Player>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

    for (mut transform, mut dash) in query.iter_mut() {
        dash.cooldown_timer.tick(time.delta());

        if dash.is_dashing {
            dash.duration_timer.tick(time.delta());
            if dash.duration_timer.just_finished() {
                dash.is_dashing = false;
                dash.cooldown_timer.reset();
            } else {
                transform.translation += dash.direction * DASH_SPEED * time.delta_secs();
            }
        } else {
            let mut direction = Vec3::ZERO;

            if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
            if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
            if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
            if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }
            
            if direction.length_squared() > 0.0 {
                let normalized_dir = direction.normalize();
                transform.translation += MOVE_SPEED * normalized_dir * time.delta_secs();

                if keys.just_pressed(KeyCode::Space)
                    && dash.cooldown_timer.elapsed() >= dash.cooldown_timer.duration()
                {
                    dash.is_dashing = true;
                    dash.direction = normalized_dir;
                    dash.duration_timer.reset();
                }
            }
        }
   
        transform.translation.y = transform.translation.y.clamp(bottom_boundary, top_boundary);

        if transform.translation.x > wrap_threshold {
            transform.translation.x = -wrap_threshold;
        } else if transform.translation.x < -wrap_threshold {
            transform.translation.x = wrap_threshold;
        }
    }
}