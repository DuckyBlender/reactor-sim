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

const SLIDER_HEIGHT: f32 = 24.0;
const SLIDER_TRACK_HEIGHT: f32 = 8.0;
const SLIDER_THUMB_SIZE: f32 = 20.0;
const SLIDER_RIGHT_MARGIN: f32 = 20.0;
const SLIDER_PANEL_ROW_GAP: f32 = 32.0;
const SLIDER_PANEL_PADDING: f32 = 24.0;
const SLIDER_ROW_GAP: f32 = 24.0;
const SLIDER_CONTAINER_WIDTH: f32 = 240.0;
const SLIDER_TITLE_FONT_SIZE: f32 = 18.0;
const SLIDER_VALUE_FONT_SIZE: f32 = 20.0;
const SLIDER_INTERNAL_FONT_SIZE: f32 = 14.0;
const SLIDER_ICON_SIZE: f32 = 48.0;
const SLIDER_VALUE_WIDTH: f32 = 60.0;

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

#[derive(Component)]
pub struct ReactivityAppliedText;

#[derive(Component)]
pub struct TurbineAppliedText;

#[derive(Component)]
pub struct TurbineIcon;

pub fn base_slider(initial_value: f32, max: f32) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Center,
            height: Val::Px(SLIDER_HEIGHT),
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
        SliderRange::new(0.0, max),
        TabIndex(0),
        Children::spawn((
            Spawn((
                Node {
                    height: Val::Px(SLIDER_TRACK_HEIGHT),
                    ..default()
                },
                BackgroundColor(SLIDER_TRACK),
                BorderRadius::all(Val::Px(SLIDER_TRACK_HEIGHT / 2.0)),
            )),
            Spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(SLIDER_RIGHT_MARGIN),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                children![(
                    SliderThumb,
                    SliderThumbVisual,
                    Node {
                        display: Display::Flex,
                        width: Val::Px(SLIDER_THUMB_SIZE),
                        height: Val::Px(SLIDER_THUMB_SIZE),
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
            row_gap: Val::Px(SLIDER_PANEL_ROW_GAP),
            padding: UiRect::all(Val::Px(SLIDER_PANEL_PADDING)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        BorderRadius::all(Val::Px(12.0)),
        Transform::default(),
        children![
            // Reactivity slider
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(SLIDER_ROW_GAP),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    // Nuclear icon
                    (
                        Node {
                            width: Val::Px(SLIDER_ICON_SIZE),
                            height: Val::Px(SLIDER_ICON_SIZE),
                            ..default()
                        },
                        ImageNode::new(asset_server.load("imgs/nuclear.png")),
                    ),
                    // Slider container
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            width: Val::Px(SLIDER_CONTAINER_WIDTH),
                            ..default()
                        },
                        children![
                            (
                                Text::new("Reaktywność"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: SLIDER_TITLE_FONT_SIZE,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ),
                            create_reactivity_slider(reactivity_value),
                            (
                                Text::new(format!("Internal: {}%", reactivity_value as i32)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: SLIDER_INTERNAL_FONT_SIZE,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
                                ReactivityAppliedText,
                            ),
                        ],
                    ),
                    // Value display
                    (
                        Text::new(format!("{}%", reactivity_value as i32)),
                        TextFont {
                            font: font.clone(),
                            font_size: SLIDER_VALUE_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ReactivityValueText,
                        Node {
                            width: Val::Px(SLIDER_VALUE_WIDTH),
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
                    column_gap: Val::Px(SLIDER_ROW_GAP),
                    align_items: AlignItems::Center,
                    ..default()
                },
                Transform::default(),
                children![
                    // Turbine icon (spinning)
                    (
                        Node {
                            width: Val::Px(SLIDER_ICON_SIZE),
                            height: Val::Px(SLIDER_ICON_SIZE),
                            ..default()
                        },
                        ImageNode::new(asset_server.load("imgs/turbine.png")),
                        Transform::default(),
                        TurbineIcon,
                    ),
                    // Slider container
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            width: Val::Px(SLIDER_CONTAINER_WIDTH),
                            ..default()
                        },
                        children![
                            (
                                Text::new("Prędkość Transferu"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: SLIDER_TITLE_FONT_SIZE,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ),
                            create_turbine_slider(turbine_value),
                            (
                                Text::new(format!("Internal: {}%", turbine_value as i32)),
                                TextFont {
                                    font: font.clone(),
                                    font_size: SLIDER_INTERNAL_FONT_SIZE,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
                                TurbineAppliedText,
                            ),
                        ],
                    ),
                    // Value display
                    (
                        Text::new(format!("{}%", turbine_value as i32)),
                        TextFont {
                            font,
                            font_size: SLIDER_VALUE_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TurbineValueText,
                        Node {
                            width: Val::Px(SLIDER_VALUE_WIDTH),
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
        base_slider(initial_value, 100.0),
        observe(
            |value_change: On<ValueChange<f32>>, mut controls: ResMut<ControlSettings>| {
                controls.reactivity_target = value_change.value;
            },
        ),
    )
}

pub fn create_turbine_slider(initial_value: f32) -> impl Bundle {
    (
        TurbineSlider,
        base_slider(initial_value, 100.0),
        observe(
            |value_change: On<ValueChange<f32>>, mut controls: ResMut<ControlSettings>| {
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
    for (slider_ent, value, range, hovered, drag_state, disabled) in sliders.iter() {
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

pub fn update_applied_value_text(
    controls: Res<ControlSettings>,
    mut reactivity_applied_texts: Query<
        &mut Text,
        (With<ReactivityAppliedText>, Without<TurbineAppliedText>),
    >,
    mut turbine_applied_texts: Query<
        &mut Text,
        (With<TurbineAppliedText>, Without<ReactivityAppliedText>),
    >,
) {
    if !controls.is_changed() {
        return;
    }

    for mut text in reactivity_applied_texts.iter_mut() {
        **text = format!("Internal: {:.1}%", controls.reactivity_applied);
    }

    for mut text in turbine_applied_texts.iter_mut() {
        **text = format!("Internal: {:.1}%", controls.turbine_applied);
    }
}

