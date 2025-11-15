use crate::{
    GameState,
    simulation::{ControlSettings, EnvironmentState, ReactorState, TurbineState},
    ui::{indicators::gauge_grid, PauseState},
};
use bevy::prelude::*;
use rand::Rng;



pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Tutorial), (setup_tutorial_scene, play_tutorial_sound))
            .add_systems(
                Update,
                (
                    advance_tutorial_on_space,
                    handle_tutorial_escape,
                    update_tutorial_ui,
                    update_highlight_box,
                    update_uranek_animation,
                ).run_if(in_state(GameState::Tutorial)),
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
struct TutorialCamera;

#[derive(Component)]
struct UranekSprite;

#[derive(Component, Default)]
struct UranekAnimationState {
    timer: Timer,
    is_talking: bool,
}

#[derive(Component)]
struct UranekSpriteIndices {
    talk: usize,
    idle: usize,
    wave: usize,
    #[allow(dead_code)]
    hot: usize,
}

#[derive(Component)]
struct UranekTextBox;

#[derive(Component)]
struct UranekText;

#[derive(Component)]
struct TutorialReactivitySliderMarker;

#[derive(Component)]
struct TutorialTurbineSliderMarker;

#[derive(Component)]
struct TutorialGaugeGridMarker;

// New: highlight components for tutorial focus frames
#[derive(Component)]
struct TutorialHighlight {
    kind: HighlightKind,
}

enum HighlightKind {
    Gauge,
    Reactivity,
    Turbine,
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

    // Camera
    commands.spawn((Camera2d, TutorialCamera, DespawnOnExit(GameState::Tutorial)));

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    // Load spritesheet and create atlas layout
    let spritesheet_texture = asset_server.load("sprites/spritesheet.png");
    
    // Parse sprites.txt to define atlas layout
    // Format: talk,0,0,928,1120; idle,929,0,928,1120; wave,0,1121,928,1120; hot,929,1121,928,1120
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(1857, 2241));
    
    let talk_idx = layout.add_texture(URect::new(0, 0, 928, 1120));
    let idle_idx = layout.add_texture(URect::new(929, 0, 1857, 1120));
    let wave_idx = layout.add_texture(URect::new(0, 1121, 928, 2241));
    let hot_idx = layout.add_texture(URect::new(929, 1121, 1857, 2241));
    
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    
    let sprite_indices = UranekSpriteIndices {
        talk: talk_idx,
        idle: idle_idx,
        wave: wave_idx,
        hot: hot_idx,
    };

    // Root container
    commands
        .spawn((
            DespawnOnExit(GameState::Tutorial),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(40.0)),
                column_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.04, 0.04, 0.08)),
        ))
        .with_children(|root| {
            // LEFT SIDE: Game UI (60%)
            root
                .spawn(Node {
                    width: Val::Percent(35.0),
                    height: Val::Percent(102.50),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::End,
                    column_gap: Val::Px(40.0),
                    ..default()
                })
                .with_children(|game_ui| {
                    // Gauges (left bottom) with highlight
                    game_ui
                        .spawn((
                            Node {
                                position_type: PositionType::Relative,
                                ..default()
                            },
                            TutorialGaugeGridMarker,
                        ))
                        .with_children(|gauge_container| {
                            // Actual gauge grid UI
                            // Assume the first spawned child inside `gauge_grid` corresponds to the main reactor gauge;
                            // we wrap JUST that gauge in a highlight frame instead of the entire grid.
                            gauge_container
                                .spawn(Node {
                                    position_type: PositionType::Relative,
                                    ..default()
                                })
                                .with_children(|single_gauge_wrapper| {
                                    // Highlight behind a single gauge
                                    single_gauge_wrapper.spawn((
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
                                        TutorialHighlight { kind: HighlightKind::Gauge },
                                    ));

                                    // Single gauge node from the grid
                                    single_gauge_wrapper.spawn(gauge_grid(font.clone()));
                                });
                        });

                    // Sliders (right bottom)
                    game_ui
                        .spawn(Node {
                            width: Val::Px(420.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(40.0),
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Reactivity slider row with marker and highlight
                            parent
                                .spawn((
                                    Node {
                                        position_type: PositionType::Relative,
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(20.0),
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    TutorialReactivitySliderMarker,
                                ))
                                .with_children(|row| {
                                    // Highlight behind reactivity slider row
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
                                        TutorialHighlight { kind: HighlightKind::Reactivity },
                                    ));

                                    // Nuclear icon
                                    row.spawn((
                                        Node {
                                            width: Val::Px(48.0),
                                            height: Val::Px(48.0),
                                            ..default()
                                        },
                                        ImageNode::new(asset_server.load("imgs/nuclear.png")),
                                    ));
                                    // Slider container
                                    row.spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(8.0),
                                        width: Val::Px(300.0),
                                        ..default()
                                    })
                                    .with_children(|slider_container| {
                                        slider_container.spawn((
                                            Text::new("Reaktywność"),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 16.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        ));
                                        slider_container.spawn(crate::ui::sliders::create_reactivity_slider(0.0));
                                    });
                                    // Value display
                                    row.spawn((
                                        Text::new("0%"),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        crate::ui::sliders::ReactivityValueText,
                                        Node {
                                            width: Val::Px(60.0),
                                            justify_content: JustifyContent::End,
                                            ..default()
                                        },
                                    ));
                                });

                            // Turbine slider row with marker and highlight
                            parent
                                .spawn((
                                    Node {
                                        position_type: PositionType::Relative,
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(20.0),
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    TutorialTurbineSliderMarker,
                                ))
                                .with_children(|row| {
                                    // Highlight behind turbine slider row
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
                                        TutorialHighlight { kind: HighlightKind::Turbine },
                                    ));

                                    // Turbine icon
                                    row.spawn((
                                        Node {
                                            width: Val::Px(48.0),
                                            height: Val::Px(48.0),
                                            ..default()
                                        },
                                        ImageNode::new(asset_server.load("imgs/turbine.png")),
                                        Transform::default(),
                                        crate::ui::sliders::TurbineIcon,
                                    ));
                                    // Slider container
                                    row.spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(8.0),
                                        width: Val::Px(300.0),
                                        ..default()
                                    })
                                    .with_children(|slider_container| {
                                        slider_container.spawn((
                                            Text::new("Prędkość Turbiny"),
                                            TextFont {
                                                font: font.clone(),
                                                font_size: 16.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        ));
                                        slider_container.spawn(crate::ui::sliders::create_turbine_slider(0.0));
                                    });
                                    // Value display
                                    row.spawn((
                                        Text::new("0%"),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        crate::ui::sliders::TurbineValueText,
                                        Node {
                                            width: Val::Px(60.0),
                                            justify_content: JustifyContent::End,
                                            ..default()
                                        },
                                    ));
                                });
                        });
                });

            // RIGHT SIDE: UraneK + Speech Bubble (40%)
            root
                .spawn(Node {
                    width: Val::Percent(40.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(30.0),
                    ..default()
                })
                .with_children(|right_panel| {
                    // UraneK Sprite - start with wave pose
                    right_panel.spawn((
                        ImageNode {
                            image: spritesheet_texture.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_layout.clone(),
                                index: sprite_indices.wave,
                            }),
                            ..default()
                        },
                        Node {
                            width: Val::Px(256.0),
                            height: Val::Px(256.0),
                            ..default()
                        },
                        UranekSprite,
                        UranekAnimationState {
                            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                            is_talking: false,
                        },
                        sprite_indices,
                    ));

                    // Speech Bubble
                    right_panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(20.0)),
                                border: UiRect::all(Val::Px(3.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(12.0)),
                            BorderColor::all(Color::srgb(0.9, 0.9, 0.3)),
                            BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.95)),
                            UranekTextBox,
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
                                UranekText,
                            ));
                        });

                    // Help text
                    right_panel.spawn((
                        Text::new("[SPACJA] - dalej\n[ESC] - powrót do menu"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });
        });

    // Remove old pointer arrow entity entirely (no longer needed)
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
    if tutorial_state.step_index > 6 {
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
    mut text_query: Query<&mut Text, With<UranekText>>,
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
        3 => {
            "Ten drugi wskaźnik to TURBINA.\n\nOna robi z gorącej wody pieniądze. Za zimna - nie kręci. Za gorąca - kręci się ostatni raz."
        }
        4 => {
            "Suwak REAKTYWNOŚCI steruje, jak mocno reaktor się rozgrzewa.\n\nW grze będziesz nim delikatnie kręcić."
        }
        5 => {
            "Suwak TURBINY reguluje przepływ.\n\nWięcej przepływu = więcej mocy, ale też cieplejsza turbina."
        }
        6 => {
            "I to tyle z teorii! Teraz przejdziemy do prawdziwej zmiany.\n\nNaciśnij [SPACJA], żeby odpalić prawdziwy reaktor."
        }
        _ => "Gotowy na prawdziwy reaktor?",
    };

    **text = new_text.to_string();
}


