use bevy::{
    input_focus::tab_navigation::TabIndex,
    picking::hover::Hovered,
    prelude::*,
    ui::InteractionDisabled,
    ui_widgets::{
        CoreSliderDragState, Slider, SliderRange, SliderThumb, SliderValue, TrackClick,
        ValueChange, observe,
    },
};

use crate::simulation::ControlSettings;

pub const SLIDER_TRACK: Color = Color::srgb(0.18, 0.2, 0.26);
pub const SLIDER_THUMB: Color = Color::srgb(0.95, 0.55, 0.2);
pub const SLIDER_THUMB_HOVERED: Color = Color::srgb(1.0, 0.65, 0.3);

#[derive(Component)]
pub struct ReactorSlider;

#[derive(Component)]
pub struct ReactivitySlider;

#[derive(Component)]
pub struct TurbineSlider;

#[derive(Component)]
pub struct SliderThumbVisual;

#[derive(Component)]
pub struct ReactivityValueText;

#[derive(Component)]
pub struct TurbineValueText;

pub fn base_slider(initial_value: f32) -> impl Bundle {
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

pub fn slider_panel(
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

pub fn create_reactivity_slider(initial_value: f32) -> impl Bundle {
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

pub fn create_turbine_slider(initial_value: f32) -> impl Bundle {
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

pub fn sync_slider_values(
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
pub fn update_slider_visuals(
    sliders: Query<
        (
            Entity,
            &SliderValue,
            &SliderRange,
            &Hovered,
            &CoreSliderDragState,
            Has<InteractionDisabled>,
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
    for (slider_ent, value, range, hovered, drag_state, disabled) in
        sliders.iter()
    {
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

pub fn update_slider_value_text(
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

