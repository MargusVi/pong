use avian2d::prelude::*;
use bevy::{color::palettes::basic::RED, prelude::*, render::camera::ScalingMode};

const BALL_RADIUS: f32 = 15.0;

#[derive(Component)]
struct Camera;

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
    ));
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(RED))),
        Camera,
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
        ))
        .add_systems(Startup, (setup_camera, spawn_ball))
        .run(); // Inicia o loop principal do jogo
}
