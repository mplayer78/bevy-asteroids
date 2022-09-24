use std::f32::consts::PI;

use bevy::{prelude::*, transform, ecs::{query, entity}, winit::WinitSettings};
use bevy_rapier2d::prelude::*;
use rand::random;

pub struct GameEvents;

impl Plugin for GameEvents {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MeteorSpawnEvent>()
            .add_event::<StartGameEvent>()
            .add_event::<ShipSpawnEvent>();
    }
}

pub struct SetupScreen;

impl Plugin for SetupScreen {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_graphics)
            .add_system(screen_wrap)
            .add_system(screen_despawn);
    }
}

fn main() {
    App::new()
        .add_startup_system(setup_game)
        .add_plugin(GameEvents)
        .add_plugin(SetupScreen)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_system(setup_physics)
        .add_system(button_interaction)
        .add_system(update_game_state)
        .add_plugin(UpdateUI)
        .add_system(controls)
        .add_system(create_meteor)
        .add_system(create_ship)
        .add_system(spaceship_collision)
        .add_system(spawn_bullet)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn_bundle(Camera2dBundle::default());
}

const ASTEROID_BASE: f32 = 16.25;

struct MeteorSpawnEvent {
    size: u8,
    initial_velocity: Vec2,
    initial_position: Vec2
}

struct ShipSpawnEvent {
    initial_position: Vec2,
    initial_orientation: f32
}

struct StartGameEvent;

fn setup_physics(
    windows: Res<Windows>,
    mut game_query: Query<&mut Game>,
    mut meteor_event: EventWriter<MeteorSpawnEvent>,
    mut ship_event: EventWriter<ShipSpawnEvent>
) {
    let window = windows.get_primary().unwrap();
    
    /* Create the ground. */
    let mut game = game_query.single_mut();
    if matches!(game.gameState, GameState::Loading) {
        meteor_event.send(MeteorSpawnEvent {
            initial_velocity: Vec2 { x: random::<f32>() * 100.0 - 50.0, y: random::<f32>() * 100.0 - 50.0 },
            initial_position: Vec2 {
                x: ((random::<f32>() - 0.5) * window.width()),
                y: ((random::<f32>() - 0.5) * window.height()),
             },
            size: 8
        });
        
        ship_event.send(ShipSpawnEvent {
            initial_position: Vec2 { x: 0.0, y: 0.0 },
            initial_orientation: PI / 2.0
        });
        
        game.gameState = GameState::InProgress
    }
}

#[derive(Component)]
struct ScreenWrap;

#[derive(Component)]
struct ScreenDespawn;

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Meteor {
    size: u8
}

const INITIAL_SCORE: u8 = 0;
const INITIAL_LIVES: u8 = 3;

#[derive(Component)]
struct Game {
    score: u8,
    lives: u8,
    gameState: GameState
}

enum GameState {
    Loading,
    InProgress,
    Ended,
    Waiting
}

fn setup_game(
    mut commands: Commands
) {
    commands
        .spawn()
        .insert(Game {
            score: INITIAL_SCORE,
            lives: INITIAL_LIVES,
            gameState: GameState::Waiting
        });
}

fn controls(
    keyboard_input: Res<Input<KeyCode>>,
    mut body: Query<(&mut Transform, &mut ExternalImpulse, &mut Velocity)>
) {
    for (mut transform, mut impulse, mut velocity) in body.iter_mut() {
        if keyboard_input.pressed(KeyCode::Up) {
            let axis_angle = transform.rotation.to_axis_angle();
            impulse.impulse = Vec2::from_angle(axis_angle.1 * axis_angle.0.z) * 1.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            velocity.angvel = 0.0;
            transform.rotate_axis(Vec3::new(0.0, 0.0, 1.0), 0.1);
        }
        if keyboard_input.pressed(KeyCode::Right) {
            velocity.angvel = 0.0;
            transform.rotate_axis(Vec3::new(0.0, 0.0, 1.0), -0.1);
        }
    }    
}

fn screen_wrap(windows: Res<Windows>, mut q: Query<(&mut Transform, &Sprite, &ScreenWrap )>) {
    let window = windows.get_primary().unwrap();
    for (mut transform, sprite, _) in q.iter_mut() {
        let mut biggest_dimension = 0.0;
        if let Some(size) = sprite.custom_size {
            biggest_dimension = size.x.max(size.y)
        }
        if (transform.translation.x.abs() - biggest_dimension / 2.0) > window.width() / 2.0 {
            transform.translation.x *= -1.0;
        }
        if (transform.translation.y.abs() - biggest_dimension / 2.0) > window.height() / 2.0 {
            transform.translation.y *= -1.0;
        }
    }
}

