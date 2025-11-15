use crate::GameState;
use bevy::prelude::*;

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct QuitButton;

#[derive(Component)]
struct CreditsButton;

#[derive(Component)]
struct TutorialButton;

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct SettingsButton;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(
            Update,
            (
                button_system,
                handle_play_button,
                handle_quit_button,
                handle_credits_button,
                handle_tutorial_button,
                handle_settings_button,
            )
                .run_if(in_state(GameState::MainMenu)),
        )
         .add_systems(
            Update,
            (button_system, handle_back_button)
                .run_if(in_state(GameState::Credits).or(in_state(GameState::Settings))),
        );
    }
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(GameState::MainMenu)));
    commands
        .spawn((
            DespawnOnExit(GameState::MainMenu),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Symulator Reaktora z Urankiem"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.75, 0.25)),
                    PlayButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Endless Mode"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.4, 0.8)),
                    TutorialButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Tutorial"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.00, 0.00, 0.5)),
                    SettingsButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.00, 0.00, 0.5)),
                    CreditsButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Credits"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.75, 0.25, 0.25)),
                    QuitButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quit"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&PlayButton>,
            Option<&QuitButton>,
            Option<&TutorialButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, play_btn, quit_btn, tutorial_btn) in &mut interaction_query {
        let (normal_color, hover_color, pressed_color) = if play_btn.is_some() {
            (
                Color::srgb(0.25, 0.75, 0.25),
                Color::srgb(0.45, 0.85, 0.45),
                Color::srgb(0.35, 0.65, 0.35),
            )
        } else if quit_btn.is_some() {
            (
                Color::srgb(0.75, 0.25, 0.25),
                Color::srgb(0.85, 0.45, 0.45),
                Color::srgb(0.65, 0.35, 0.35),
            )
        } else if tutorial_btn.is_some() {
            (
                Color::srgb(0.2, 0.4, 0.8),
                Color::srgb(0.4, 0.6, 0.9),
                Color::srgb(0.3, 0.5, 0.7),
            )
        } else {
            (
                Color::srgb(0.2, 0.2, 0.2),
                Color::srgb(0.35, 0.35, 0.35),
                Color::srgb(0.15, 0.15, 0.15),
            )
        };

        match *interaction {
            Interaction::Pressed => *color = pressed_color.into(),
            Interaction::Hovered => *color = hover_color.into(),
            Interaction::None => *color = normal_color.into(),
        }
    }
}

fn handle_play_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("State change: MainMenu -> InGame");
            next_state.set(GameState::InGame);
        }
    }
}

fn handle_quit_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            exit_writer.write(AppExit::Success);
        }
    }
}

fn handle_credits_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CreditsButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("State change: MainMenu -> Credits");
            next_state.set(GameState::Credits);
        }
    }
}

fn handle_tutorial_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<TutorialButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("State change: MainMenu -> Tutorial");
            next_state.set(GameState::Tutorial);
        }
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

fn handle_settings_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("State change: MainMenu -> Settings");
            next_state.set(GameState::Settings);
        }
    }
}