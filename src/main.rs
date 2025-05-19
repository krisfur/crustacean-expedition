use bevy::prelude::*;
use bevy::app::AppExit;
use rand::Rng;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ParryTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .add_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, player_turn.run_if(in_state(GameState::PlayerTurn)))
        .add_systems(Update, enemy_telegraph.run_if(in_state(GameState::EnemyTelegraph)))
        .add_systems(Update, enemy_attack.run_if(in_state(GameState::EnemyAttack)))
        .add_systems(Update,check_end_conditions.run_if(not(in_state(GameState::GameOver))))
        .add_systems(Update, update_health_texts)
        .add_systems(Update, exit_on_keypress.run_if(in_state(GameState::GameOver)))
        .add_systems(OnEnter(GameState::GameOver), cleanup_gameplay_ui)
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum GameState {
    #[default]
    PlayerTurn,
    EnemyTelegraph,
    EnemyAttack,
    GameOver,
}

#[derive(Component)]
struct GameplayUI;

#[derive(Component)]
struct Health(i32);

#[derive(Component)]
struct MaxHealth(i32);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Resource)]
struct ParryTimer(Timer);

#[derive(Component)]
struct HealthText {
    owner: Entity,
}

#[derive(Component)]
struct CombatMessage;

#[derive(Component)]
struct GameOverMessage;

const ATTACK_DURATION: f32 = 0.4; // This changes the parry window! Telegraph durations is a RAND for fun


fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Player entity
    let player_entity = commands.spawn((
        TextBundle::from_section(
            r#"    /\
    ( /   @ @    ()
     \\ __| |__  /
      \/.   .".   .\/
     /-|  .   .   . |-\
    / /-\  .   .  /-\ \
     / /-`---'-\ \"#,
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Percent(10.0),
            bottom: Val::Percent(30.0),
            ..default()
        }),
        Player,
        Health(3),
        MaxHealth(3),
        GameplayUI,
    ))
    .id();

    // Player HP text
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "HP: 3 / 3",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(10.0),
                bottom: Val::Percent(20.0),
                ..default()
            },
            ..default()
        },
        HealthText { owner: player_entity },
        GameplayUI,
    ));

    // Enemy entity
    let enemy_entity = commands.spawn((
        TextBundle::from_section(
            "  .-\"-.\n ( o o )\n |  ∆  |\n | __ |\n '-----'  GHOST",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::RED,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            right: Val::Percent(10.0),
            top: Val::Percent(30.0),
            ..default()
        }),
        Enemy,
        Health(5),
        MaxHealth(5),
        GameplayUI,
    ))
    .id();

    // Enemy HP text
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "HP: 5 / 5",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Percent(10.0),
                top: Val::Percent(20.0),
                ..default()
            },
            ..default()
        },
        HealthText { owner: enemy_entity },
        GameplayUI,
    ));
}



fn player_turn(
    keyboard_input: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<ParryTimer>,
    mut query: Query<&mut Health, With<Enemy>>,
) {
    if keyboard_input.just_pressed(KeyCode::A) {
        if let Ok(mut enemy_health) = query.get_single_mut() {
            enemy_health.0 -= 1;
            println!("Player attacks! Enemy health is now {}", enemy_health.0);
        }
        next_state.set(GameState::EnemyTelegraph);
        let mut rng = rand::rng();
        let duration = rng.random_range(0.5..=1.5);
        timer.0 = Timer::from_seconds(duration, TimerMode::Once);
    }
}

fn enemy_telegraph(
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<ParryTimer>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    message_query: Query<Entity, With<CombatMessage>>,
) {
    if message_query.is_empty() {
        show_combat_message(&mut commands, asset_server, "Enemy is preparing to attack...");
    }

    if timer.0.tick(time.delta()).finished() {
        clear_combat_messages(&mut commands, &message_query);
        next_state.set(GameState::EnemyAttack);
        timer.0 = Timer::from_seconds(ATTACK_DURATION, TimerMode::Once); // reset for attack phase
    }
}


fn enemy_attack(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut timer: ResMut<ParryTimer>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    message_query: Query<Entity, With<CombatMessage>>,
) {
    if message_query.is_empty() {
        show_combat_message(&mut commands, asset_server, "PARRY NOW!");
    }

    timer.0.tick(time.delta());

    if timer.0.elapsed_secs() < 0.5 && keyboard_input.just_pressed(KeyCode::Space) {
        println!("Parry successful!");
        clear_combat_messages(&mut commands, &message_query);
        next_state.set(GameState::PlayerTurn);
        return;
    }

    if timer.0.finished() {
        if let Ok(mut player_health) = player_query.get_single_mut() {
            player_health.0 -= 1;
            println!("Enemy hits! Player health: {}", player_health.0);
        }

        clear_combat_messages(&mut commands, &message_query);
        next_state.set(GameState::PlayerTurn);
    }
}

fn update_health_texts(
    mut text_query: Query<(&HealthText, &mut Text)>,
    health_query: Query<(&Health, &MaxHealth)>,
) {
    for (health_text, mut text) in text_query.iter_mut() {
        if let Ok((health, max)) = health_query.get(health_text.owner) {
            text.sections[0].value = format!("HP: {} / {}", health.0, max.0);
        }
    }
}

fn show_combat_message(commands: &mut Commands, asset_server: Res<AssetServer>, message: &str) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                message,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::YELLOW,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                left: Val::Percent(35.0),
                ..default()
            },
            ..default()
        },
        CombatMessage,
        GameplayUI,
    ));
}

fn clear_combat_messages(commands: &mut Commands, query: &Query<Entity, With<CombatMessage>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn check_end_conditions(
    player_q: Query<&Health, With<Player>>,
    enemy_q: Query<&Health, With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if let Ok(health) = player_q.get_single() {
        if health.0 <= 0 {
            println!("You lost!");
            show_game_over_screen(&mut commands, &asset_server, "YOU LOSE");
            next_state.set(GameState::GameOver);
        }
    }

    if let Ok(health) = enemy_q.get_single() {
        if health.0 <= 0 {
            println!("You won!");
            show_game_over_screen(&mut commands, &asset_server, "YOU WIN");
            next_state.set(GameState::GameOver);
        }
    }
}

fn show_game_over_screen(commands: &mut Commands, asset_server: &Res<AssetServer>, text: &str) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(45.0),
                left: Val::Percent(35.0),
                ..default()
            },
            ..default()
        },
        GameOverMessage,
    ));
}

fn exit_on_keypress(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_exit: EventWriter<AppExit>,
) {
    if keyboard_input.get_just_pressed().next().is_some() {
        app_exit.send(AppExit);
    }
}

fn cleanup_gameplay_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<GameplayUI>>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}