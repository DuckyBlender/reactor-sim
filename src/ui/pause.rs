use bevy::prelude::*;

use crate::{GameState, FONT_REGULAR, menu::main_menu::ReturnToMenuButton};

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

    let font = asset_server.load(FONT_REGULAR);

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
                Text::new("PAUZA"),
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
                Text::new("Naciśnij ESC, aby wznowić"),
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
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ReturnToMenuButton,
                children![(
                    Text::new("Powrót do menu"),
                    TextFont {
                        font,
                        font_size: 24.0,
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
