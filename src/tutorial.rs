use crate::{
    FONT_REGULAR, GameState,
    simulation::{ControlSettings, EnvironmentState, ReactorState, TurbineState},
    ui::{
        PauseState,
        sliders::{ReactivitySlider, TurbineSlider},
    },
};
use bevy::prelude::*;
use rand::Rng;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Tutorial),
            (setup_tutorial_scene, play_tutorial_sound),
        )
        .add_systems(
            Update,
            (
                add_tutorial_highlights,
                advance_tutorial_on_space,
                handle_tutorial_escape,
                update_tutorial_ui,
                update_highlight_box,
            )
                .run_if(in_state(GameState::Tutorial)),
        );
    }
}

#[derive(Resource, Default)]
pub struct TutorialState {
    pub step_index: usize,
}

/// Marker for Uranek talking audio so we can ensure only one exists at a time
#[derive(Component)]
struct UranekTalkingAudio;

#[derive(Component)]
struct TutorialSpeechBubble;

#[derive(Component)]
struct TutorialText;

#[derive(Component)]
struct TutorialHelpText;

// New: highlight components for tutorial focus frames
#[derive(Component)]
struct TutorialHighlight {
    kind: HighlightKind,
}

enum HighlightKind {
    Gauge,
    Reactivity,
    Turbine,
    RefuelButton,
    UpgradeButton,
}

fn setup_tutorial_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Reset game state
    commands.insert_resource(ControlSettings::default());
    commands.insert_resource(ReactorState::default());
    commands.insert_resource(TurbineState::default());
    commands.insert_resource(EnvironmentState::default());
    commands.insert_resource(TutorialState::default());

    let font = asset_server.load(FONT_REGULAR);

    // Load Uranek spritesheet for tutorial
    let spritesheet_texture = asset_server.load("sprites/spritesheet.png");
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(1857, 2241));
    layout.add_texture(URect::new(0, 0, 928, 1120)); // talk
    let idle_idx = layout.add_texture(URect::new(929, 0, 1857, 1120)); // idle
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Uranek companion container with tutorial speech bubble
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                align_items: AlignItems::End,
                ..default()
            },
            DespawnOnExit(GameState::Tutorial),
        ))
        .with_children(|parent| {
            // Uranek sprite
            parent.spawn((
                ImageNode {
                    image: spritesheet_texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: texture_atlas_layout,
                        index: idle_idx,
                    }),
                    ..default()
                },
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(220.0),
                    ..default()
                },
            ));
            
            // Tutorial speech bubble
            parent.spawn((
                Node {
                    width: Val::Px(500.0),
                    height: Val::Auto,
                    padding: UiRect::all(Val::Px(20.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(12.0)),
                BorderColor::all(Color::srgb(0.9, 0.9, 0.3)),
                BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.95)),
                TutorialSpeechBubble,
            ))
            .with_children(|bubble| {
                bubble.spawn((
                    Text::new("Cześć! Jestem URANEK, twój ulubiony operator reaktora.\n\nNaciśnij [SPACJA], żeby zacząć."),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    TutorialText,
                ));
            });
        });

    // Help text overlay
    commands.spawn((
        Text::new("[SPACJA] - dalej\n[ESC] - powrót do menu"),
        TextFont {
            font: font.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        TutorialHelpText,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            right: Val::Px(40.0),
            ..default()
        },
        DespawnOnExit(GameState::Tutorial),
    ));
}

fn advance_tutorial_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tutorial_state: ResMut<TutorialState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_audio_q: Query<Entity, With<UranekTalkingAudio>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    // advance step
    tutorial_state.step_index += 1;

    // despawn any previous talking audio so it doesn't multiply
    for entity in existing_audio_q.iter() {
        commands.entity(entity).despawn();
    }

    // play single-shot talking audio for this step
    let random_num = rand::rng().random_range(1..=10);
    let sound_path = format!("sound/talking/talking_{:03}.mp3", random_num);
    let handle: Handle<AudioSource> = asset_server.load(sound_path);
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings::ONCE,
        UranekTalkingAudio,
        DespawnOnExit(GameState::Tutorial),
    ));

    // Exit after final step
    if tutorial_state.step_index > 12 {
        next_state.set(GameState::InGame);
    }
}

