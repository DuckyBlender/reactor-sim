use bevy::prelude::*;
use crate::{
    GameState,
    simulation::{ControlSettings, ReactorState, TurbineState, EnvironmentState},
    ui::indicators::gauge_grid,
};

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Tutorial), setup_tutorial_scene)
            .add_systems(
                Update,
                (
                    advance_tutorial_on_space,
                    update_tutorial_ui,
                    update_highlight_box,
                    update_uranek_animation,
                ).run_if(in_state(GameState::Tutorial)),
            )
            .add_systems(OnExit(GameState::Tutorial), teardown_tutorial_scene);
    }
}

#[derive(Resource, Default)]
pub struct TutorialState {
    pub step_index: usize,
}

#[derive(Component)]
struct TutorialCamera;

#[derive(Component)]
struct UranekSprite;

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

#[derive(Component)]
struct DespawnOnExit(GameState);

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
    let uranek_texture: Handle<Image> = asset_server.load("sprites/sprite.png");

    // Create texture atlas for 2x2 sprite grid
    let layout = TextureAtlasLayout::from_grid(UVec2::new(128, 128), 2, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

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
                            // Highlight behind gauges
                            gauge_container.spawn((
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

                            // Actual gauge grid UI
                            gauge_container.spawn(gauge_grid(font.clone()));
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
                                            Text::new("Prędkość Transferu"),
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
                    // UraneK Sprite
                    right_panel.spawn((
                        ImageNode {
                            image: uranek_texture.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_layout.clone(),
                                index: 0,
                            }),
                            ..default()
                        },
                        Node {
                            width: Val::Px(256.0),
                            height: Val::Px(256.0),
                            ..default()
                        },
                        UranekSprite,
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
                        Text::new("[SPACJA] - dalej"),
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

fn teardown_tutorial_scene(
    mut commands: Commands,
    query: Query<Entity, With<DespawnOnExit>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TutorialState>();
}

fn advance_tutorial_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tutorial_state: ResMut<TutorialState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        tutorial_state.step_index += 1;

        // Exit after final step
        if tutorial_state.step_index > 6 {
            next_state.set(GameState::InGame);
        }
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
        0 => "Cześć! Jestem URANEK.\n\nPracuję jako operator reaktora już od 12 godzin bez kawy.\nTy będziesz mi pomagać, zanim przeróbimy się na żarówkę świetlną.",
        1 => "Najpierw REAKTOR.\n\nTutaj grzeje się paliwo. Zbyt zimny reaktor = zero kasy.\nZa gorący = grill all-inclusive.",
        2 => "Ten okrągły wskaźnik to temperatura REAKTORA.\n\nTrzymaj ją raczej w zielono-żółtej strefie.",
        3 => "A to TURBINA.\n\nOna robi z gorącej wody pieniądze. Za zimna - nie kręci. Za gorąca - kręci się ostatni raz.",
        4 => "Suwak REAKTYWNOŚCI (po prawej) steruje, jak mocno reaktor się rozgrzewa.\n\nW grze będziesz nim delikatnie kręcić.",
        5 => "Suwak TURBINY reguluje przepływ.\n\nWięcej przepływu = więcej mocy, ale też cieplejsza turbina.",
        6 => "I to tyle z teorii! Teraz przejdziemy do prawdziwej zmiany.\n\nNaciśnij [SPACJA], żeby odpalić prawdziwy reaktor.",
        _ => "Gotowy na prawdziwy reaktor?",
    };

    **text = new_text.to_string();
}
fn update_uranek_animation(
    tutorial_state: Res<TutorialState>,
    time: Res<Time>,
    mut uranek_query: Query<&mut ImageNode, With<UranekSprite>>,
) {
    let mut uranek_iter = uranek_query.iter_mut();
    let Some(mut image_node) = uranek_iter.next() else {
        return;
    };

    // Animate uranek sprite based on tutorial state
    let frame = ((time.elapsed_secs() * 2.0) as usize) % 2;
    
    if let Some(ref mut atlas) = image_node.texture_atlas {
        atlas.index = match tutorial_state.step_index {
            0 => frame, // Idle animation (frames 0-1)
            1..=6 => 2 + frame, // Talking animation (frames 2-3)
            _ => 0,
        };
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
            (1 | 2 | 3, HighlightKind::Gauge) => true,
            (4, HighlightKind::Reactivity) => true,
            (5, HighlightKind::Turbine) => true,
            _ => false,
        };

        let (fill_a, border_a) = if active { (alpha, border_alpha) } else { (0.0, 0.0) };

        *bg = BackgroundColor(Color::srgba(1.0, 1.0, 0.3, fill_a));
        border.top = Color::srgba(1.0, 1.0, 0.3, border_a);
        border.right = Color::srgba(1.0, 1.0, 0.3, border_a);
        border.bottom = Color::srgba(1.0, 1.0, 0.3, border_a);
        border.left = Color::srgba(1.0, 1.0, 0.3, border_a);
    }
}





