use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex},
    picking::hover::Hovered,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{
        CoreSliderDragState, Slider, SliderRange, SliderThumb, SliderValue, TrackClick,
        ValueChange, observe,
    },
};

use crate::{
    GameState,
    simulation::{ControlSettings, EnvironmentState, ReactorState, TurbineState},
};

pub struct ReactorUiPlugin;

impl Plugin for ReactorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui).add_systems(
            Update,
            (
                sync_slider_values,
                update_slider_visuals.after(sync_slider_values),
                update_slider_value_text,
                update_indicators,
                update_game_over_overlay,
            ),
        );
    }
}

const SLIDER_TRACK: Color = Color::srgb(0.18, 0.2, 0.26);
const SLIDER_THUMB: Color = Color::srgb(0.95, 0.55, 0.2);
const SLIDER_THUMB_HOVERED: Color = Color::srgb(1.0, 0.65, 0.3);

#[derive(Component)]
struct ReactorSlider;

#[derive(Component)]
struct ReactivitySlider;

#[derive(Component)]
struct TurbineSlider;

#[derive(Component)]
struct ReactorTempIndicator;

#[derive(Component)]
struct ReactorPressureIndicator;

#[derive(Component)]
struct TurbineTempIndicator;

#[derive(Component)]
struct RadiationIndicator;

#[derive(Component)]
struct GameOverBanner;

#[derive(Component)]
struct SliderThumbVisual;

#[derive(Component)]
struct ReactivityValueText;

#[derive(Component)]
struct TurbineValueText;

fn base_slider(initial_value: f32) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Center,
            height: Val::Px(20.0),
            width: Val::Percent(100.0),
            ..default()
        },
        Name::new("Slider"),
        Hovered::default(),
        ReactorSlider,
        Slider {
            track_click: TrackClick::Snap,
        },
        SliderValue(initial_value),
        SliderRange::new(0.0, 100.0),
        TabIndex(0),
        Children::spawn((
            Spawn((
                Node {
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(SLIDER_TRACK),
                BorderRadius::all(Val::Px(4.0)),
            )),
            Spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(16.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                children![(
                    SliderThumb,
                    SliderThumbVisual,
                    Node {
                        display: Display::Flex,
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(0.0),
                        ..default()
                    },
                    BorderRadius::MAX,
                    BackgroundColor(SLIDER_THUMB),
                )],
            )),
        )),
    )
}

fn setup_ui(
    mut commands: Commands,
    controls: Res<ControlSettings>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2d);

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    // Main UI root
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(30.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
        TabGroup::default(),
        children![
            // Title
            (
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    right: Val::Px(20.0),
                    ..default()
                },
            ),
            // Price display
            (
                Text::new("A$16"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    right: Val::Px(20.0),
                    ..default()
                },
            ),
            // Bottom section with gauges and sliders
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::End,
                    ..default()
                },
                children![
                    // Left side - Gauges
                    gauge_grid(font.clone()),
                    // Right side - Sliders
                    slider_panel(
                        controls.reactivity_target,
                        controls.turbine_target,
                        font.clone(),
                        &asset_server
                    ),
                ],
            ),
        ],
    ));

    // Game Over overlay
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        Visibility::Hidden,
        GlobalZIndex(100),
        GameOverBanner,
        children![(
            Text::new("GAME OVER"),
            TextFont {
                font: font.clone(),
                font_size: 72.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.2, 0.2)),
        )],
    ));
}

fn gauge_grid(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::auto(), GridTrack::auto()],
            grid_template_rows: vec![GridTrack::auto(), GridTrack::auto()],
            column_gap: Val::Px(30.0),
            row_gap: Val::Px(30.0),
            ..default()
        },
        children![
            gauge(
                "Temperatura (Reaktor)",
                "0°C",
                ReactorTempIndicator,
                font.clone()
            ),
            gauge(
                "Ciśnienie (Reaktor)",
                "0 bar",
                ReactorPressureIndicator,
                font.clone()
            ),
            gauge(
                "Temperatura (Turbina)",
                "0°C",
                TurbineTempIndicator,
                font.clone()
            ),
            gauge("Promieniowanie", "0 mSv/h", RadiationIndicator, font.clone()),
        ],
    )
}

fn gauge(
    title: &str,
    initial_value: &str,
    marker: impl Component,
    font: Handle<Font>,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..default()
        },
        children![
            // Title
            (
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ),
            // Gauge visual (simplified circle)
            (
                Node {
                    width: Val::Px(100.0),
                    height: Val::Px(100.0),
                    border: UiRect::all(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::MAX,
                BorderColor::all(Color::srgb(0.2, 0.8, 0.2)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                children![(
                    Text::new(initial_value),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    marker,
                )],
            ),
        ],
    )
}

fn slider_panel(
    reactivity_value: f32,
    turbine_value: f32,
    font: Handle<Font>,
    asset_server: &AssetServer,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(40.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        children![
            // Reactivity slider
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    // Nuclear icon
                    (
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                        ImageNode::new(asset_server.load("imgs/nuclear.png")),
                    ),
                    // Slider container
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            width: Val::Px(300.0),
                            ..default()
                        },
                        children![
                            (
                                Text::new("Reaktywność"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ),
                            create_reactivity_slider(reactivity_value),
                        ],
                    ),
                    // Value display
                    (
                        Text::new(format!("{}%", reactivity_value as i32)),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ReactivityValueText,
                        Node {
                            width: Val::Px(60.0),
                            justify_content: JustifyContent::End,
                            ..default()
                        },
                    ),
                ],
            ),
            // Turbine slider
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    // Turbine icon
                    (
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                        ImageNode::new(asset_server.load("imgs/turbine.png")),
                    ),
                    // Slider container
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            width: Val::Px(300.0),
                            ..default()
                        },
                        children![
                            (
                                Text::new("Moc Turbiny"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ),
                            create_turbine_slider(turbine_value),
                        ],
                    ),
                    // Value display
                    (
                        Text::new(format!("{}%", turbine_value as i32)),
                        TextFont {
                            font: font.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TurbineValueText,
                        Node {
                            width: Val::Px(60.0),
                            justify_content: JustifyContent::End,
                            ..default()
                        },
                    ),
                ],
            ),
        ],
    )
}