fn screen_despawn(
    windows: Res<Windows>, 
    mut commands: Commands, 
    mut q: Query<(&Transform,  Entity, &ScreenDespawn )>
) {
    let window = windows.get_primary().unwrap();
    for (transform, entity, _) in q.iter_mut() {
        let biggest_dimension = 0.0;
        if (transform.translation.x.abs() - biggest_dimension / 2.0) > window.width() / 2.0 {
            commands.entity(entity).despawn();
        }
        if (transform.translation.y.abs() - biggest_dimension / 2.0) > window.height() / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spaceship_collision(
    rapier_context: Res<RapierContext>,
    query_ship: Query<Entity, With<Spaceship>>,
    query_meteor: Query<(Entity, &Meteor, &Velocity, &Transform), With<Meteor>>,
    query_bullets: Query<Entity, With<Bullet>>,
    mut query_game: Query<&mut Game>,
    mut commands: Commands,
    mut meteor_event: EventWriter<MeteorSpawnEvent>,
    mut ship_event: EventWriter<ShipSpawnEvent>
) {
    for (entity_meteor, meteor, meteor_velocity, transform) in query_meteor.iter() {
        let mut game = query_game.single_mut();

        for entity_ship in query_ship.iter() {
            if rapier_context.intersection_pair(entity_meteor, entity_ship) == Some(true) {
                commands.entity(entity_ship).despawn();
                game.lives -= 1;
                if game.lives > 0 {
                    ship_event.send(ShipSpawnEvent {
                        initial_position: Vec2 { x: 0.0, y: 0.0 },
                        initial_orientation: PI / 2.0
                    });
                } else {
                    game.gameState = GameState::Ended
                }
            }
        }
        
        for entity_bullets in query_bullets.iter() {
            if rapier_context.intersection_pair(entity_meteor, entity_bullets) == Some(true) {
                game.score += 1;
                    
                if meteor.size > 2 {
                    meteor_event.send(MeteorSpawnEvent { 
                        size: meteor.size / 2,
                        initial_velocity: meteor_velocity.linvel.rotate(Vec2::from_angle(0.7)),
                        initial_position: Vec2 {
                            x: transform.translation.x,
                            y: transform.translation.y
                        }
                    });
                    meteor_event.send(MeteorSpawnEvent { 
                        size: meteor.size / 2,
                        initial_velocity: meteor_velocity.linvel.rotate(Vec2::from_angle(-0.7)),
                        initial_position: Vec2 {
                            x: transform.translation.x,
                            y: transform.translation.y
                        }
                    });
                }
                commands.entity(entity_meteor).despawn();
                commands.entity(entity_bullets).despawn();
            }
        }
    }
}

fn create_meteor(
    mut meteor_event: EventReader<MeteorSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    for ev in meteor_event.iter() {
        let sprite_string = format!("meteor_{}.png", ev.size);
        commands.spawn()
            .insert_bundle(SpriteBundle {
                texture: asset_server.load(&sprite_string),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(ASTEROID_BASE, ASTEROID_BASE) * (ev.size as f32)),
                    ..default()
                },
                transform: Transform {
                    scale: Vec3::new(10.0, 10.0, 0.0),
                    rotation: Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), -3.141 / 2.0),
                    ..default()
                },
                ..default()
            })
            .insert(Velocity {
                linvel: ev.initial_velocity,
                ..default()
            })
            .insert(Meteor { size: ev.size })
            .insert(Collider::ball(ASTEROID_BASE * (ev.size as f32) / 2.0))
            .insert_bundle(TransformBundle::from(Transform::from_xyz(
                    ev.initial_position.x,
                    ev.initial_position.y,
                    0.0,
            )))
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(0.0))
            .insert(ScreenWrap);
    }
}

fn create_ship(
    mut ship_event: EventReader<ShipSpawnEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    for ev in ship_event.iter() {
        println!("Ship Spawn");
        commands
        .spawn()
        .insert_bundle(SpriteBundle {
            texture: asset_server.load("spaceship.png").clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(65.0, 33.0)),
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(10.0, 10.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Spaceship)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(RigidBody::Dynamic)
        .insert(Collider::capsule_x(33.0 / 2.0, 33.0 / 2.0))
        .insert(Restitution::coefficient(0.7))
        .insert(GravityScale(0.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(ev.initial_position.x, ev.initial_position.y, 0.0)))
        .insert_bundle(TransformBundle::from(Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), ev.initial_orientation))))
        .insert(ExternalImpulse {
            ..default()
        })
        .insert(Velocity {
            ..default()
        })
        .insert(Sensor)
        .insert(ScreenWrap);
    }
}

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct ReadyToFire(bool);

const BULLET_COLOUR: Color = Color::rgb(0.7, 0.5, 0.5);

const BULLET_SPEED: f32 = 200.0;

