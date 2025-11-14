use bevy::prelude::*;

use crate::simulation::{EnvironmentState, ReactorState, TurbineState};

#[derive(Component)]
pub struct ReactorTempIndicator;

#[derive(Component)]
pub struct ReactorPressureIndicator;

#[derive(Component)]
pub struct TurbineTempIndicator;

#[derive(Component)]
pub struct RadiationIndicator;

pub fn gauge_grid(font: Handle<Font>) -> impl Bundle {
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

#[allow(clippy::too_many_arguments)]
pub fn update_indicators(
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

