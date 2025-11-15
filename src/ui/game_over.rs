use bevy::prelude::*;
use crate::{GameState, simulation::GameOverReason};

#[derive(Component)]
struct GameOverReasonText;

#[derive(Component)]
struct GameOverUI;

#[derive(Component)]
struct ReturnToMenuButton;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), setup_game_over_ui)
        .add_systems(Update, handle_return_button.run_if(in_state(GameState::GameOver)));
    }
}

fn setup_game_over_ui(
    mut commands: Commands,
    game_over_reason: Res<GameOverReason>,
    asset_server: Res<AssetServer>,
) {
    info!("State change: <Unknown> -> GameOver");
    // Camera
    commands.spawn((Camera2d, DespawnOnExit(GameState::GameOver)));

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    let reason_text = match *game_over_reason {
        GameOverReason::ReactorExplosion => "REACTOR EXPLOSION",
        GameOverReason::ReactorMeltdown => "REACTOR MELTDOWN",
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
                ReturnToMenuButton,
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

fn handle_return_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ReturnToMenuButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("State change: GameOver -> MainMenu");
            next_state.set(GameState::MainMenu);
        }
    }
}