fn spawn_bullet(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(&Velocity, &Transform), With<Spaceship>>,
) {
    for (ship_velocity, ship_transform) in query.iter() {
        let (axes, ang) = ship_transform.rotation.to_axis_angle();
        if keyboard_input.just_pressed(KeyCode::Space) {
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: BULLET_COLOUR,
                        ..default()
                    },
                    transform: Transform {
                        scale: Vec3::new(5.0, 5.0, 5.0),
                        translation: Vec3::new(65.0 / 2.0, 33.0 / 2.0, 0.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(Bullet)
                .insert(RigidBody::KinematicVelocityBased)
                .insert(Collider::ball(2.5))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(
                    ship_transform.translation.x, 
                    ship_transform.translation.y, 
                    ship_transform.translation.z
                )))
                .insert(Velocity {
                    linvel: Vec2::from_angle(axes.z * ang) * BULLET_SPEED + ship_velocity.linvel,
                    angvel: 0.0,
                })
                .insert(ScreenDespawn)
                .insert(Sensor);
        }
    }
}

#[derive(Component)]
struct ScoreUI;

#[derive(Component)]
struct LivesUI;

#[derive(Component)]
struct GameMessage;

#[derive(Component)]
struct GameAction;

fn update_game_state(
    mut commands: Commands,
    mut game_event: EventReader<StartGameEvent>,
    mut query_game: Query<&mut Game>,
    entity_query: Query<Entity, With<Meteor>>
) {
    let mut game = query_game.single_mut();

    for _ev in game_event.iter() {
        game.gameState = GameState::Loading;
        game.lives = INITIAL_LIVES;
        game.score = INITIAL_SCORE;
    }
    
    for entity in entity_query.iter() {
        match game.gameState {
            GameState::Ended => commands.entity(entity).despawn(),
            _ => ()
        }
    }
    
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut start_game_event: EventWriter<StartGameEvent>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                start_game_event.send(StartGameEvent);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
        
        commands
        .spawn_bundle(NodeBundle {
            style: Style {
                padding: UiRect { left: Val::Px(10.0), right: Val::Px(10.0), top: Val::Px(10.0), bottom: Val::Px(10.0) },
                size: Size { width: Val::Percent(100.0), height: Val::Percent(100.0) },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: format!("Game Over"),
                                style: TextStyle {
                                    font: asset_server.load("BungeeSpice-Regular.ttf"),
                                    font_size: 40.0,
                                    color: Color::rgb(0.0, 1.0, 0.0),
                                },
                            },
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(GameMessage);
            
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect {left: Val::Px(10.0), right: Val::Px(10.0), top: Val::Px(10.0), bottom: Val::Px(10.0)},
                        margin: UiRect {top: Val::Px(10.0), bottom: Val::Px(10.0), ..default()},
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Start Game",
                        TextStyle {
                            font: asset_server.load("BungeeSpice-Regular.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                })
                .insert(GameAction);
        });     
    
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                padding: UiRect { left: Val::Px(10.0), right: Val::Px(10.0), top: Val::Px(10.0), bottom: Val::Px(10.0) },
                size: Size { width: Val::Percent(100.0), height: Val::Percent(100.0) },
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            color: UiColor::from(Color::Rgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: format!("Final Score: {}", 0),
                            style: TextStyle {
                                font: asset_server.load("BungeeSpice-Regular.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.0, 1.0, 0.0),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(ScoreUI);
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: format!("Lives: {}", 3),
                            style: TextStyle {
                                font: asset_server.load("BungeeSpice-Regular.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.0, 1.0, 0.0),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(LivesUI);
        });     
}

fn update_score(
    query_game: Query<&Game>,
    mut query_score: Query<&mut Text, With<ScoreUI>>,
) {    
    for mut ts in query_score.iter_mut() {
        if let Some(text_value) = ts.sections.get_mut(0) {
          if let Ok(game) = query_game.get_single() {
            text_value.value = format!("Final Score: {}", game.score);
          }
        }
    }
}
fn update_lives(
    query_game: Query<&Game>,
    mut query_lives: Query<&mut Text, With<LivesUI>>,
) {    
    for mut ts in query_lives.iter_mut() {
        if let Some(text_value) = ts.sections.get_mut(0) {
          if let Ok(game) = query_game.get_single() {
            text_value.value = format!("Lives: {}", game.lives);
          }
        }
    }
}

fn update_button(
    query_game: Query<&Game>,
    mut query_button: Query<(&mut Style, &GameAction)>,
    // mut query_message: Query<(&mut Style, &GameMessage)>,
) {
    let game = query_game.single();
    for (mut button_style, _) in query_button.iter_mut() {
        match game.gameState {
            GameState::InProgress => button_style.display = Display::None,
            GameState::Ended | GameState::Waiting | GameState::Loading => button_style.display = Display::Flex,
        }
    }
}

fn update_message(
    query_game: Query<&Game>,
    mut query_message: Query<&mut Style, With<GameMessage>>,
) {
    let game = query_game.single();
    
    for mut message_style in query_message.iter_mut() {
        match game.gameState {
            GameState::InProgress | GameState::Waiting | GameState::Loading => message_style.display = Display::None,
            GameState::Ended => message_style.display = Display::Flex,
        }
    }
}

pub struct UpdateUI;

impl Plugin for UpdateUI {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_ui)
            .add_system(update_score)
            .add_system(update_lives)
            .add_system(update_button)
            .add_system(update_message);
    }
}