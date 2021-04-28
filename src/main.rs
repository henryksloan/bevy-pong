use bevy::{
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard {
            left_score: 0,
            right_score: 0,
        })
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_startup_system(setup.system())
        .add_system(paddle_movement_system.system())
        .add_system(ai_paddle_movement_system.system())
        .add_system(ball_collision_system.system())
        .add_system(ball_movement_system.system())
        .add_system(scoreboard_system.system())
        .run();
}

struct Paddle;
struct PlayerPaddle;
struct AiPaddle;

struct Ball {
    velocity: Vec3,
}

struct Scoreboard {
    left_score: usize,
    right_score: usize,
}

struct ScoreText {
    index: usize,
}

struct Collider;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Paddles
    let paddle_material = materials.add(Color::rgb(0.1, 0.1, 0.1).into());
    let paddle_offset = 525.0;
    let paddle_sprite = Sprite::new(Vec2::new(20.0, 120.0));
    // Left
    commands
        .spawn_bundle(SpriteBundle {
            material: paddle_material.clone(),
            transform: Transform::from_xyz(-paddle_offset, 0.0, 0.0),
            sprite: paddle_sprite.clone(),
            ..Default::default()
        })
        .insert(Paddle)
        .insert(PlayerPaddle)
        .insert(Collider);
    // Right
    commands
        .spawn_bundle(SpriteBundle {
            material: paddle_material.clone(),
            transform: Transform::from_xyz(paddle_offset, 0.0, 0.0),
            sprite: paddle_sprite.clone(),
            ..Default::default()
        })
        .insert(Paddle)
        .insert(AiPaddle)
        .insert(Collider);

    // Ball
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
            transform: Transform::from_xyz(0.0, -50.0, 1.0),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .insert(Ball {
            velocity: 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
        });

    // Score
    let score_text = Text {
        sections: vec![
            TextSection {
                value: "Score: ".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.5, 0.5, 1.0),
                },
            },
            TextSection {
                value: "".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(1.0, 0.5, 0.5),
                },
            },
        ],
        ..Default::default()
    };
    // Left
    commands
        .spawn_bundle(TextBundle {
            text: score_text.clone(),
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(20.0),
                    left: Val::Px(20.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScoreText { index: 0 });
    // Right
    commands
        .spawn_bundle(TextBundle {
            text: score_text.clone(),
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(20.0),
                    left: Val::Px(1100.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScoreText { index: 1 });
}

fn paddle_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<PlayerPaddle>>,
) {
    if let Ok(mut transform) = query.single_mut() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Up) {
            direction += 1.0;
        }

        if keyboard_input.pressed(KeyCode::Down) {
            direction -= 1.0;
        }

        let translation = &mut transform.translation;
        // move the paddle horizontally
        translation.y += time.delta_seconds() * direction * 200.0;
        // bound the paddle within the walls
        translation.y = translation.y.min(300.0).max(-300.0);
    }
}

fn ai_paddle_movement_system(
    time: Res<Time>,
    mut queries: QuerySet<(
        Query<(&Ball, &Transform)>,
        Query<&mut Transform, With<AiPaddle>>,
    )>,
) {
    let ball_translation = queries.q0().single().unwrap().1.translation;
    if let Ok(mut transform) = queries.q1_mut().single_mut() {
        let mut direction = 0.0;
        if ball_translation.x > 250.0 {
            if ball_translation.y > transform.translation.y {
                direction += 1.0;
            }

            if ball_translation.y < transform.translation.y {
                direction -= 1.0;
            }
        }

        let translation = &mut transform.translation;
        // move the paddle horizontally
        translation.y += time.delta_seconds() * direction * 200.0;
        // bound the paddle within the walls
        translation.y = translation.y.min(300.0).max(-300.0);
    }
}

fn ball_movement_system(
    mut scoreboard: ResMut<Scoreboard>,
    time: Res<Time>,
    mut ball_query: Query<(&mut Ball, &mut Transform)>,
) {
    // clamp the timestep to stop the ball from escaping when the game starts
    let delta_seconds = f32::min(0.2, time.delta_seconds());

    if let Ok((mut ball, mut transform)) = ball_query.single_mut() {
        transform.translation += ball.velocity * delta_seconds;

        if (transform.translation.y > 340.0 && ball.velocity.y > 0.0)
            || (transform.translation.y < -340.0 && ball.velocity.y < 0.0)
        {
            ball.velocity.y = -ball.velocity.y;
        }

        // TODO: Replace with score
        if transform.translation.x > 620.0 {
            scoreboard.left_score += 1;
            transform.translation = Vec3::ZERO;
        } else if transform.translation.x < -620.0 {
            scoreboard.right_score += 1;
            transform.translation = Vec3::ZERO;
        }
    }
}

fn ball_collision_system(
    mut ball_query: Query<(&mut Ball, &Transform, &Sprite)>,
    collider_query: Query<(&Transform, &Sprite), With<Collider>>,
) {
    if let Ok((mut ball, ball_transform, sprite)) = ball_query.single_mut() {
        let ball_size = sprite.size;
        let velocity = &mut ball.velocity;

        // check collision with walls
        for (transform, sprite) in collider_query.iter() {
            let collision = collide(
                ball_transform.translation,
                ball_size,
                transform.translation,
                sprite.size,
            );
            if let Some(collision) = collision {
                // reflect the ball when it collides
                let mut reflect_x = false;
                let mut reflect_y = false;

                // only reflect if the ball's velocity is going in the opposite direction of the
                // collision
                match collision {
                    Collision::Left => reflect_x = velocity.x > 0.0,
                    Collision::Right => reflect_x = velocity.x < 0.0,
                    Collision::Top => reflect_y = velocity.y < 0.0,
                    Collision::Bottom => reflect_y = velocity.y > 0.0,
                }

                // reflect velocity on the x-axis if we hit something on the x-axis
                if reflect_x {
                    velocity.x = -velocity.x;
                }

                // reflect velocity on the y-axis if we hit something on the y-axis
                if reflect_y {
                    velocity.y = -velocity.y;
                }

                break;
            }
        }
    }
}

fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<(&mut Text, &ScoreText)>) {
    for (mut text, score_text) in query.iter_mut() {
        let new_text = if score_text.index == 0 {
            scoreboard.left_score
        } else {
            scoreboard.right_score
        };
        text.sections[0].value = format!("Score: {}", new_text);
    }
}
