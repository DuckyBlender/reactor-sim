use bevy::{input_focus::tab_navigation::TabGroup, prelude::*};

use crate::{
    FONT_REGULAR, GameState,
    model::RefuelAnimationState,
    simulation::{ControlSettings, EnvironmentState},
};

mod game_over;
pub mod indicators;
mod pause;
pub mod sliders;
pub use pause::PauseState;
pub mod uranek;

#[derive(Component)]
pub struct UpgradeButton;

#[derive(Component)]
struct MoneyText;
pub struct ReactorUiPlugin;

impl Plugin for ReactorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<uranek::UranekState>()
            .add_systems(OnEnter(GameState::InGame), setup_game_ui)
            .add_systems(OnEnter(GameState::Tutorial), setup_game_ui)
            .add_systems(
                Update,
                (
                    sliders::sync_slider_values,
                    sliders::update_slider_visuals.after(sliders::sync_slider_values),
                    sliders::update_slider_value_text,
                    sliders::update_applied_value_text,
                ),
            )
            .add_systems(
                Update,
                (
                    indicators::update_indicators,
                    indicators::update_gauge_colors,
                    indicators::handle_turbine_destroyed,
                    indicators::rebuild_turbine_gauge_from_buyback,
                    update_money_display,
                    update_refuel_button_state,
                    handle_refuel_button,
                )
                    .run_if(in_state(GameState::InGame).or(in_state(GameState::Tutorial))),
            )
            .add_systems(
                Update,
                (
                    uranek::update_uranek_idle_animation,
                    uranek::update_uranek_dialogue,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_plugins(game_over::GameOverPlugin)
            .add_plugins(pause::PausePlugin);
    }
}

fn setup_game_ui(
    mut commands: Commands,
    controls: Res<ControlSettings>,
    asset_server: Res<AssetServer>,
    mut uranek_state: ResMut<uranek::UranekState>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
    state: Res<State<GameState>>,
) {
    let current_state = *state.get();
    *uranek_state = uranek::UranekState::default();
    uranek_state.last_default_text_time = time.elapsed_secs();

    let font = asset_server.load(FONT_REGULAR);

    // Load spritesheet and create atlas layout
    let spritesheet_texture = asset_server.load("sprites/spritesheet.png");

    // Parse sprites.txt to define atlas layout
    // Format: talk,0,0,928,1120; idle,929,0,928,1120; wave,0,1121,928,1120; hot,929,1121,928,1120
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(1857, 2241));

    let talk_idx = layout.add_texture(URect::new(0, 0, 928, 1120));
    let idle_idx = layout.add_texture(URect::new(929, 0, 1857, 1120));
    layout.add_texture(URect::new(0, 1121, 928, 2241)); // wave
    let hot_idx = layout.add_texture(URect::new(929, 1121, 1857, 2241)); // hot

    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Load and store Uranek assets once
    let uranek_assets = uranek::UranekAssets {
        spritesheet: spritesheet_texture.clone(),
        atlas_layout: texture_atlas_layout.clone(),
        idle_idx,
        talk_idx,
        hot_idx,
    };

    // Store sprite info for use before inserting resource
    let uranek_sprite_texture = uranek_assets.spritesheet.clone();
    let uranek_atlas_layout = uranek_assets.atlas_layout.clone();
    let uranek_idle_idx = uranek_assets.idle_idx;

    commands.insert_resource(uranek_assets);

    // Main UI root
    let mut ui_root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(40.0)),
            row_gap: Val::Px(60.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        TabGroup::default(),
        Transform::default(),
        children![
            // Title
            (
                TextFont {
                    font: font.clone(),
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    right: Val::Px(40.0),
                    ..default()
                },
            ),
            // Money display
            (
                Text::new("$0"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MoneyText,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(100.0),
                    right: Val::Px(40.0),
                    ..default()
                },
            ),
            (
                Button,
                Node {
                    width: Val::Px(260.0),
                    height: Val::Px(96.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(200.0),
                    right: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(18.0)),
                BorderColor::all(Color::srgba(0.95, 0.8, 0.3, 0.75)),
                BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
                UpgradeButton,
                children![
                    (
                        Text::new("Refuel Rods"),
                        TextFont {
                            font: font.clone(),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                    ),
                    (
                        Text::new("$250 instant service"),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.8, 0.4)),
                    ),
                ],
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
                Transform::default(),
                children![
                    // Left side - Gauges
                    indicators::gauge_grid(font.clone()),
                    // Right side - Sliders
                    sliders::slider_panel(
                        controls.reactivity_target,
                        controls.turbine_target,
                        font.clone(),
                        &asset_server
                    ),
                ],
            ),
        ],
    ));

    match current_state {
        GameState::InGame => {
            ui_root.insert(DespawnOnExit(GameState::InGame));
        }
        GameState::Tutorial => {
            ui_root.insert(DespawnOnExit(GameState::Tutorial));
        }
        _ => {}
    }

    // Spawn Uranek companion only in InGame state, not during Tutorial
    if current_state == GameState::InGame {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                align_items: AlignItems::End,
                ..default()
            },
            DespawnOnExit(GameState::InGame),
            children![
                // Uranek sprite
                (
                    ImageNode {
                        image: uranek_sprite_texture,
                        texture_atlas: Some(TextureAtlas {
                            layout: uranek_atlas_layout,
                            index: uranek_idle_idx,
                        }),
                        ..default()
                    },
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(220.0),
                        ..default()
                    },
                    uranek::Uranek,
                    uranek::UranekIdle,
                ),
                // Speech bubble (initially visible with default text)
                (
                    Node {
                        width: Val::Px(420.0),
                        min_height: Val::Px(80.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
                    BorderRadius::all(Val::Px(12.0)),
                    BorderColor::all(Color::srgb(0.6, 0.9, 1.0)),
                    uranek::UranekBubble,
                    children![(
                        Text::new("Uranek: Pilnujmy, żeby ten reaktor trzymał się w ryzach."),
                        TextFont {
                            font,
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        uranek::UranekText,
                    ),],
                ),
            ],
        ));
    }
}