fn update_uranek_animation(
    tutorial_state: Res<TutorialState>,
    time: Res<Time>,
    mut uranek_query: Query<
        (&mut ImageNode, &mut UranekAnimationState, &UranekSpriteIndices),
        With<UranekSprite>,
    >,
) {
    let Ok((mut image_node, mut anim, indices)) = uranek_query.single_mut() else {
        return;
    };

    let Some(atlas) = &mut image_node.texture_atlas else {
        return;
    };

    let step = tutorial_state.step_index;

    // Step 0: wave pose (greeting)
    if step == 0 {
        atlas.index = indices.wave;
        anim.is_talking = false;
        return;
    }

    // Steps 1..=6: talking - alternate between idle and talk at 5 Hz
    if (1..=6).contains(&step) {
        anim.is_talking = true;
        anim.timer.tick(time.delta());
        
        if anim.timer.just_finished() {
            // Toggle between talk and idle sprites
            atlas.index = if atlas.index == indices.talk {
                indices.idle
            } else {
                indices.talk
            };
        }
        return;
    }

    // After tutorial: just show idle
    atlas.index = indices.idle;
    anim.is_talking = false;
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
            // Gauges: glow only while he explicitly talks about the reactor & turbine gauges
            (2 | 3, HighlightKind::Gauge) => true,
            // Reactivity slider: when he explains the reactivity control
            (4, HighlightKind::Reactivity) => true,
            // Turbine slider: when he explains the turbine control
            (5, HighlightKind::Turbine) => true,
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
