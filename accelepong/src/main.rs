// TODO: refazer sistema de colisão, o atual buga a velocidades muito altas da bola pois a bola fica presa dentro da raquete e quando sai a direção da colisão ativa pode ser a oposta da que deveria, fazendo com que a checagem ocorra do lado errado e a bola atravesse as raquetes infinitamente.

use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};

// Constantes para velocidades e tamanhos dos elementos do jogo
const BALL_SIZE: f32 = 5.; // Tamanho da bola
const PADDLE_SPEED: f32 = 10.; // Velocidade das raquetes
const PADDLE_WIDTH: f32 = 10.; // Largura das raquetes
const PADDLE_HEIGHT: f32 = 50.; // Altura das raquetes
const GUTTER_HEIGHT: f32 = 96.; // Altura das barreiras superior e inferior

// Recurso para armazenar a velocidade da bola que pode ser alterada durante o jogo
#[derive(Resource)]
struct BallSpeed(f32);

// Implementação padrão para BallSpeed
impl Default for BallSpeed {
    fn default() -> Self {
        BallSpeed(5.0) // Velocidade inicial da bola
    }
}

// Componente para exibir a pontuação do jogador
#[derive(Component)]
struct PlayerScore;

// Componente para exibir a pontuação da IA
#[derive(Component)]
struct AiScore;

// Recurso para armazenar a pontuação atual do jogo
#[derive(Resource, Default)]
struct Score {
    player: u32, // Pontuação do jogador
    ai: u32,     // Pontuação da IA
}

// Enum para identificar quem marcou ponto
enum Scorer {
    Ai,     // IA marcou ponto
    Player, // Jogador marcou ponto
}

// Evento disparado quando alguém marca ponto
#[derive(Event)]
struct Scored(Scorer);

// Componente para representar a bola no jogo
#[derive(Component)]
#[require(
      Position,
      Velocity(|| Velocity(Vec2::new(-1., 1.))),     // Velocidade inicial da bola
      Shape(|| Shape(Vec2::new(BALL_SIZE, BALL_SIZE))), // Tamanho da bola
  )]
struct Ball;

// Componente para representar as raquetes
#[derive(Component)]
#[require(
      Position,
      Shape(|| Shape(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT))),
      Velocity
  )]
struct Paddle;

// Componente para as barreiras superior e inferior
#[derive(Component)]
#[require(Position, Shape)]
struct Gutter;

// Componente para armazenar a posição de uma entidade
#[derive(Component, Default)]
struct Position(Vec2);

// Componente para armazenar a velocidade de uma entidade
#[derive(Component, Default)]
struct Velocity(Vec2);

// Componente para armazenar o tamanho/forma de uma entidade
#[derive(Component, Default)]
struct Shape(Vec2);

// Componente para identificar a raquete controlada pelo jogador
#[derive(Component)]
struct Player;

// Componente para identificar a raquete controlada pela IA
#[derive(Component)]
struct Ai;

// Enum para identificar o tipo de colisão
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,   // Colisão no lado esquerdo
    Right,  // Colisão no lado direito
    Top,    // Colisão no topo
    Bottom, // Colisão na parte inferior
}

// Sistema para mover a raquete da IA seguindo a posição da bola
fn move_ai(
    mut ai: Query<(&mut Velocity, &Position), With<Ai>>,
    ball: Query<&Position, With<Ball>>,
) {
    if let Ok((mut velocity, position)) = ai.get_single_mut() {
        if let Ok(ball_position) = ball.get_single() {
            let a_to_b = ball_position.0 - position.0;
            velocity.0.y = a_to_b.y.signum(); // Move na direção da bola
        }
    }
}

// Sistema para atualizar visualmente o placar
fn update_scoreboard(
    mut player_score: Query<&mut Text, With<PlayerScore>>,
    mut ai_score: Query<&mut Text, (With<AiScore>, Without<PlayerScore>)>,
    score: Res<Score>,
) {
    if score.is_changed() {
        // Só atualiza quando a pontuação mudar
        if let Ok(mut player_score) = player_score.get_single_mut() {
            player_score.0 = score.player.to_string(); // Atualiza texto do jogador
        }

        if let Ok(mut ai_score) = ai_score.get_single_mut() {
            ai_score.0 = score.ai.to_string(); // Atualiza texto da IA
        }
    }
}

