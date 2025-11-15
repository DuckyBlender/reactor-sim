use crate::{FONT_MEDIUM, FONT_REGULAR, GameState, sound::AudioSettings, ui::sliders::base_slider};
use bevy::{
    prelude::*,
    ui_widgets::{SliderValue, ValueChange, observe},
};

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct SettingsUI;

#[derive(Component)]
struct VolumeSlider;

#[derive(Component)]
struct VolumeText;

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Settings), setup_settings)
            .add_systems(
                Update,
                sync_volume_slider.run_if(in_state(GameState::Settings)),
            )
            .add_systems(Update, update_volume_text)
            .add_systems(
                Update,
                (handle_back_button).run_if(in_state(GameState::Settings)),
            );
    }
}

fn handle_back_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("State change: Settings -> MainMenu");
            next_state.set(GameState::MainMenu);
        }
    }
}

fn update_volume_text(settings: Res<AudioSettings>, mut texts: Query<&mut Text, With<VolumeText>>) {
    if !settings.is_changed() {
        return;
    }

    for mut text in texts.iter_mut() {
        **text = format!("{}%", (settings.volume * 100.0) as i32);
    }
}

fn sync_volume_slider(
    settings: Res<AudioSettings>,
    sliders: Query<(Entity, &SliderValue), With<VolumeSlider>>,
    mut commands: Commands,
) {
    if settings.is_changed() {
        for (entity, value) in sliders.iter() {
            if value.0 != settings.volume {
                commands.entity(entity).insert(SliderValue(settings.volume));
            }
        }
    }
}

fn create_volume_slider(initial_value: f32) -> impl Bundle {
    (
        VolumeSlider,
        base_slider(initial_value, 1.0),
        observe(
            |value_change: On<ValueChange<f32>>, mut settings: ResMut<AudioSettings>| {
                settings.volume = value_change.value;
            },
        ),
    )
}

fn setup_settings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
) {
    let font = asset_server.load(FONT_REGULAR);
    let font_medium = asset_server.load(FONT_MEDIUM);

    commands.spawn((Camera2d, DespawnOnExit(GameState::Settings)));
    commands
        .spawn((
            DespawnOnExit(GameState::Settings),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(32.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            SettingsUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Ustawienia"),
                TextFont {
                    font: font_medium.clone(),
                    font_size: 32.0,
                    ..default()
                },
            ));

            parent
                .spawn((Node {
                    width: Val::Percent(50.0),
                    margin: UiRect {
                        top: Val::Px(32.0),
                        ..default()
                    },
                    row_gap: Val::Px(12.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Głośność"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                    ));

                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(create_volume_slider(settings.volume));
                            parent.spawn((
                                Text::new(format!("{}%", (settings.volume * 100.0) as u8)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 24.0,
                                    ..default()
                                },
                                VolumeText,
                            ));
                        });

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
                        .with_children(|parent| {
                            parent.spawn((
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
        });
}
