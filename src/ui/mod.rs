use bevy::{
    input_focus::tab_navigation::TabGroup,
    prelude::*,
};

use crate::{GameState, simulation::ControlSettings};

mod indicators;
mod sliders;

#[derive(Component)]
struct GameOverBanner;

pub struct ReactorUiPlugin;

impl Plugin for ReactorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui).add_systems(
            Update,
            (
                sliders::sync_slider_values,
                sliders::update_slider_visuals.after(sliders::sync_slider_values),
                sliders::update_slider_value_text,
                indicators::update_indicators,
                update_game_over_overlay,
            ),
        );
    }
}

fn setup_ui(
    mut commands: Commands,
    controls: Res<ControlSettings>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2d);

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    // Main UI root
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(30.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
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
            // Price display
            (
                Text::new("A$16"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
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

    // Game Over overlay
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        Visibility::Hidden,
        GlobalZIndex(100),
        GameOverBanner,
        children![(
            Text::new("GAME OVER"),
            TextFont {
                font: font.clone(),
                font_size: 72.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.2, 0.2)),
        )],
    ));
}

fn update_game_over_overlay(
    state: Res<State<GameState>>,
    mut query: Query<&mut Visibility, With<GameOverBanner>>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = if matches!(state.get(), GameState::GameOver) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

