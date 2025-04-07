use avian2d::prelude::*;
use bevy::{color::palettes::basic::RED, prelude::*, render::camera::ScalingMode};
use iyes_perf_ui::prelude::*;

const BALL_RADIUS: f32 = 15.0;

#[derive(Resource, Default)]
struct BallSpeed {
    speed: f32,
}

#[derive(Component)]
struct Camera;

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

fn generate_random_number(start: i32, end: i32) -> i32 {
    use rand::Rng;

    let mut rng = rand::rng();
    rng.random_range(start..end)
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
        LinearVelocity(velocity_from_angle(
            generate_random_number(0, 360) as f32,
            ball_speed.speed,
        )), // Send the ball to a random angle with the defined speed
        // Custom components
        Ball,
    ));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    position: WindowPosition::Centered(MonitorSelection::Index(1)),
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
        .insert_resource(Gravity(Vec2::ZERO))
        .insert_resource(BallSpeed { speed: 500.0 })
        .add_systems(Startup, (setup_debug, setup_camera, spawn_ball))
        .run();
}