fn create_reactivity_slider(initial_value: f32) -> impl Bundle {
    (
        ReactivitySlider,
        base_slider(initial_value),
        observe(
            |value_change: On<ValueChange<f32>>, mut controls: ResMut<ControlSettings>| {
                info!("Reactivity slider changed: {}", value_change.value);
                controls.reactivity_target = value_change.value;
            },
        ),
    )
}

fn create_turbine_slider(initial_value: f32) -> impl Bundle {
    (
        TurbineSlider,
        base_slider(initial_value),
        observe(
            |value_change: On<ValueChange<f32>>, mut controls: ResMut<ControlSettings>| {
                info!("Turbine slider changed: {}", value_change.value);
                controls.turbine_target = value_change.value;
            },
        ),
    )
}

fn sync_slider_values(
    controls: Res<ControlSettings>,
    reactivity_sliders: Query<(Entity, &SliderValue), With<ReactivitySlider>>,
    turbine_sliders: Query<(Entity, &SliderValue), With<TurbineSlider>>,
    mut commands: Commands,
) {
    if controls.is_changed() {
        for (entity, value) in reactivity_sliders.iter() {
            if value.0 != controls.reactivity_target {
                commands
                    .entity(entity)
                    .insert(SliderValue(controls.reactivity_target));
            }
        }
        for (entity, value) in turbine_sliders.iter() {
            if value.0 != controls.turbine_target {
                commands
                    .entity(entity)
                    .insert(SliderValue(controls.turbine_target));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_slider_visuals(
    sliders: Query<
        (
            Entity,
            &SliderValue,
            &SliderRange,
            &Hovered,
            &CoreSliderDragState,
            Has<InteractionDisabled>,
            Has<ReactivitySlider>,
            Has<TurbineSlider>,
        ),
        (
            Or<(
                Changed<SliderValue>,
                Changed<SliderRange>,
                Changed<Hovered>,
                Changed<CoreSliderDragState>,
                Added<InteractionDisabled>,
            )>,
            With<ReactorSlider>,
        ),
    >,
    children: Query<&Children>,
    mut thumbs: Query<
        (&mut Node, &mut BackgroundColor, Has<SliderThumbVisual>),
        Without<ReactorSlider>,
    >,
) {
    for (slider_ent, value, range, hovered, drag_state, disabled, is_reactivity, is_turbine) in
        sliders.iter()
    {
        let slider_type = if is_reactivity {
            "Reactivity"
        } else if is_turbine {
            "Turbine"
        } else {
            "Unknown"
        };
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, mut thumb_bg, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                let new_pos = Val::Percent(range.thumb_position(value.0) * 100.0);
                thumb_node.left = new_pos;

                let is_active = hovered.0 || drag_state.dragging;
                thumb_bg.0 = if disabled {
                    Color::srgb(0.5, 0.5, 0.5)
                } else if is_active {
                    SLIDER_THUMB_HOVERED
                } else {
                    SLIDER_THUMB
                };
            }
        }
    }
}

fn update_slider_value_text(
    controls: Res<ControlSettings>,
    mut reactivity_texts: Query<&mut Text, (With<ReactivityValueText>, Without<TurbineValueText>)>,
    mut turbine_texts: Query<&mut Text, (With<TurbineValueText>, Without<ReactivityValueText>)>,
) {
    if !controls.is_changed() {
        return;
    }

    for mut text in reactivity_texts.iter_mut() {
        **text = format!("{}%", controls.reactivity_target as i32);
    }

    for mut text in turbine_texts.iter_mut() {
        **text = format!("{}%", controls.turbine_target as i32);
    }
}

fn update_indicators(
    reactor: Res<ReactorState>,
    turbine: Res<TurbineState>,
    environment: Res<EnvironmentState>,
    mut texts: Query<&mut Text>,
    reactor_temp: Query<Entity, With<ReactorTempIndicator>>,
    reactor_pressure: Query<Entity, With<ReactorPressureIndicator>>,
    turbine_temp: Query<Entity, With<TurbineTempIndicator>>,
    radiation: Query<Entity, With<RadiationIndicator>>,
) {
    if !(reactor.is_changed() || turbine.is_changed() || environment.is_changed()) {
        return;
    }

    for entity in reactor_temp.iter() {
        if let Ok(mut text) = texts.get_mut(entity) {
            **text = format!("{:.0}°C", reactor.temperature);
        }
    }

    for entity in reactor_pressure.iter() {
        if let Ok(mut text) = texts.get_mut(entity) {
            **text = format!("{:.0} RPM", reactor.pressure * 10.0);
        }
    }

    for entity in turbine_temp.iter() {
        if let Ok(mut text) = texts.get_mut(entity) {
            **text = format!("{:.0}°C", turbine.temperature);
        }
    }

    for entity in radiation.iter() {
        if let Ok(mut text) = texts.get_mut(entity) {
            **text = format!("{:.0} Rad", environment.radiation);
        }
    }
}

fn update_game_over_overlay(
    state: Res<State<GameState>>,
    mut query: Query<&mut Visibility, With<GameOverBanner>>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = if matches!(state.get(), GameState::GameOver) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
