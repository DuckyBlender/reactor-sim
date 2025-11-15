use crate::{FONT_REGULAR, GameState, menu::main_menu::ReturnToMenuButton, simulation::GameOverReason};
use bevy::prelude::*;

#[derive(Component)]
struct GameOverReasonText;

#[derive(Component)]
struct GameOverUI;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), setup_game_over_ui)
            .add_systems(
                Update,
                handle_return_button.run_if(in_state(GameState::GameOver)),
            );
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

    let font = asset_server.load(FONT_REGULAR);

    let reason_text = match *game_over_reason {
        GameOverReason::ReactorExplosion => "EKSPLOZJA REAKTORA",
        GameOverReason::ReactorMeltdown => "ROZPAD REAKTORA",
        GameOverReason::None => "Nieznana przyczyna",
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
                Text::new("KONIEC GRY"),
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
            crate::menu::main_menu::create_return_to_menu_button(font),
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
