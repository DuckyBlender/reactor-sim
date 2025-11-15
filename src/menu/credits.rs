use bevy::prelude::*;
use crate::{GameState, FONT_REGULAR};

#[derive(Component)]
struct BackButton;

pub struct CreditsMenuPlugin;

impl Plugin for CreditsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Credits), setup_credits)
        .add_systems(
            Update,
            (handle_back_button)
                .run_if(in_state(GameState::Credits)),
        );
    }
}

fn handle_back_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("State change: Credits -> MainMenu");
            next_state.set(GameState::MainMenu);
        }
    }
}

fn setup_credits(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(FONT_REGULAR);
    
    commands.spawn((Camera2d, DespawnOnExit(GameState::Credits)));
    commands
        .spawn((
            DespawnOnExit(GameState::Credits),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(
                    r#"
Symulator Reaktora z Urankiem

Autorzy:

Kacper Sowinski - Project Manager, Developer
Alan Klas - Lead Developer, Code Reviewer, 
Mateusz Oskar Kmiec - Developer, Sound & Visual Designer,
Ignacy Sztykiel - Developer, Sound & Visual Designer
                
                "#,
                ),
                TextFont {
                    font: font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BackButton,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Powrót"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

