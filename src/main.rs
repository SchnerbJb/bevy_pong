use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::{prelude::*, rapier::crossbeam::channel::after};

const PADDLE_SPEED: f32 = 10.;
const PADDLE_HEIGHT: f32 = 100.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(Msaa::Off)
        .add_startup_system(setup)
        .add_system(ball_movement)
        .add_system(paddle_movement)
        .add_system(detect_collisions)
        .add_system(update_text)
        .run();
}

enum PaddleDirection {
    UP,
    DOWN,
}

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Resource)]
struct Style(TextStyle);

#[derive(Component)]
struct Ball {
    velocity: f32,
    horizontal_direction: f32,
    vertical_direction: f32,
}

#[derive(Component)]
struct Paddle {
    player_id: i8,
}

#[derive(Component)]
struct Score {
    score: i8,
    player_id: i8,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let collision_sound = asset_server.load("sounds_breakout_collision.ogg");
    commands.insert_resource(CollisionSound(collision_sound));
    let font = asset_server.load("font.ttf");
    let first_style = TextStyle {
        font: font.clone(),
        font_size: 120.,
        color: Color::BLUE,
    };
    commands.insert_resource(Style(first_style.clone()));
    let second_style = TextStyle {
        font,
        font_size: 120.,
        color: Color::RED,
    };
    commands.insert_resource(Style(second_style.clone()));
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("0", first_style).with_alignment(TextAlignment::Center),
            transform: Transform::from_xyz(400., 300., 0.),
            ..default()
        },
        Score {
            score: 0,
            player_id: 1,
        },
    ));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("0", second_style).with_alignment(TextAlignment::Center),
            transform: Transform::from_xyz(-400., 300., 0.),
            ..default()
        },
        Score {
            score: 0,
            player_id: 2,
        },
    ));

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(20.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_xyz(0., 0., 10.),
            ..default()
        })
        .insert(Ball {
            velocity: 0.,
            horizontal_direction: 1.,
            vertical_direction: 1.,
        })
        .insert(RigidBody::Dynamic)
        .insert(Ccd::enabled())
        .insert(GravityScale(0.0))
        .insert(Collider::ball(20.))
        .insert(ActiveEvents::COLLISION_EVENTS);
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Box::new(30., PADDLE_HEIGHT, 0.).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_xyz(-500., 0., 0.),
            ..default()
        })
        .insert(Paddle { player_id: 1 })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(30. / 2., PADDLE_HEIGHT / 2.))
        .insert(ActiveEvents::COLLISION_EVENTS);

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Box::new(30., PADDLE_HEIGHT, 0.).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_xyz(500., 0., 0.),
            ..default()
        })
        .insert(Paddle { player_id: 2 })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(30. / 2., PADDLE_HEIGHT / 2.))
        .insert(ActiveEvents::COLLISION_EVENTS);
}


fn update_text(mut scores_query: Query<(&mut Text, &Score)>) {
    for ( mut text, score ) in scores_query.iter_mut() {
        text.sections[0].value = score.score.to_string();
    }
}

fn move_ball(
    mut ball_transform: &mut Transform,
    mut ball: &mut Ball,
    height: f32,
    width: f32,
    scores_query: &mut Query<'_, '_, &mut Score>,
) {
    ball_transform.translation.x += ball.horizontal_direction * ball.velocity;
    ball_transform.translation.y += ball.vertical_direction * ball.velocity;

    // ball hit the back wall and need to disapear and change the score
    if ball_transform.translation.x > width / 2. {
        ball_transform.translation.x = width / 2.;
        ball.horizontal_direction = -ball.horizontal_direction;
        for mut score in scores_query.iter_mut() {
            if score.player_id == 2 {
                score.score += 1;
            }
            ball_transform.translation.x = 0.0;
            ball_transform.translation.y = 0.0;
            ball.velocity = 0.0;
        }
    }
    if ball_transform.translation.x < -width / 2. {
        ball_transform.translation.x = -width / 2.;
        ball.horizontal_direction = -ball.horizontal_direction;
        for mut score in scores_query.iter_mut() {
            if score.player_id == 1 {
                score.score += 1;
            }
            ball_transform.translation.x = 0.0;
            ball_transform.translation.y = 0.0;
            ball.velocity = 0.0;
        }
    }

    // ball hit the top or bottom, and need to change direction
    if ball_transform.translation.y > height / 2. {
        ball_transform.translation.y = height / 2.;
        ball.vertical_direction = -ball.vertical_direction;
    }
    if ball_transform.translation.y < -height / 2. {
        ball_transform.translation.y = -height / 2.;
        ball.vertical_direction = -ball.vertical_direction;
    }
}

fn ball_movement(
    mut query: Query<(&mut Transform, &mut Ball)>,
    mut window: Query<&mut Window>,
    mut scores_query: Query<&mut Score>
) {
    let window = window.get_single_mut().unwrap();
    for (mut transform, mut ball) in query.iter_mut() {
        move_ball(
            transform.as_mut(),
            ball.as_mut(),
            window.height(),
            window.width(),
            &mut scores_query
        );
    }
}

fn move_paddle(paddle_transform: &mut Transform, height: f32, direction: PaddleDirection) {
    match direction {
        PaddleDirection::UP => {
            if paddle_transform.translation.y + PADDLE_HEIGHT / 2. < height / 2. {
                paddle_transform.translation.y += PADDLE_SPEED;
            }
        }
        PaddleDirection::DOWN => {
            if paddle_transform.translation.y - PADDLE_HEIGHT / 2. > -height / 2. {
                paddle_transform.translation.y -= PADDLE_SPEED;
            }
        }
    }
}

fn paddle_movement(
    mut query: Query<(&mut Transform, &Paddle)>,
    key: Res<Input<KeyCode>>,
    mut window: Query<&mut Window>,
    mut ball: Query<&mut Ball>,
) {
    let window = window.get_single_mut().unwrap();
    let height = window.height();

    for (mut transform, paddle) in query.iter_mut() {
        if paddle.player_id == 1 {
            let mut ball = ball.get_single_mut().unwrap();
            if key.pressed(KeyCode::W) {
                move_paddle(transform.as_mut(), height, PaddleDirection::UP);
                if ball.velocity == 0. {
                    ball.velocity = 10.;
                }
            }
            else if key.pressed(KeyCode::S) {
                move_paddle(transform.as_mut(), height, PaddleDirection::DOWN);
                if ball.velocity == 0. {
                    ball.velocity = 10.;
                }
            }
        } else {
            let mut ball = ball.get_single_mut().unwrap();
            if key.pressed(KeyCode::O) {
                move_paddle(transform.as_mut(), height, PaddleDirection::UP);
                if ball.velocity == 0. {
                    ball.velocity = 10.;
                }
            }
            else if key.pressed(KeyCode::L) {
                move_paddle(transform.as_mut(), height, PaddleDirection::DOWN);
                if ball.velocity == 0. {
                    ball.velocity = 10.;
                }
            }
        }
    }
}

fn detect_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<&mut Ball>,
    sound: Res<CollisionSound>,
    audio: Res<Audio>,
) {
    let mut ball = query.get_single_mut().unwrap();
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(_, _, _) = collision_event {
            // audio.play(sound.0.clone());
            ball.horizontal_direction = -ball.horizontal_direction;
        }
    }
}
