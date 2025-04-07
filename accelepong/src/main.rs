use avian2d::prelude::*;
use bevy::{color::palettes::basic::RED, prelude::*, render::camera::ScalingMode};
use iyes_perf_ui::prelude::*;

const BALL_RADIUS: f32 = 15.0;

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

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        // Componentes visuais
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(RED))),
        // Componentes para a física
        RigidBody::Dynamic,
        Collider::circle(BALL_RADIUS),
        LinearVelocity(Vec2 { x: 500.0, y: 0.0 }),
        // Componentes personalizados
        Ball,
    ));
}

// Função principal que configura e inicia o jogo
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
        .add_systems(Startup, (setup_debug, setup_camera, spawn_ball))
        .run(); // Inicia o loop principal do jogo
}
