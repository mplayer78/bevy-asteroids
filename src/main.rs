use std::f32::consts::PI;

use bevy::prelude::*;
use rand::prelude::random;

fn main() {
    App::new()
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_spaceship)
        .add_startup_system(spawn_meteor)
        .add_startup_system(randomise_position)
        .add_system(boosters_fire)
        .add_system(spaceship_movement)
        .add_system(movement)
        .add_system(screen_wrap)
        .add_system(screen_despawn)
        .add_system(set_translation_from_position)
        .add_system(rotate_to_heading)
        .add_system(asteroid_collision)
        .add_system(spawn_bullet)
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct ReadyToFire(bool);

const BULLET_COLOUR: Color = Color::rgb(0.7, 0.5, 0.5);

fn spawn_bullet(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Position, &Movement), With<Spaceship>>,
) {
    let (player_pos, player_mov) = query.single_mut();
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: BULLET_COLOUR,
                    ..default()
                },
                transform: Transform {
                    scale: Vec3::new(5.0, 5.0, 5.0),
                    ..default()
                },
                ..default()
            })
            .insert(Bullet)
            .insert(Movement {
                heading: player_mov.orientation,
                velocity: player_mov.velocity + 2.0,
                orientation: 0.0,
            })
            .insert(ScreenDespawn)
            .insert(Position {
                x: player_pos.x,
                y: player_pos.y,
            });
    }
}

#[derive(Component)]
struct ScreenWrap;

#[derive(Component)]
struct ScreenDespawn;

const PLAYER_SPRITE: &str = "spaceship.png";

fn spawn_spaceship(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(PLAYER_SPRITE),
            sprite: Sprite {
                custom_size: Some(Vec2::new(6.5, 3.3)),
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(10.0, 10.0, 0.0),
                rotation: Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), -3.141 / 2.0),
                ..default()
            },
            ..default()
        })
        .insert(Spaceship)
        .insert(Movement {
            heading: 0.0,
            velocity: 0.0,
            orientation: PI,
        })
        .insert(ScreenWrap)
        .insert(Booster {
            turn: 0.0,
            thrust: 0.0,
        })
        .insert(Position {
            x: ((random::<f32>() - 0.5) * 1280.0),
            y: ((random::<f32>() - 0.5) * 720.0),
        });
}

fn spaceship_movement(mut ship_position: Query<(&mut Movement, &Booster), With<Spaceship>>) {
    for (mut movement, booster) in ship_position.iter_mut() {
        movement.orientation = (movement.orientation + booster.turn + 2.0 * PI) % (2.0 * PI);
        let mut x_travel = movement.heading.cos() * movement.velocity
            + movement.orientation.cos() * booster.thrust;
        let y_travel = movement.heading.sin() * movement.velocity
            + movement.orientation.sin() * booster.thrust;
        let new_velocity = (x_travel.powf(2.0) + y_travel.powf(2.0)).powf(0.5);
        if x_travel == 0.0 {
            x_travel = 0.0000000001; // handle division by 0 in case of no movement
        };
        let new_heading = ((y_travel / x_travel).atan() + 2.0 * PI) % (2.0 * PI);
        movement.velocity = new_velocity;
        movement.heading = new_heading;
    }
}

const FORWARD_THRUSTER: f32 = 0.1;
const TURN_THRUSTER: f32 = 0.005;

fn boosters_fire(
    keyboard_input: Res<Input<KeyCode>>,
    mut boosters: Query<&mut Booster, With<Spaceship>>,
) {
    let mut booster = boosters.single_mut();
    if keyboard_input.pressed(KeyCode::Left) {
        booster.turn -= TURN_THRUSTER;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        booster.turn += TURN_THRUSTER;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        booster.thrust = FORWARD_THRUSTER;
    } else {
        booster.thrust = 0.0;
    }
}

fn rotate_to_heading(mut item_position: Query<(&Movement, &mut Transform)>) {
    for (movement, mut transform) in item_position.iter_mut() {
        transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), movement.orientation);
    }
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
struct Movement {
    heading: f32,
    velocity: f32,
    orientation: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
struct Booster {
    turn: f32,
    thrust: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Meteor;

#[derive(Component)]
struct RandomMovement {
    max_speed: f32,
}

const METEOR_COLOUR: Color = Color::rgb(0.7, 0.1, 0.2);

fn spawn_meteor(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: METEOR_COLOUR,
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(10.0, 10.0, 10.0),
                ..default()
            },
            ..default()
        })
        .insert(Meteor)
        .insert(Position {
            x: ((random::<f32>() - 0.5) * 1280.0),
            y: ((random::<f32>() - 0.5) * 720.0),
        })
        .insert(Movement {
            heading: random::<f32>() * 3.141 * 2.0,
            velocity: random::<f32>() * 2.0,
            orientation: 0.0,
        })
        .insert(ScreenDespawn);
}

fn randomise_position(windows: Res<Windows>, mut meteor: Query<&mut Position, With<Meteor>>) {
    let window = windows.get_primary().unwrap();
    for mut position in meteor.iter_mut() {
        position.x = (random::<f32>() - 0.5) * window.width();
        position.y = (random::<f32>() - 0.5) * window.height();
    }
}

fn set_translation_from_position(mut positionable: Query<(&mut Position, &mut Transform)>) {
    for (position, mut transform) in positionable.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn screen_wrap(windows: Res<Windows>, mut wrappable: Query<(&mut Position, &ScreenWrap)>) {
    let window = windows.get_primary().unwrap();
    for (mut position, _) in wrappable.iter_mut() {
        if position.x.abs() > window.width() / 2.0 {
            position.x *= -1.0;
        }
        if position.y.abs() > window.height() / 2.0 {
            position.y *= -1.0;
        }
    }
}

fn screen_despawn(
    mut commands: Commands,
    windows: Res<Windows>,
    mut wrappable: Query<(Entity, &mut Position, &ScreenDespawn)>,
) {
    let window = windows.get_primary().unwrap();
    for (entity, position, _) in wrappable.iter_mut() {
        if position.x.abs() > window.width() / 2.0 || position.y.abs() > window.height() / 2.0 {
            commands.entity(entity).despawn()
        }
    }
}

fn movement(mut moveable: Query<(&mut Position, &mut Movement)>) {
    for (mut position, movement) in moveable.iter_mut() {
        let delta_x = movement.orientation.cos().abs() * movement.velocity;
        let delta_y = movement.orientation.sin() * movement.velocity;
        position.x += delta_x;
        position.y += delta_y;
    }
}

fn asteroid_collision(
    mut commands: Commands,
    spaceship_position: Query<&Position, With<Spaceship>>,
    asteroid_positions: Query<(Entity, &Position), With<Meteor>>,
) {
    for spaceship_pos in spaceship_position.iter() {
        for (ent, asteroid_position) in asteroid_positions.iter() {
            if (spaceship_pos.x - asteroid_position.x).abs() < 20.0
                && (spaceship_pos.y - asteroid_position.y).abs() < 20.0
            {
                commands.entity(ent).despawn();
            }
        }
    }
}