fn update_money_display(
    environment: Res<EnvironmentState>,
    mut texts: Query<&mut Text, With<MoneyText>>,
) {
    if !environment.is_changed() {
        return;
    }

    for mut text in texts.iter_mut() {
        **text = format!("${:.0}", environment.money);
    }
}

const REFUEL_COST: f32 = 250.0;

#[allow(clippy::type_complexity)]
fn update_refuel_button_state(
    environment: Res<EnvironmentState>,
    mut button_query: Query<
        (&mut BackgroundColor, &mut BorderColor, &Interaction),
        (With<UpgradeButton>, Changed<Interaction>),
    >,
) {
    const NORMAL_BG: Color = Color::srgba(0.08, 0.08, 0.12, 0.9);
    const HOVER_BG: Color = Color::srgba(0.12, 0.12, 0.16, 0.95);
    const PRESS_BG: Color = Color::srgba(0.06, 0.06, 0.10, 1.0);
    const DISABLED_BG: Color = Color::srgba(0.05, 0.05, 0.08, 0.7);
    
    const NORMAL_BORDER: Color = Color::srgba(0.95, 0.8, 0.3, 0.75);
    const HOVER_BORDER: Color = Color::srgba(1.0, 0.9, 0.4, 0.9);
    const PRESS_BORDER: Color = Color::srgba(0.85, 0.7, 0.25, 1.0);
    const DISABLED_BORDER: Color = Color::srgba(0.3, 0.3, 0.3, 0.4);

    for (mut bg_color, mut border_color, interaction) in button_query.iter_mut() {
        let can_afford = environment.money >= REFUEL_COST;
        
        if !can_afford {
            *bg_color = DISABLED_BG.into();
            *border_color = BorderColor::all(DISABLED_BORDER);
        } else {
            match *interaction {
                Interaction::Pressed => {
                    *bg_color = PRESS_BG.into();
                    *border_color = BorderColor::all(PRESS_BORDER);
                }
                Interaction::Hovered => {
                    *bg_color = HOVER_BG.into();
                    *border_color = BorderColor::all(HOVER_BORDER);
                }
                Interaction::None => {
                    *bg_color = NORMAL_BG.into();
                    *border_color = BorderColor::all(NORMAL_BORDER);
                }
            }
        }
    }
}

fn handle_refuel_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<UpgradeButton>)>,
    mut environment: ResMut<EnvironmentState>,
    mut refuel_animation: ResMut<RefuelAnimationState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed && environment.money >= REFUEL_COST {
            environment.money -= REFUEL_COST;
            environment.fuel_left = 1.0;
            refuel_animation.trigger();
        }
    }
}
