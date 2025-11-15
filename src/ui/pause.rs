use bevy::prelude::*;

use crate::GameState;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<PauseState>()
        .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
        .add_systems(Update, handle_pause_input)
        .add_systems(
            Update,
        (handle_unpause_input, handle_return_button).run_if(in_state(GameState::Paused)),
        );
    }
}

#[derive(Resource, Default)]
pub struct PauseState {
    pub previous_state: Option<GameState>,
}

#[derive(Component)]
struct PauseMenu;

#[derive(Component)]
struct ReturnToMenuButton;

fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut pause_state: ResMut<PauseState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        pause_state.previous_state = Some(*current_state.get());
        next_state.set(GameState::Paused);
    }
}

fn handle_unpause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    pause_state: Res<PauseState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        let resume_state = pause_state.previous_state.unwrap_or(GameState::InGame);
        next_state.set(resume_state);
    }
}

fn setup_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pause_state: ResMut<PauseState>,
) {
    // Store the previous state if not already stored (fallback to InGame)
    if pause_state.previous_state.is_none() {
        pause_state.previous_state = Some(GameState::InGame);
    }
    
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
                Transform::default(),
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
                Transform::default(),
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
            info!("State change: Paused -> MainMenu");
            next_state.set(GameState::MainMenu);
        }
    }
}