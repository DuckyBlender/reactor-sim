use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, Button, observe},
};

use crate::simulation::{
    EnvironmentState, REACTOR_TEMP_LIMIT, ReactorState, TURBINE_TEMP_LIMIT, TurbineState,
};

// Use larger base sizes for high DPI displays
const GAUGE_SIZE: f32 = 120.0;
const GAUGE_BORDER: f32 = 8.0;
const GAUGE_TITLE_FONT_SIZE: f32 = 16.0;
const GAUGE_VALUE_FONT_SIZE: f32 = 20.0;
const GAUGE_DURABILITY_FONT_SIZE: f32 = 14.0;
const GAUGE_CONTAINER_GAP: f32 = 8.0;
const GAUGE_GRID_GAP: f32 = 24.0;
const GAUGE_GRID_PADDING: f32 = 20.0;

#[derive(Component)]
pub struct ReactorTempIndicator;

#[derive(Component)]
pub struct ReactorPressureIndicator;

#[derive(Component)]
pub struct TurbineTempIndicator;

#[derive(Component)]
pub struct PowerIndicator;

#[derive(Component)]
pub struct GaugeBorder {
    pub gauge_type: GaugeType,
}

#[derive(Component)]
pub struct TurbineDurabilityText;

#[derive(Component)]
pub struct BlinkTimer(pub Timer);

#[derive(Component)]
pub struct BuyBackButton;

#[derive(Component)]
pub struct TurbineGaugeContainer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaugeType {
    ReactorTemp,
    ReactorPressure,
    TurbineTemp,
    Power,
}

pub fn gauge_grid(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::auto(), GridTrack::auto()],
            grid_template_rows: vec![GridTrack::auto(), GridTrack::auto()],
            column_gap: Val::Px(GAUGE_GRID_GAP),
            row_gap: Val::Px(GAUGE_GRID_GAP),
            padding: UiRect::all(Val::Px(GAUGE_GRID_PADDING)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        BorderRadius::all(Val::Px(12.0)),
        children![
            gauge(
                "Temp. Reaktora",
                "0°C",
                ReactorTempIndicator,
                GaugeType::ReactorTemp,
                font.clone()
            ),
            gauge(
                "Ciśn. Reaktora",
                "0 bar",
                ReactorPressureIndicator,
                GaugeType::ReactorPressure,
                font.clone()
            ),
            turbine_gauge(font.clone()),
            gauge(
                "Moc",
                "0 MW",
                PowerIndicator,
                GaugeType::Power,
                font
            ),
        ],
    )
}

