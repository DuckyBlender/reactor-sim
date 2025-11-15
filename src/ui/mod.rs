use bevy::{
    input_focus::tab_navigation::TabGroup,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, Button, observe},
};

use crate::{
    GameState,
    simulation::{ControlSettings, EnvironmentState, GameOverReason, ReactorState, TurbineState, REACTOR_TEMP_LIMIT, TURBINE_TEMP_LIMIT},
};

pub mod indicators;
pub mod sliders;

#[derive(Component)]
struct GameOverReasonText;

#[derive(Component)]
struct GameOverUI;

#[derive(Component)]
struct MoneyText;

#[derive(Component)]
struct PauseMenu;

#[derive(Component)]
struct ReturnToMenuButton;

#[derive(Component)]
struct Uranek;

#[derive(Component)]
struct UranekText;

#[derive(Component)]
struct UranekIdle;

#[derive(Component)]
struct UranekBubble;

#[derive(Resource)]
struct UranekState {
    last_blink_time: f32,
    blink_frame: u8,
    last_comment_time: f32,
    last_default_text_time: f32,
    talk_timeout: f32,
}

impl Default for UranekState {
    fn default() -> Self {
        Self {
            last_blink_time: 0.0,
            blink_frame: 0,
            last_comment_time: -999.0,
            last_default_text_time: 0.0,
            talk_timeout: 0.0,
        }
    }
}

pub struct ReactorUiPlugin;

impl Plugin for ReactorUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UranekState::default())
            .add_systems(OnEnter(GameState::InGame), setup_game_ui)
            .add_systems(
                Update,
                (
                    sliders::sync_slider_values,
                    sliders::update_slider_visuals.after(sliders::sync_slider_values),
                    sliders::update_slider_value_text,
                    sliders::update_applied_value_text,
                ),
            )
            .add_systems(
                Update,
                (
                    sliders::spin_turbine_icon,
                    indicators::update_indicators,
                    indicators::update_gauge_colors,
                    indicators::handle_turbine_destroyed,
                    indicators::rebuild_turbine_gauge_from_buyback,
                    update_money_display,
                    handle_pause_input,
                    update_uranek_idle_animation,
                    update_uranek_dialogue,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(
                Update,
                handle_unpause_input.run_if(in_state(GameState::Paused)),
            )
            .add_systems(OnEnter(GameState::GameOver), setup_game_over_ui);
    }
}

fn setup_game_ui(
    mut commands: Commands,
    controls: Res<ControlSettings>,
    asset_server: Res<AssetServer>,
    mut uranek_state: ResMut<UranekState>,
    time: Res<Time>,
) {
    *uranek_state = UranekState::default();
    uranek_state.last_default_text_time = time.elapsed_secs();

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");
    let uranek_idle_0: Handle<Image> = asset_server.load("sprites/idle_0.png");

    // Main UI root
    commands.spawn((
        DespawnOnExit(GameState::InGame),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(40.0)),
            row_gap: Val::Px(60.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        TabGroup::default(),
        Transform::default(),
        children![
            // Title
            (
                TextFont {
                    font: font.clone(),
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    right: Val::Px(40.0),
                    ..default()
                },
            ),
            // Money display
            (
                Text::new("$0"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MoneyText,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(100.0),
                    right: Val::Px(40.0),
                    ..default()
                },
            ),
            // Uranek companion (top-left corner)
            (
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    left: Val::Px(20.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    align_items: AlignItems::End,
                    ..default()
                },
                children![
                    // Uranek sprite
                    (
                        Node {
                            width: Val::Px(220.0),
                            height: Val::Px(220.0),
                            ..default()
                        },
                        Uranek,
                        UranekIdle,
                        ImageNode::new(uranek_idle_0.clone()),
                    ),
                    // Speech bubble (initially visible with default text)
                    (
                        Node {
                            width: Val::Px(420.0),
                            min_height: Val::Px(80.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
                        BorderRadius::all(Val::Px(12.0)),
                        BorderColor::all(Color::srgb(0.6, 0.9, 1.0)),
                        UranekBubble,
                        children![
                            (
                                Text::new("Uranek: Pilnujmy, żeby ten reaktor trzymał się w ryzach."),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                UranekText,
                            ),
                        ],
                    ),
                ],
            ),
            // Bottom section with gauges and sliders
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::End,
                    ..default()
                },
                Transform::default(),
                children![
                    // Left side - Gauges
                    indicators::gauge_grid(font.clone()),
                    // Right side - Sliders
                    sliders::slider_panel(
                        controls.reactivity_target,
                        controls.turbine_target,
                        font,
                        &asset_server
                    ),
                ],
            ),
        ],
    ));
}

fn update_money_display(
    environment: Res<EnvironmentState>,
    mut texts: Query<&mut Text, With<MoneyText>>,
) {
    if !environment.is_changed() {
        return;
    }

    for mut text in texts.iter_mut() {
        **text = format!("${:.0}", environment.money);
    }
}

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

fn handle_unpause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::InGame);
    }
}

fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, DespawnOnExit(GameState::Paused)));

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    commands.spawn((
        DespawnOnExit(GameState::Paused),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        Transform::default(),
        GlobalTransform::default(),
        PauseMenu,
        children![
            (
                Text::new("PAUSED"),
                TextFont {
                    font: font.clone(),
                    font_size: 144.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ),
            (
                Text::new("Press ESC to resume"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ),
            (
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(120.0),
                    border: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(16.0)),
                BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                Button,
                Hovered::default(),
                ReturnToMenuButton,
                observe(
                    |_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::MainMenu);
                    },
                ),
                children![(
                    Text::new("Return to Menu"),
                    TextFont {
                        font,
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ),
        ],
    ));
}

fn setup_game_over_ui(
    mut commands: Commands,
    game_over_reason: Res<GameOverReason>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((Camera2d, DespawnOnExit(GameState::GameOver)));

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    let reason_text = match *game_over_reason {
        GameOverReason::ReactorOverheat => "Reactor temperature exceeded safe limits",
        GameOverReason::TurbineOverheat => "Turbine temperature exceeded safe limits",
        GameOverReason::None => "Unknown cause",
    };

    // Game Over screen
    commands.spawn((
        DespawnOnExit(GameState::GameOver),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        GameOverUI,
        children![
            (
                Text::new("GAME OVER"),
                TextFont {
                    font: font.clone(),
                    font_size: 144.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ),
            (
                Text::new(reason_text),
                TextFont {
                    font: font.clone(),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                GameOverReasonText,
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ),
            (
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(120.0),
                    border: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(16.0)),
                BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                Button,
                Hovered::default(),
                ReturnToMenuButton,
                observe(
                    |_activate: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::MainMenu);
                    },
                ),
                children![(
                    Text::new("Return to Menu"),
                    TextFont {
                        font,
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ),
        ],
    ));
}

fn update_uranek_idle_animation(
    time: Res<Time>,
    mut state: ResMut<UranekState>,
    mut sprite_query: Query<(&mut ImageNode, &mut UranekIdle)>,
    asset_server: Res<AssetServer>,
) {
    let now = time.elapsed_secs();

    // Blink parameters: every few seconds a short blink for 1 second
    let interval = 8.0; // interval between blinks (eyes open)
    let blink_duration = 1.0; // time of 'closed' eyes

    if state.blink_frame == 1 {
        // Blink phase – after a second we return to the base frame
        if now - state.last_blink_time >= blink_duration {
            state.blink_frame = 0;
            let handle: Handle<Image> = asset_server.load("sprites/idle_0.png");
            for (mut image_node, _) in sprite_query.iter_mut() {
                image_node.image = handle.clone();
            }
            // New reference point: from now we count the full interval to the next blink
            state.last_blink_time = now;
        }
        return;
    }

    // Eyes open – we check if the full interval to the next blink has passed
    if now - state.last_blink_time >= interval {
        state.last_blink_time = now;
        state.blink_frame = 1;
        let handle: Handle<Image> = asset_server.load("sprites/idle_1.png");
        for (mut image_node, _) in sprite_query.iter_mut() {
            image_node.image = handle.clone();
        }
    }
}

fn update_uranek_dialogue(
    time: Res<Time>,
    reactor: Res<ReactorState>,
    turbine: Res<TurbineState>,
    environment: Res<EnvironmentState>,
    mut state: ResMut<UranekState>,
    mut text_query: Query<&mut Text, With<UranekText>>,
    mut bubble_query: Query<(&mut Node, &mut BackgroundColor), With<UranekBubble>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let now = time.elapsed_secs();

    // 1) Handle initial greeting lifetime
    if now - state.last_default_text_time > 3.0 && state.last_default_text_time > 0.0 {
        for mut text in text_query.iter_mut() {
            **text = String::new();
        }
        state.last_default_text_time = 0.0;
    }

    // 2) Handle continuous talk timeout (for warning/earnings lines)
    //    If talk_timeout expired and this is not the initial greeting, clear text.
    if state.talk_timeout > 0.0 && now > state.talk_timeout {
        for mut text in text_query.iter_mut() {
            **text = String::new();
        }
        state.talk_timeout = 0.0;
    }

    // 3) Determine if Uranek has something new to say
    const COMMENT_COOLDOWN: f32 = 6.0;
    let mut message: Option<&'static str> = None;

    let reactor_ratio = reactor.temperature / REACTOR_TEMP_LIMIT;
    let turbine_ratio = turbine.temperature / TURBINE_TEMP_LIMIT;

    if now - state.last_comment_time >= COMMENT_COOLDOWN {
        if reactor_ratio > 0.95 {
            message = Some("Uranek: REAKTOR ZARAZ SIĘ ROZTOPI! Obniż reaktywność!");
        } else if reactor_ratio > 0.8 {
            message = Some("Uranek: Reaktor jest głęboko w czerwonym. Włóż pręty, teraz.");
        } else if turbine_ratio > 0.95 {
            message = Some("Uranek: Turbina wyje! Ochłodź ją, zanim wybuchnie!");
        } else if turbine_ratio > 0.8 {
            message = Some("Uranek: Turbina jest w strefie zagrożenia. Zmniejsz przepływ.");
        } else if environment.money > 1000.0 {
            message = Some("Uranek: Świetnie! Ta elektrownia drukuje pieniądze.");
        }
    }

    if let Some(msg) = message {
        // New line -> update text and set talk timeout
        for mut text in text_query.iter_mut() {
            **text = msg.to_string();
        }

        // Show speech bubble while talking
        if let Ok((mut node, mut bg)) = bubble_query.single_mut() {
            node.border = UiRect::all(Val::Px(2.0));
            *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85));
        }

        // Play talking sound
        commands.spawn(AudioPlayer::new(asset_server.load("sound/uranek_talking.mp3")));
        state.last_comment_time = now;

        // Bubble stays visible for this many seconds after the line
        state.talk_timeout = now + 4.0;
    }

    // 4) Final bubble visibility: show if there is any text, hide otherwise
    let mut has_any_text = false;
    for text in text_query.iter_mut() {
        if !text.to_string().is_empty() {
            has_any_text = true;
            break;
        }
    }

    if let Ok((mut node, mut bg)) = bubble_query.single_mut() {
        if has_any_text {
            node.border = UiRect::all(Val::Px(2.0));
            *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85));
        } else {
            node.border = UiRect::all(Val::Px(0.0));
            *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.0));
        }
    }
}
