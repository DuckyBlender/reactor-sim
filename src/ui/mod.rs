use bevy::{
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    picking::hover::Hovered,
    ui_widgets::{Button, observe, Activate},
};

use crate::{GameState, simulation::{ControlSettings, GameOverReason, EnvironmentState}};

mod indicators;
mod sliders;

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

pub struct ReactorUiPlugin;

impl Plugin for ReactorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_game_ui)
            .add_systems(
                Update,
                (
                    sliders::sync_slider_values,
                    sliders::update_slider_visuals.after(sliders::sync_slider_values),
                    sliders::update_slider_value_text,
                    sliders::spin_turbine_icon,
                    indicators::update_indicators,
                    indicators::update_gauge_colors,
                    indicators::handle_turbine_destroyed,
                    indicators::rebuild_turbine_gauge_from_buyback,
                    update_money_display,
                    handle_pause_input,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(Update, handle_unpause_input.run_if(in_state(GameState::Paused)))
            .add_systems(OnEnter(GameState::GameOver), setup_game_over_ui);
    }
}

fn setup_game_ui(
    mut commands: Commands,
    controls: Res<ControlSettings>,
    asset_server: Res<AssetServer>,
) {
    // Camera for 3D
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 5., 12.).looking_at(Vec3::new(0., 4., 0.), Vec3::Y),
        DespawnOnExit(GameState::InGame)
    ));

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    // Main UI root
    commands.spawn((
        DespawnOnExit(GameState::InGame),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(30.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        TabGroup::default(),
        children![
            // Title
            (
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    right: Val::Px(20.0),
                    ..default()
                },
            ),
            // Money display
            (
                Text::new("A$0"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MoneyText,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    right: Val::Px(20.0),
                    ..default()
                },
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
                children![
                    // Left side - Gauges
                    indicators::gauge_grid(font.clone()),
                    // Right side - Sliders
                    sliders::slider_panel(
                        controls.reactivity_target,
                        controls.turbine_target,
                        font.clone(),
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
        **text = format!("A${:.0}", environment.money);
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

fn setup_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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
        PauseMenu,
        children![
            (
                Text::new("PAUSED"),
                TextFont {
                    font: font.clone(),
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ),
            (
                Text::new("Press ESC to resume"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ),
            (
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(8.0)),
                BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                Button,
                Hovered::default(),
                ReturnToMenuButton,
                observe(
                    |_activate: On<Activate>,
                     mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::MainMenu);
                    },
                ),
                children![(
                    Text::new("Return to Menu"),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
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
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ),
            (
                Text::new(reason_text),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                GameOverReasonText,
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ),
            (
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(8.0)),
                BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                Button,
                Hovered::default(),
                ReturnToMenuButton,
                observe(
                    |_activate: On<Activate>,
                     mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::MainMenu);
                    },
                ),
                children![(
                    Text::new("Return to Menu"),
                    TextFont {
                        font: font.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ),
        ],
    ));
}