// Sistema para criar o placar visual
fn spawn_scoreboard(mut commands: Commands) {
    // Cria texto para pontuação do jogador
    commands.spawn((
        PlayerScore,
        Text::new("0"),
        TextFont {
            font_size: 72.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        },
    ));

    // Cria texto para pontuação da IA
    commands.spawn((
        AiScore,
        Text::new("0"),
        TextFont {
            font_size: 72.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));
}

// Sistema para atualizar o recurso de pontuação
fn update_score(mut score: ResMut<Score>, mut events: EventReader<Scored>) {
    for event in events.read() {
        match event.0 {
            Scorer::Ai => score.ai += 1,         // Incrementa pontuação da IA
            Scorer::Player => score.player += 1, // Incrementa pontuação do jogador
        }
    }
}

// Sistema para detectar quando alguém marca ponto
fn detect_scoring(
    mut ball: Query<&mut Position, With<Ball>>,
    window: Query<&Window>,
    mut events: EventWriter<Scored>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();

        if let Ok(ball) = ball.get_single_mut() {
            // Se a bola saiu pela direita, IA marcou ponto
            if ball.0.x > window_width / 2. {
                events.send(Scored(Scorer::Ai));
            // Se a bola saiu pela esquerda, jogador marcou ponto
            } else if ball.0.x < -window_width / 2. {
                events.send(Scored(Scorer::Player));
            }
        }
    }
}

// Sistema para resetar a posição da bola após alguém marcar ponto
fn reset_ball(
    mut ball: Query<(&mut Position, &mut Velocity), With<Ball>>,
    mut events: EventReader<Scored>,
) {
    for event in events.read() {
        if let Ok((mut position, mut velocity)) = ball.get_single_mut() {
            match event.0 {
                Scorer::Ai => {
                    position.0 = Vec2::new(0., 0.); // Centro da tela
                    velocity.0 = Vec2::new(-1., 1.); // Direção para a esquerda
                }
                Scorer::Player => {
                    position.0 = Vec2::new(0., 0.); // Centro da tela
                    velocity.0 = Vec2::new(1., 1.); // Direção para a direita
                }
            }
        }
    }
}

// Sistema para processar entrada do teclado do jogador
fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle: Query<&mut Velocity, With<Player>>,
) {
    if let Ok(mut velocity) = paddle.get_single_mut() {
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            velocity.0.y = 1.; // Move para cima
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            velocity.0.y = -1.; // Move para baixo
        } else {
            velocity.0.y = 0.; // Para o movimento
        }
    }
}

// Sistema para criar as barreiras superior e inferior
fn spawn_gutters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let window_height = window.resolution.height();

        // Calcula a posição das barreiras
        let top_gutter_y = window_height / 2. - GUTTER_HEIGHT / 2.;
        let bottom_gutter_y = -window_height / 2. + GUTTER_HEIGHT / 2.;

        let shape = Rectangle::from_size(Vec2::new(window_width, GUTTER_HEIGHT));
        let color = Color::srgb(0., 0., 0.); // Cor preta

        // Podemos compartilhar as meshes entre as barreiras clonando-as
        let mesh_handle = meshes.add(shape);
        let material_handle = materials.add(color);

        // Cria a barreira superior
        commands.spawn((
            Gutter,
            Shape(shape.size()),
            Position(Vec2::new(0., top_gutter_y)),
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(material_handle.clone()),
        ));

        // Cria a barreira inferior
        commands.spawn((
            Gutter,
            Shape(shape.size()),
            Position(Vec2::new(0., bottom_gutter_y)),
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(material_handle.clone()),
        ));
    }
}

// Sistema para atualizar a posição visual com base na posição lógica
fn project_positions(mut positionables: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.); // Converte Vec2 para Vec3 (z=0)
    }
}

// Sistema para mover a bola
fn move_ball(mut ball: Query<(&mut Position, &Velocity), With<Ball>>, ball_speed: Res<BallSpeed>) {
    if let Ok((mut position, velocity)) = ball.get_single_mut() {
        position.0 += velocity.0 * ball_speed.0; // Atualiza posição com base na velocidade
    }
}

// Sistema para mover as raquetes
fn move_paddles(
    mut paddle: Query<(&mut Position, &Velocity), With<Paddle>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();

        for (mut position, velocity) in &mut paddle {
            let new_position = position.0 + velocity.0 * PADDLE_SPEED;
            // Verifica se a raquete não ultrapassará os limites da tela
            if new_position.y.abs() < window_height / 2. - GUTTER_HEIGHT - PADDLE_HEIGHT / 2. {
                position.0 = new_position;
            }
        }
    }
}

