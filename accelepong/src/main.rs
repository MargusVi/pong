use avian2d::prelude::*;
use bevy::{
    color::palettes::basic::RED, prelude::*, render::camera::ScalingMode, window::WindowMode,
};
use iyes_perf_ui::prelude::*;

const BALL_RADIUS: f32 = 15.0;
const WALL_THICKNESS: f32 = 100.0;

#[derive(Component)]
struct BallMovement {
    angle: f32,
    speed: f32,
    speed_increment: f32,
}

#[derive(Component)]
struct Camera;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Ball;

fn setup_debug(mut commands: Commands) {
    commands.spawn(PerfUiDefaultEntries::default());
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1920.0,
                min_height: 1080.0,
            },
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
        Camera,
    ));
}

fn get_random_ball_start_angle() -> f32 {
    use rand::Rng;

    // We want to avoid angles in ranges 330-30 and 150-210
    // So we'll use the ranges 30-150 and 210-330
    let mut rng = rand::rng();
    let range_selector = rng.random_range(0..2);

    // Choose from the valid ranges
    if range_selector == 0 {
        // Range 30-150
        rng.random_range(30.0..150.0)
    } else {
        // Range 210-330
        rng.random_range(210.0..330.0)
    }
}

fn velocity_from_angle(angle_degrees: f32, speed: f32) -> Vec2 {
    // Convert angle from degrees to radians
    let angle_radians = angle_degrees.to_radians();

    // Calculate direction vector components
    let x = speed * angle_radians.cos();
    let y = speed * angle_radians.sin();

    println!("Velocity: {{{}, {}}}", x, y);

    // Return the velocity vector
    Vec2::new(x, y)
}

fn spawn_play_field(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    let window = window.get_single().unwrap();
    let window_width = window.resolution.width();
    let window_height = window.resolution.height();

    // Top wall
    commands.spawn((
        Position::from_xy(0.0, (window_height - WALL_THICKNESS) / 2.0),
        Mesh2d(meshes.add(Rectangle::from_size(Vec2::new(
            window_width,
            WALL_THICKNESS,
        )))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 0.))),
        RigidBody::Static,
        Collider::rectangle(window_width, WALL_THICKNESS),
        Wall,
    ));

    // Bottom wall
    commands.spawn((
        Position::from_xy(0.0, -((window_height - WALL_THICKNESS) / 2.0)),
        Mesh2d(meshes.add(Rectangle::from_size(Vec2::new(
            window_width,
            WALL_THICKNESS,
        )))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 0.))),
        RigidBody::Static,
        Collider::rectangle(window_width, WALL_THICKNESS),
        Wall,
    ));

    // Left wall - positioned just outside the visible area
    commands.spawn((
        Position::from_xy(-(window_width / 2.0 + WALL_THICKNESS / 2.0), 0.0),
        Mesh2d(meshes.add(Rectangle::from_size(Vec2::new(
            WALL_THICKNESS,
            window_height,
        )))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 0.))),
        RigidBody::Static,
        Collider::rectangle(WALL_THICKNESS, window_height),
        Wall,
    ));

    // Right wall - positioned just outside the visible area
    commands.spawn((
        Position::from_xy(window_width / 2.0 + WALL_THICKNESS / 2.0, 0.0),
        Mesh2d(meshes.add(Rectangle::from_size(Vec2::new(
            WALL_THICKNESS,
            window_height,
        )))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 0.))),
        RigidBody::Static,
        Collider::rectangle(WALL_THICKNESS, window_height),
        Wall,
    ));
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let initial_angle = get_random_ball_start_angle();
    let speed = 500.0;
    let speed_increment = 10.0;
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(RED))),
        RigidBody::Dynamic,
        Collider::circle(BALL_RADIUS),
        Restitution::PERFECTLY_ELASTIC,
        Friction::ZERO,
        LinearDamping(0.0),
        Mass::ZERO,
        // Inicializa o vetor de velocidade
        LinearVelocity(velocity_from_angle(initial_angle, speed)),
        // Agora adiciona o componente que armazena o estado do movimento
        BallMovement {
            angle: initial_angle,
            speed,
            speed_increment,
        },
        Ball,
    ));
}

fn collision_system(
    mut collision_events: EventReader<Collision>,
    mut ball_query: Query<(&mut LinearVelocity, &mut BallMovement), With<Ball>>,
    wall_query: Query<(), With<Wall>>,
) {
    for Collision(contacts) in collision_events.read() {
        if contacts.collision_started() {
            let (ball_entity, _wall_entity, _manifolds) =
                if ball_query.get(contacts.entity1).is_ok()
                    && wall_query.get(contacts.entity2).is_ok()
                {
                    (contacts.entity1, contacts.entity2, &contacts.manifolds)
                } else if ball_query.get(contacts.entity2).is_ok()
                    && wall_query.get(contacts.entity1).is_ok()
                {
                    (contacts.entity2, contacts.entity1, &contacts.manifolds)
                } else {
                    continue;
                };

            if let Some(first_manifold) = contacts.manifolds.first() {
                let normal = first_manifold.global_normal2(&Rotation::default());

                if let Ok((mut velocity, mut ball_movement)) = ball_query.get_mut(ball_entity) {
                    let old_velocity = Vec2::new(velocity.x, velocity.y);
                    let dot = old_velocity.dot(normal);
                    let mut new_velocity = old_velocity - 2.0 * dot * normal;

                    // Incrementa a velocidade
                    ball_movement.speed += ball_movement.speed_increment;
                    ball_movement.speed_increment += ball_movement.speed_increment * 10.0 / 100.0;

                    // Normaliza o vetor de velocidade para manter a direção e aplica a nova magnitude
                    new_velocity = new_velocity.normalize() * ball_movement.speed;

                    velocity.x = new_velocity.x;
                    velocity.y = new_velocity.y;

                    ball_movement.angle = new_velocity.y.atan2(new_velocity.x).to_degrees();

                    println!(
                        "Reflexão: velocidade alterada de {:?} para {:?}, nova velocidade: {}",
                        old_velocity, new_velocity, ball_movement.speed
                    );
                }
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    position: WindowPosition::Automatic,
                    mode: WindowMode::Fullscreen(MonitorSelection::Index(1)), // TODO: verificar como separar o espaço virtual da resolução da tela para que mudanças de resolução não afetem o jogo.
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
            PerfUiPlugin,
        ))
        .insert_resource(Gravity::ZERO)
        .add_systems(
            Startup,
            (setup_debug, setup_camera, spawn_play_field, spawn_ball),
        )
        .add_systems(PostUpdate, collision_system)
        .run();
}
