use bevy::prelude::*;
use crate::{
    GameState,
    simulation::{ControlSettings, ReactorState, TurbineState, EnvironmentState},
    ui::{indicators::gauge_grid, sliders::slider_panel},
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
struct HighlightBox;

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
                    // Gauges (left bottom)
                    game_ui.spawn(gauge_grid(font.clone()));

                    // Sliders (right bottom)
                    game_ui.spawn(Node {
                        width: Val::Px(420.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(40.0),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            slider_panel(0.0, 0.0, font.clone(), &asset_server),
                            TutorialReactivitySliderMarker,
                            TutorialTurbineSliderMarker,
                        ));
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

    // Highlight overlay (initially invisible)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(0.0),
            height: Val::Px(0.0),
            border: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BorderRadius::all(Val::Px(12.0)),
        BorderColor::all(Color::srgb(1.0, 1.0, 0.4)),
        BackgroundColor(Color::srgba(1.0, 1.0, 0.4, 0.1)),
        HighlightBox,
        DespawnOnExit(GameState::Tutorial),
    ));
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
    mut highlight_query: Query<(&mut Node, &mut BorderColor, &mut BackgroundColor), With<HighlightBox>>,
    gauge_grid: Query<&GlobalTransform, With<TutorialGaugeGridMarker>>,
    reactivity_slider: Query<&GlobalTransform, With<TutorialReactivitySliderMarker>>,
    turbine_slider: Query<&GlobalTransform, With<TutorialTurbineSliderMarker>>,
) {
    if !tutorial_state.is_changed() {
        return;
    }

    let mut highlight_iter = highlight_query.iter_mut();
    let Some((mut node, mut border_color, mut bg_color)) = highlight_iter.next() else {
        return;
    };

    // Default: hide highlight
    node.width = Val::Px(0.0);
    node.height = Val::Px(0.0);
    border_color.set_all(Color::NONE);
    bg_color.0 = Color::NONE;

    // Show highlight based on step
    let target_transform = match tutorial_state.step_index {
        1 | 2 => {
            let mut iter = gauge_grid.iter();
            iter.next()
        }
        3 => {
            let mut iter = gauge_grid.iter();
            iter.next()
        }
        4 => {
            let mut iter = reactivity_slider.iter();
            iter.next()
        }
        5 => {
            let mut iter = turbine_slider.iter();
            iter.next()
        }
        _ => None,
    };

    if let Some(transform) = target_transform {
        let translation = transform.translation();
        
        // Set highlight box based on target
        let (width, height, offset_x, offset_y) = match tutorial_state.step_index {
            1 | 2 | 3 => (280.0, 280.0, -10.0, -10.0), // Gauge grid
            4 | 5 => (440.0, 100.0, -10.0, -10.0), // Sliders
            _ => (0.0, 0.0, 0.0, 0.0),
        };

        node.left = Val::Px(translation.x + offset_x);
        node.top = Val::Px(translation.y + offset_y);
        node.width = Val::Px(width);
        node.height = Val::Px(height);
        border_color.set_all(Color::srgb(1.0, 1.0, 0.4));
        bg_color.0 = Color::srgba(1.0, 1.0, 0.4, 0.1);
    }
}





