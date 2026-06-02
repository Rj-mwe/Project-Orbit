use bevy::prelude::*;
// rand is used via `rand::random()` below

const MOVE_SPEED: f32 = 300.0;
const DASH_SPEED: f32 = 1500.0;
const DASH_DURATION: f32 = 0.15;
const DASH_COOLDOWN: f32 = 1.0;
const PLAYER_RADIUS: f32 = 10.0;
const TRAIL_LIFESPAN: f32 = 0.25;

const SPAWN_INTERVAL: f32 = 2.0;
const COLLECTIBLE_SIZE: f32 = 20.0;
const MAX_COLLECTIBLES: usize = 8;

#[derive(Component)] 
struct Player;

#[derive(Component)]
struct Dash {
    duration_timer: Timer,
    cooldown_timer: Timer,
    direction: Vec3,
    last_direction: Vec3,
    is_dashing: bool,
}

#[derive(Component)]
struct TrailEffect {
    timer: Timer,
}

#[derive(Component)]
struct Collectible;

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource)]
struct Score {
    collected: u32,
}

#[derive(Component)]
struct ScoreText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SpawnTimer(Timer::from_seconds(SPAWN_INTERVAL, TimerMode::Repeating)))
        .insert_resource(Score { collected: 0 })
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, fade_trails, spawn_collectibles, collect_items, update_score_display))
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
            last_direction: Vec3::new(1.0, 0.0, 0.0),
            is_dashing: false,
        },
        Mesh2d(meshes.add(Circle { radius: PLAYER_RADIUS })),
        MeshMaterial2d(materials.add(Color::from(bevy::color::palettes::css::WHITE))),
    ));

    commands.spawn((
        Text2d::new("Collected: 0 | On Map: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_translation(Vec3::new(-400.0, 300.0, 0.0)),
        ScoreText,
    ));
}

fn move_player(
    mut query: Query<(&mut Transform, &mut Dash, &MeshMaterial2d<ColorMaterial>), With<Player>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
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

    for (mut transform, mut dash, material_handle) in query.iter_mut() {
        dash.cooldown_timer.tick(time.delta());

        if dash.is_dashing {
            dash.duration_timer.tick(time.delta());
            if dash.duration_timer.just_finished() {
                dash.is_dashing = false;
                dash.cooldown_timer.reset();
            } else {
                transform.translation += dash.direction * DASH_SPEED * time.delta_secs();
                commands.spawn((
                    TrailEffect {
                        timer: Timer::from_seconds(TRAIL_LIFESPAN, TimerMode::Once),
                    },
                    Mesh2d(meshes.add(Circle { radius: PLAYER_RADIUS * 0.8 })),
                    MeshMaterial2d(materials.add(Color::from(bevy::color::palettes::css::SKY_BLUE))),
                    Transform::from_translation(transform.translation),
                ));
            }
        } else {
            let mut direction = Vec3::ZERO;

            if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
            if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
            if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
            if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }
            
            if direction.length_squared() > 0.0 {
                let normalized_dir = direction.normalize();
                dash.last_direction = normalized_dir;
                transform.translation += MOVE_SPEED * normalized_dir * time.delta_secs();
            }
        }

        if (keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::ShiftLeft))
            && dash.cooldown_timer.elapsed() >= dash.cooldown_timer.duration()
            && !dash.is_dashing
        {
            dash.is_dashing = true;
            dash.direction = dash.last_direction;
            dash.duration_timer.reset();
        }

        if let Some(material) = materials.get_mut(material_handle) {
            if dash.is_dashing {
                material.color = Color::from(bevy::color::palettes::css::SKY_BLUE);
            } else if dash.cooldown_timer.elapsed() < dash.cooldown_timer.duration() {
                material.color = Color::from(bevy::color::palettes::css::RED);
            } else {
                material.color = Color::from(bevy::color::palettes::css::WHITE);
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

fn fade_trails(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut TrailEffect)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut trail) in query.iter_mut() {
        trail. timer.tick(time.delta());

        if trail.timer.just_finished() {
            commands.entity(entity).despawn();
        } else {
            let progress = trail.timer.fraction_remaining();
            transform.scale = Vec3::splat(progress);
        }
    }
}

fn spawn_collectibles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawn_timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    windows: Query<&Window>,
    collectible_query: Query<&Transform, With<Collectible>>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        if collectible_query.iter().count() >= MAX_COLLECTIBLES {
            return;
        }

        let window = match windows.single() {
            Ok(window) => window,
            Err(_) => return,
        };

        let existing_positions: Vec<Vec2> = collectible_query
            .iter()
            .map(|transform| transform.translation.truncate())
            .collect();

        let half_width = window.width() / 2.0 - COLLECTIBLE_SIZE;
        let half_height = window.height() / 2.0 - COLLECTIBLE_SIZE;
        let mut spawn_position = None;

        for _ in 0..20 {
            let x = rand::random::<f32>() * (half_width * 2.0) - half_width;
            let y = rand::random::<f32>() * (half_height * 2.0) - half_height;
            let candidate = Vec2::new(x, y);

            let overlaps = existing_positions.iter().any(|&pos| {
                pos.distance(candidate) < COLLECTIBLE_SIZE
            });

            if !overlaps {
                spawn_position = Some(candidate);
                break;
            }
        }

        if let Some(position) = spawn_position {
            commands.spawn((
                Collectible,
                Mesh2d(meshes.add(Circle { radius: COLLECTIBLE_SIZE / 2.0 })),
                MeshMaterial2d(materials.add(Color::from(bevy::color::palettes::css::GOLD))),
                Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
            ));
        }
    }
}

fn collect_items(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    collectible_query: Query<(Entity, &Transform), With<Collectible>>,
    mut score: ResMut<Score>,
) {
    let player_transform = match player_query.single() {
        Ok(transform) => transform,
        Err(_) => return,
    };

    for (entity, collectible_transform) in collectible_query.iter() {
        let distance = player_transform.translation.distance(collectible_transform.translation);
        if distance < PLAYER_RADIUS + COLLECTIBLE_SIZE / 2.0 {
            commands.entity(entity).despawn();
            score.collected += 1;
        }
    }
}

fn update_score_display(
    mut score_text_query: Query<&mut Text2d, With<ScoreText>>,
    score: Res<Score>,
    collectible_query: Query<&Transform, With<Collectible>>,
) {
    let on_map = collectible_query.iter().count();

    if let Ok(mut text) = score_text_query.single_mut() {
        text.0 = format!("Collected: {} | On Map: {}", score.collected, on_map);
    }
}