fn handle_tutorial_escape(
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

fn update_tutorial_ui(
    tutorial_state: Res<TutorialState>,
    mut text_query: Query<&mut Text, With<TutorialText>>,
) {
    if !tutorial_state.is_changed() {
        return;
    }

    let mut text_iter = text_query.iter_mut();
    let Some(mut text) = text_iter.next() else {
        return;
    };

    let new_text = match tutorial_state.step_index {
        0 => {
            "Cześć! Jestem URANEK.\n\nPracuję jako operator reaktora już od 12 godzin bez przerwy.\nTy będziesz mi pomagać, zanim przerobimy się na żarówkę świetlną."
        }
        1 => {
            "Najpierw REAKTOR.\n\nTutaj grzeje się paliwo. Zbyt zimny reaktor = zero kasy.\nZa gorący = grill all-inclusive."
        }
        2 => {
            "Ten pierwszy okrągły wskaźnik to temperatura REAKTORA.\n\nTrzymaj ją raczej w zielono-żółtej strefie."
        }
        3 => "Ten drugi wskaźnik to ciśnienie REAKTORA.\n\nZa wysokie = kabum!",
        4 => "Ten trzeci wskaźnik to temperatura TURBINY.\n\nOna robi z gorącej wody pieniądze.",
        5 => "Ten czwarty wskaźnik to poziom paliwa.\n\nPaliwo rozpada się z czasem - to rozpad promieniotwórczy.",
        6 => {
            "Paliwo ma PÓŁOKRES ROZPADU około 3 minuty.\n\nZa mało paliwa = słabsza reakcja i wolniejsze ogrzewanie."
        }
        7 => {
            "Żeby uzupełnić paliwo, użyj tego przycisku.\n\nKosztuje $250, ale daje świeże pręty paliwowe.\nUWAGA: można tylko przy reaktywności i turbinie = 0%, i gdy paliwo < 90%!"
        }
        8 => {
            "Ten przycisk to UPGRADE TURBINY.\n\nZa $500 zwiększasz maksymalną temperaturę turbiny z 290°C do 350°C."
        }
        9 => {
            "Suwak REAKTYWNOŚCI steruje, jak mocno reaktor się rozgrzewa.\n\nW grze będziesz nim delikatnie kręcić."
        }
        10 => {
            "Suwak TURBINY reguluje przepływ.\n\nWięcej przepływu = więcej mocy, ale też cieplejsza turbina."
        }
        11 => {
            "Pamiętaj o celu!\n\nMasz przetrwać jak najdłużej i zarobić jak najwięcej kasy.\nZobaczysz swoje pieniądze w prawym górnym rogu."
        }
        12 => {
            "I to tyle z teorii! Teraz przejdziemy do prawdziwej zmiany.\n\nNaciśnij [SPACJA], żeby odpalić prawdziwy reaktor."
        }
        _ => "Gotowy na prawdziwy reaktor?",
    };

    **text = new_text.to_string();
}

fn play_tutorial_sound(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_audio_q: Query<Entity, With<UranekTalkingAudio>>,
) {
    // despawn any previous talking audio so it doesn't multiply
    for entity in existing_audio_q.iter() {
        commands.entity(entity).despawn();
    }

    // Play speaking sound
    let random_num = rand::rng().random_range(1..=10);
    let sound_path = format!("sound/talking/talking_{:03}.mp3", random_num);
    let handle: Handle<AudioSource> = asset_server.load(sound_path);
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings::ONCE,
        UranekTalkingAudio,
        DespawnOnExit(GameState::Tutorial),
    ));
}