// Função auxiliar para detectar colisões entre a bola e outros objetos
fn collide_with_side(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None; // Sem colisão
    }

    let closest = wall.closest_point(ball.center());
    let offset = ball.center() - closest;

    // Determina o lado da colisão com base no maior deslocamento
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

// Sistema para tratar colisões da bola
fn handle_collisions(
    mut ball: Query<(&mut Velocity, &Position, &Shape), With<Ball>>,
    other_things: Query<(&Position, &Shape), Without<Ball>>,
    mut ball_speed: ResMut<BallSpeed>,
) {
    if let Ok((mut ball_velocity, ball_position, ball_shape)) = ball.get_single_mut() {
        for (position, shape) in &other_things {
            let circle = Circle {
                radius: ball_shape.0.x,
            };
            // Verifica colisão entre a bola e o objeto
            if let Some(collision) = collide_with_side(
                BoundingCircle::new(ball_position.0, circle.radius),
                Aabb2d::new(position.0, shape.0 / 2.0),
            ) {
                // Inverte a direção da bola baseado no tipo de colisão
                match collision {
                    Collision::Left => {
                        ball_velocity.0.x *= -1.; // Inverte direção horizontal
                        ball_speed.0 += 0.1; // Aumenta velocidade da bola
                    }
                    Collision::Right => {
                        ball_velocity.0.x *= -1.; // Inverte direção horizontal
                        ball_speed.0 += 0.1; // Aumenta velocidade da bola
                    }
                    Collision::Top => {
                        ball_velocity.0.y *= -1.; // Inverte direção vertical
                    }
                    Collision::Bottom => {
                        ball_velocity.0.y *= -1.; // Inverte direção vertical
                    }
                }
            }
        }
    }
}

// Sistema para criar as raquetes do jogador e da IA
fn spawn_paddles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    println!("Spawning paddles..."); // Log de depuração

    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let padding = 50.;
        // Calcula a posição das raquetes
        let right_paddle_x = window_width / 2. - padding; // Raquete do jogador à direita
        let left_paddle_x = -window_width / 2. + padding; // Raquete da IA à esquerda

        let shape = Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT);

        let mesh = meshes.add(shape);
        let player_color = materials.add(Color::srgb(0., 1., 0.)); // Verde para o jogador
        let ai_color = materials.add(Color::srgb(0., 0., 1.)); // Azul para a IA

        // Cria a raquete do jogador
        commands.spawn((
            Player,
            Paddle,
            Shape(shape.size()),
            Position(Vec2::new(right_paddle_x, 0.)),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(player_color.clone()),
        ));

        // Cria a raquete da IA
        commands.spawn((
            Ai,
            Paddle,
            Position(Vec2::new(left_paddle_x, 0.)),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(ai_color.clone()),
        ));
    }
}

// Sistema para criar a bola
fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Spawning ball..."); // Log de depuração

    let shape = Circle::new(BALL_SIZE);
    let color = Color::srgb(1., 0., 0.); // Vermelho

    let mesh = meshes.add(shape);
    let material = materials.add(color);

    // Cria a entidade da bola
    commands.spawn((Ball, Mesh2d(mesh), MeshMaterial2d(material)));
}

// Sistema para configurar a câmera 2D
fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2d); // Cria uma câmera 2D simples
}

// Função principal que configura e inicia o jogo
fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // Adiciona os plugins padrão do Bevy
        .init_resource::<Score>() // Inicializa o recurso de pontuação
        .init_resource::<BallSpeed>() // Inicializa o recurso de velocidade da bola
        .add_event::<Scored>() // Adiciona o evento de pontuação
        .add_systems(
            Startup, // Sistemas executados na inicialização
            (
                spawn_ball,       // Cria a bola
                spawn_camera,     // Configura a câmera
                spawn_paddles,    // Cria as raquetes
                spawn_gutters,    // Cria as barreiras superior e inferior
                spawn_scoreboard, // Cria o placar
            ),
        )
        .add_systems(
            Update, // Sistemas executados a cada quadro
            (
                move_ball,                               // Move a bola
                handle_player_input,                     // Processa entrada do jogador
                detect_scoring,                          // Detecta pontuação
                move_ai,                                 // Move a IA
                reset_ball.after(detect_scoring),        // Reseta a bola após pontuação
                update_score.after(detect_scoring),      // Atualiza a pontuação
                update_scoreboard.after(update_score),   // Atualiza o placar visual
                move_paddles.after(handle_player_input), // Move as raquetes
                project_positions.after(move_ball),      // Atualiza posições visuais
                handle_collisions.after(move_ball),      // Trata colisões
            ),
        )
        .run(); // Inicia o loop principal do jogo
}