fn turbine_gauge(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(GAUGE_CONTAINER_GAP),
            ..default()
        },
        TurbineGaugeContainer,
        children![
            // Title
            (
                Text::new("Temp.  Turbiny"),
                TextFont {
                    font: font.clone(),
                    font_size: GAUGE_TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    height: Val::Px(24.0), // Match title height
                    ..default()
                },
            ),
            // Gauge visual
            (
                Node {
                    width: Val::Px(GAUGE_SIZE),
                    height: Val::Px(GAUGE_SIZE),
                    border: UiRect::all(Val::Px(GAUGE_BORDER)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                BorderRadius::MAX,
                BorderColor::all(Color::srgb(0.2, 0.8, 0.2)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                GaugeBorder {
                    gauge_type: GaugeType::TurbineTemp
                },
                BlinkTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
                children![
                    (
                        Text::new("0°C"),
                        TextFont {
                            font: font.clone(),
                            font_size: GAUGE_VALUE_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TurbineTempIndicator,
                    ),
                    (
                        Text::new("100%"),
                        TextFont {
                            font,
                            font_size: GAUGE_DURABILITY_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        TurbineDurabilityText,
                    ),
                ],
            ),
        ],
    )
}

fn gauge(
    title: &str,
    initial_value: &str,
    marker: impl Component,
    gauge_type: GaugeType,
    font: Handle<Font>,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(GAUGE_CONTAINER_GAP),
            ..default()
        },
        children![
            // Title
            (
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: GAUGE_TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    height: Val::Px(24.0), // Fixed title height for alignment
                    ..default()
                },
            ),
            // Gauge visual (simplified circle)
            (
                Node {
                    width: Val::Px(GAUGE_SIZE),
                    height: Val::Px(GAUGE_SIZE),
                    border: UiRect::all(Val::Px(GAUGE_BORDER)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::MAX,
                BorderColor::all(Color::srgb(0.2, 0.8, 0.2)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                GaugeBorder { gauge_type },
                BlinkTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
                children![(
                    Text::new(initial_value),
                    TextFont {
                        font,
                        font_size: GAUGE_VALUE_FONT_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    marker,
                )],
            ),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn update_indicators(
    reactor: Res<ReactorState>,
    turbine: Res<TurbineState>,
    environment: Res<EnvironmentState>,
    mut texts: Query<&mut Text>,
    reactor_temp: Query<Entity, With<ReactorTempIndicator>>,
    reactor_pressure: Query<Entity, With<ReactorPressureIndicator>>,
    turbine_temp: Query<Entity, With<TurbineTempIndicator>>,
    power: Query<Entity, With<PowerIndicator>>,
    durability_texts: Query<Entity, With<TurbineDurabilityText>>,
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
            **text = format!("{:.1} bar", reactor.pressure);
        }
    }

    for entity in turbine_temp.iter() {
        if let Ok(mut text) = texts.get_mut(entity) {
            **text = format!("{:.0}°C", turbine.temperature);
        }
    }

    for entity in power.iter() {
        if let Ok(mut text) = texts.get_mut(entity) {
            **text = format!("{:.0} MW", environment.power_generated);
        }
    }

    // Update turbine durability text
    for entity in durability_texts.iter() {
        if let Ok(mut text) = texts.get_mut(entity) {
            **text = format!("{:.0}%", turbine.durability);
        }
    }
}

pub fn update_gauge_colors(
    reactor: Res<ReactorState>,
    turbine: Res<TurbineState>,
    environment: Res<EnvironmentState>,
    mut gauges: Query<(&GaugeBorder, &mut BorderColor, &mut BlinkTimer)>,
    time: Res<Time>,
) {
    for (gauge_border, mut border_color, mut blink_timer) in gauges.iter_mut() {
        // Power gauge always stays green
        if gauge_border.gauge_type == GaugeType::Power {
            border_color.set_all(Color::srgb(0.2, 0.8, 0.2));
            continue;
        }

        let (current_value, limit) = match gauge_border.gauge_type {
            GaugeType::ReactorTemp => (reactor.temperature, REACTOR_TEMP_LIMIT),
            GaugeType::ReactorPressure => (reactor.pressure, 160.0), // bar
            GaugeType::TurbineTemp => (turbine.temperature, TURBINE_TEMP_LIMIT),
            GaugeType::Power => (environment.power_generated, 1000.0), // Max power (unused)
        };

        let percentage = current_value / limit;

        let color = if percentage >= 0.95 {
            // Black/transparent zone (95-100%)
            // Fade from red to black
            let fade = ((percentage - 0.95) / 0.05).clamp(0.0, 1.0);
            let red = 0.9 * (1.0 - fade);
            let green = 0.1 * (1.0 - fade);
            let blue = 0.1 * (1.0 - fade);
            let alpha = 1.0 - fade;
            Color::srgba(red, green, blue, alpha)
        } else if percentage >= 0.80 {
            // Red zone with blinking (80-95%)
            blink_timer.0.tick(time.delta());
            let visible =
                (blink_timer.0.elapsed_secs() / blink_timer.0.duration().as_secs_f32()) % 1.0 < 0.5;

            // Fade from yellow to red
            let fade = ((percentage - 0.80) / 0.15).clamp(0.0, 1.0);
            let red = 0.9;
            let green = 0.9 * (1.0 - fade) + 0.1 * fade;
            let blue = 0.1;

            if visible {
                Color::srgb(red, green, blue)
            } else {
                Color::srgba(0.0, 0.0, 0.0, 0.0)
            }
        } else if percentage >= 0.60 {
            // Yellow zone (60-80%)
            // Fade from green to yellow
            let fade = ((percentage - 0.60) / 0.20).clamp(0.0, 1.0);
            let red = 0.2 + (0.7 * fade);
            let green = 0.8 + (0.1 * fade);
            let blue = 0.2 * (1.0 - fade) + 0.1 * fade;
            Color::srgb(red, green, blue)
        } else {
            // Green zone (0-60%)
            Color::srgb(0.2, 0.8, 0.2)
        };

        border_color.set_all(color);
    }
}

pub fn handle_turbine_destroyed(
    turbine: Res<TurbineState>,
    mut commands: Commands,
    turbine_container: Query<Entity, With<TurbineGaugeContainer>>,
    children: Query<&Children>,
    gauge_borders: Query<(Entity, &GaugeBorder)>,
    asset_server: Res<AssetServer>,
) {
    if !turbine.is_changed() {
        return;
    }

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    for container_entity in turbine_container.iter() {
        // Find the gauge border child
        if let Ok(container_children) = children.get(container_entity) {
            for child in container_children.iter() {
                if let Ok((gauge_entity, gauge_border)) = gauge_borders.get(child)
                    && gauge_border.gauge_type == GaugeType::TurbineTemp
                    && turbine.is_destroyed
                {
                    // Replace gauge with buy-back button
                    commands.entity(gauge_entity).despawn();
                    commands.entity(container_entity).with_child((
                                Node {
                                    width: Val::Px(GAUGE_SIZE),
                                    height: Val::Px(GAUGE_SIZE),
                                    border: UiRect::all(Val::Px(GAUGE_BORDER - 2.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                BorderRadius::MAX,
                                BorderColor::all(Color::srgb(0.9, 0.7, 0.1)),
                                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                                Button,
                                Hovered::default(),
                                BuyBackButton,
                                observe(
                                    |_activate: On<Activate>,
                                     mut turbine: ResMut<TurbineState>,
                                     mut environment: ResMut<EnvironmentState>| {
                                        if environment.money >= 200.0 {
                                            environment.money -= 200.0;
                                            turbine.durability = 100.0;
                                            turbine.is_destroyed = false;
                                        }
                                    },
                                ),
                                children![(
                                    Text::new("Buy Back\n$200"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: GAUGE_TITLE_FONT_SIZE,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                )],
                            ));
                }
            }
        }
    }
}

pub fn rebuild_turbine_gauge_from_buyback(
    turbine: Res<TurbineState>,
    mut commands: Commands,
    turbine_container: Query<Entity, With<TurbineGaugeContainer>>,
    buyback_buttons: Query<Entity, With<BuyBackButton>>,
    asset_server: Res<AssetServer>,
) {
    if !turbine.is_changed() || turbine.is_destroyed {
        return;
    }

    let font = asset_server.load("fonts/LTSuperior-Regular.ttf");

    // Check if buyback button exists
    for button_entity in buyback_buttons.iter() {
        // Despawn the buyback button
        commands.entity(button_entity).despawn();

        // Add the gauge back
        for container_entity in turbine_container.iter() {
            commands.entity(container_entity).with_child((
                Node {
                    width: Val::Px(GAUGE_SIZE),
                    height: Val::Px(GAUGE_SIZE),
                    border: UiRect::all(Val::Px(GAUGE_BORDER)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BorderRadius::MAX,
                BorderColor::all(Color::srgb(0.2, 0.8, 0.2)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                GaugeBorder {
                    gauge_type: GaugeType::TurbineTemp,
                },
                BlinkTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
                children![
                    (
                        Text::new("0°C"),
                        TextFont {
                            font: font.clone(),
                            font_size: GAUGE_VALUE_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TurbineTempIndicator,
                    ),
                    (
                        Text::new("100%"),
                        TextFont {
                            font: font.clone(),
                            font_size: GAUGE_DURABILITY_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        TurbineDurabilityText,
                    ),
                ],
            ));
        }
    }
}