#[allow(clippy::too_many_arguments)]
fn add_tutorial_highlights(
    mut commands: Commands,
    reactivity_slider_query: Query<Entity, (With<ReactivitySlider>, Without<TutorialHighlight>)>,
    turbine_slider_query: Query<Entity, (With<TurbineSlider>, Without<TutorialHighlight>)>,
    gauge_border_query: Query<
        Entity,
        (
            With<crate::ui::indicators::GaugeBorder>,
            Without<TutorialHighlight>,
        ),
    >,
    refuel_button_query: Query<Entity, (With<crate::ui::UpgradeButton>, Without<TutorialHighlight>)>,
    upgrade_button_query: Query<Entity, (With<crate::ui::TurbineUpgradeButton>, Without<TutorialHighlight>)>,
    children: Query<&Children>,
    all_entities: Query<Entity>,
    existing_highlights: Query<Entity, With<TutorialHighlight>>,
) {
    // Check if we already have all 5 highlights (gauge, reactivity, turbine, refuel, upgrade)
    if existing_highlights.iter().count() >= 5 {
        return;
    }

    // Check if UI components exist
    if gauge_border_query.is_empty()
        || reactivity_slider_query.is_empty()
        || turbine_slider_query.is_empty()
        || refuel_button_query.is_empty()
        || upgrade_button_query.is_empty()
    {
        return; // UI not ready yet, try again next frame
    }

    // Add highlight overlay to gauge grid container
    // Find a gauge border and traverse up to find the grid container
    if let Some(gauge_entity) = gauge_border_query.iter().next() {
        // Find parent containers by checking which entities have this as a child
        for entity in all_entities.iter() {
            if let Ok(children_list) = children.get(entity)
                && children_list.contains(&gauge_entity)
            {
                // This is the gauge container, find its parent (the grid)
                let gauge_container = entity;
                for grid_entity in all_entities.iter() {
                    if let Ok(grid_children) = children.get(grid_entity)
                        && grid_children.contains(&gauge_container)
                    {
                        // This is the grid container, add highlight
                        commands.entity(grid_entity).with_children(|grid| {
                            grid.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(-16.0),
                                    right: Val::Px(-16.0),
                                    top: Val::Px(-16.0),
                                    bottom: Val::Px(-16.0),
                                    border: UiRect::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(12.0)),
                                BorderColor::all(Color::NONE),
                                BackgroundColor(Color::srgba(1.0, 1.0, 0.3, 0.0)),
                                TutorialHighlight {
                                    kind: HighlightKind::Gauge,
                                },
                                DespawnOnExit(GameState::Tutorial),
                            ));
                        });
                        break; // Found and added gauge highlight
                    }
                }
                break;
            }
        }
    }

    // Add highlight overlay to reactivity slider row
    if let Some(slider_entity) = reactivity_slider_query.iter().next() {
        // Find the parent container, then find the row (grandparent)
        for entity in all_entities.iter() {
            if let Ok(children_list) = children.get(entity)
                && children_list.contains(&slider_entity)
            {
                // This is the slider container, now find its parent (the row)
                for row_entity in all_entities.iter() {
                    if let Ok(row_children) = children.get(row_entity)
                        && row_children.contains(&entity)
                    {
                        // This is the row container, add highlight
                        commands.entity(row_entity).with_children(|row| {
                            row.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(-16.0),
                                    right: Val::Px(-16.0),
                                    top: Val::Px(-16.0),
                                    bottom: Val::Px(-16.0),
                                    border: UiRect::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(12.0)),
                                BorderColor::all(Color::NONE),
                                BackgroundColor(Color::srgba(1.0, 1.0, 0.3, 0.0)),
                                TutorialHighlight {
                                    kind: HighlightKind::Reactivity,
                                },
                                DespawnOnExit(GameState::Tutorial),
                            ));
                        });
                        break; // Found and added reactivity highlight
                    }
                }
                break;
            }
        }
    }

    // Add highlight overlay to turbine slider row
    if let Some(slider_entity) = turbine_slider_query.iter().next() {
        // Find the parent container, then find the row (grandparent)
        for entity in all_entities.iter() {
            if let Ok(children_list) = children.get(entity)
                && children_list.contains(&slider_entity)
            {
                // This is the slider container, now find its parent (the row)
                for row_entity in all_entities.iter() {
                    if let Ok(row_children) = children.get(row_entity)
                        && row_children.contains(&entity)
                    {
                        // This is the row container, add highlight
                        commands.entity(row_entity).with_children(|row| {
                            row.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(-16.0),
                                    right: Val::Px(-16.0),
                                    top: Val::Px(-16.0),
                                    bottom: Val::Px(-16.0),
                                    border: UiRect::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(12.0)),
                                BorderColor::all(Color::NONE),
                                BackgroundColor(Color::srgba(1.0, 1.0, 0.3, 0.0)),
                                TutorialHighlight {
                                    kind: HighlightKind::Turbine,
                                },
                                DespawnOnExit(GameState::Tutorial),
                            ));
                        });
                        break; // Found and added turbine highlight
                    }
                }
                break;
            }
        }
    }

    // Add highlight overlay to refuel button
    if let Some(button_entity) = refuel_button_query.iter().next() {
        commands.entity(button_entity).with_children(|button| {
            button.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-16.0),
                    right: Val::Px(-16.0),
                    top: Val::Px(-16.0),
                    bottom: Val::Px(-16.0),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(12.0)),
                BorderColor::all(Color::NONE),
                BackgroundColor(Color::srgba(1.0, 1.0, 0.3, 0.0)),
                TutorialHighlight {
                    kind: HighlightKind::RefuelButton,
                },
                DespawnOnExit(GameState::Tutorial),
            ));
        });
    }

    // Add highlight overlay to upgrade button
    if let Some(button_entity) = upgrade_button_query.iter().next() {
        commands.entity(button_entity).with_children(|button| {
            button.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-16.0),
                    right: Val::Px(-16.0),
                    top: Val::Px(-16.0),
                    bottom: Val::Px(-16.0),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(12.0)),
                BorderColor::all(Color::NONE),
                BackgroundColor(Color::srgba(1.0, 1.0, 0.3, 0.0)),
                TutorialHighlight {
                    kind: HighlightKind::UpgradeButton,
                },
                DespawnOnExit(GameState::Tutorial),
            ));
        });
    }
}

