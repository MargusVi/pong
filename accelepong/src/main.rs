use avian2d::prelude::*;
use bevy::{
    color::palettes::basic::RED, prelude::*, render::camera::ScalingMode, window::WindowMode,
};
use iyes_perf_ui::prelude::*;

const BALL_RADIUS: f32 = 15.0;
const WALL_THICKNESS: f32 = 100.0;

#[derive(Resource, Default)]
struct BallSpeed {
    speed: f32,
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

fn get_initial_ball_movement(speed: f32) -> Vec2 {
    use rand::Rng;

    // We want to avoid angles in ranges 330-30 and 150-210
    // So we'll use the ranges 30-150 and 210-330
    let mut rng = rand::rng();
    let range_selector = rng.random_range(0..2);
    println!("Range selector: {}", range_selector);

    // Choose from the valid ranges
    if range_selector == 0 {
        // Range 30-150
        velocity_from_angle(rng.random_range(30.0..150.0), speed)
    } else {
        // Range 210-330
        velocity_from_angle(rng.random_range(210.0..330.0), speed)
    }
}

fn velocity_from_angle(angle_degrees: f32, speed: f32) -> Vec2 {
    // Convert angle from degrees to radians
    let angle_radians = angle_degrees.to_radians();

    // Calculate direction vector components
    let x = speed * angle_radians.cos();
    let y = speed * angle_radians.sin();

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

    // Top wall
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
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ball_speed: ResMut<BallSpeed>,
) {
    commands.spawn((
        // Visual components
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(RED))),
        // Physics components
        RigidBody::Dynamic,
        Collider::circle(BALL_RADIUS),
        LinearVelocity(get_initial_ball_movement(ball_speed.speed)), // Send the ball to a random angle with the defined speed
        // Custom components
        Ball,
    ));
}

// TODO: está perdendo velocidade, isso não pode ocorrer
fn collision_system(
    mut collision_events: EventReader<Collision>,
    mut ball_query: Query<&mut LinearVelocity, With<Ball>>,
    wall_query: Query<(), With<Wall>>,
) {
    for Collision(contacts) in collision_events.read() {
        // Verifica se a colisão envolve a bola e uma parede
        let (ball_entity, _wall_entity, manifolds) = if ball_query.get(contacts.entity1).is_ok()
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

        // Itera sobre os manifolds de contato
        for manifold in manifolds {
            // Obtém o vetor normal do contato no espaço global
            let normal = manifold.global_normal2(&Rotation::default());

            // Atualiza a velocidade da bola refletindo-a em relação ao vetor normal
            if let Ok(mut velocity) = ball_query.get_mut(ball_entity) {
                let velocity_vector = Vec2::new(velocity.x, velocity.y);
                let reflected_velocity =
                    velocity_vector - 2.0 * velocity_vector.dot(normal) * normal;
                velocity.x = reflected_velocity.x;
                velocity.y = reflected_velocity.y;
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
                    mode: WindowMode::Fullscreen(MonitorSelection::Index(1)),
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
        .insert_resource(BallSpeed { speed: 500.0 })
        .add_systems(
            Startup,
            (setup_debug, setup_camera, spawn_play_field, spawn_ball),
        )
        .add_systems(PostUpdate, collision_system)
        .run();
}