fn update_highlight_box(
    tutorial_state: Res<TutorialState>,
    time: Res<Time>,
    mut highlight_q: Query<(&mut BackgroundColor, &mut BorderColor, &TutorialHighlight)>,
) {
    let t = (time.elapsed_secs() * 3.0).sin().abs();
    let alpha = 0.20 + 0.10 * t;
    let border_alpha = 0.80 + 0.20 * t;

    let step = tutorial_state.step_index;

    for (mut bg, mut border, highlight) in highlight_q.iter_mut() {
        let active = match (step, &highlight.kind) {
            // Gauges: glow while he talks about each gauge (steps 2-6)
            (2..=6, HighlightKind::Gauge) => true,
            // Refuel button: when he explains the refuel mechanism
            (7, HighlightKind::RefuelButton) => true,
            // Upgrade button: when he explains the turbine upgrade
            (8, HighlightKind::UpgradeButton) => true,
            // Reactivity slider: when he explains the reactivity control
            (9, HighlightKind::Reactivity) => true,
            // Turbine slider: when he explains the turbine control
            (10, HighlightKind::Turbine) => true,
            _ => false,
        };

        let (fill_a, border_a) = if active {
            (alpha, border_alpha)
        } else {
            (0.0, 0.0)
        };

        *bg = BackgroundColor(Color::srgba(1.0, 1.0, 0.3, fill_a));
        border.top = Color::srgba(1.0, 1.0, 0.3, border_a);
        border.right = Color::srgba(1.0, 1.0, 0.3, border_a);
        border.bottom = Color::srgba(1.0, 1.0, 0.3, border_a);
        border.left = Color::srgba(1.0, 1.0, 0.3, border_a);
    }
}
