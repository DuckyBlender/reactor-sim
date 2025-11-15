use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    window::PrimaryWindow,
};
use std::f32::consts::*;

use crate::{
    simulation::{ControlSettings, TurbineState},
    GameState,
};

const PARALLAX_FACTOR: f32 = 0.03;
const CAMERA_BASE_POS: Vec3 = Vec3::new(0.0, 5.0, 12.0);
const CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, 4.0, 0.0);
const REFUEL_EXTRA_LIFT: f32 = 10.0;
const REFUEL_RAISE_DURATION: f32 = 0.8;
const REFUEL_HOLD_DURATION: f32 = 1.0;
const REFUEL_LOWER_DURATION: f32 = 0.8;

pub struct Reactor3dPlugin;

impl Plugin for Reactor3dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DirectionalLightShadowMap { size: 4096 })
            .init_resource::<MousePosition>()
            .init_resource::<RefuelAnimationState>()
            .add_systems(OnEnter(GameState::InGame), setup_3d_scene)
            .add_systems(OnEnter(GameState::Tutorial), setup_3d_scene)
            .add_systems(
                Update,
                (
                    camera_parallax,
                    tag_scene_objects,
                    update_refuel_animation,
                    animate_control_rods.after(update_refuel_animation),
                    animate_turbine,
                )
                    .run_if(in_state(GameState::InGame).or(in_state(GameState::Tutorial))),
            );
    }
}

#[derive(Component)]
struct ParallaxCamera;

#[derive(Component)]
struct ControlRod {
    base_y: f32,
}

#[derive(Component)]
struct TurbineObject;

#[derive(Resource)]
pub struct RefuelAnimationState {
    phase: RefuelPhase,
    timer: Timer,
    pub extra_offset: f32,
}

impl Default for RefuelAnimationState {
    fn default() -> Self {
        Self {
            phase: RefuelPhase::Idle,
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            extra_offset: 0.0,
        }
    }
}

impl RefuelAnimationState {
    pub fn trigger(&mut self) {
        self.phase = RefuelPhase::Raising;
        self.timer = Timer::from_seconds(REFUEL_RAISE_DURATION, TimerMode::Once);
        self.extra_offset = 0.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefuelPhase {
    Idle,
    Raising,
    Hold,
    Lowering,
}

#[derive(Resource)]
struct MousePosition {
    normalized: Vec2,
}

impl Default for MousePosition {
    fn default() -> Self {
        Self {
            normalized: Vec2::ZERO,
        }
    }
}

fn setup_3d_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Res<State<GameState>>,
) {
    let current_state = *state.get();

    // Camera with parallax effect
    let mut camera_entity = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(CAMERA_BASE_POS.x, CAMERA_BASE_POS.y, CAMERA_BASE_POS.z)
            .looking_at(CAMERA_LOOK_AT, Vec3::Y),
        ParallaxCamera,
    ));

    match current_state {
        GameState::InGame => {
            camera_entity.insert(DespawnOnExit(GameState::InGame));
        }
        GameState::Tutorial => {
            camera_entity.insert(DespawnOnExit(GameState::Tutorial));
        }
        _ => {}
    }

    // Sunlight from +X direction, angled downward to shine on -X Y surface
    let mut light_entity = commands.spawn((
        DirectionalLight {
            illuminance: 10000.0, // Bright sunlight
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.98, 0.95), // Warm sunlight color
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            -FRAC_PI_2, // Rotate -90° around Y (from +X direction)
            -FRAC_PI_3, // Rotate -60° around X (angled downward)
            0.0,
        )),
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));

    match current_state {
        GameState::InGame => {
            light_entity.insert(DespawnOnExit(GameState::InGame));
        }
        GameState::Tutorial => {
            light_entity.insert(DespawnOnExit(GameState::Tutorial));
        }
        _ => {}
    }

    // Ambient light for overall scene illumination
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.8, 0.85, 0.9), // Cool ambient light
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    let mut scene_entity = commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/reactor.glb")),
    ));

    match current_state {
        GameState::InGame => {
            scene_entity.insert(DespawnOnExit(GameState::InGame));
        }
        GameState::Tutorial => {
            scene_entity.insert(DespawnOnExit(GameState::Tutorial));
        }
        _ => {}
    }
}

fn camera_parallax(
    mut mouse_position: ResMut<MousePosition>,
    mut camera_query: Query<&mut Transform, With<ParallaxCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Normalize cursor position to range [-1, 1]
    let normalized_x = (cursor_position.x / window.width()) * 2.0 - 1.0;
    let normalized_y = -((cursor_position.y / window.height()) * 2.0 - 1.0);

    mouse_position.normalized = Vec2::new(normalized_x, normalized_y);

    // Update camera position with parallax effect
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let offset_x = normalized_x * PARALLAX_FACTOR * CAMERA_BASE_POS.z;
        let offset_y = normalized_y * PARALLAX_FACTOR * CAMERA_BASE_POS.z;

        camera_transform.translation = Vec3::new(
            CAMERA_BASE_POS.x + offset_x,
            CAMERA_BASE_POS.y + offset_y,
            CAMERA_BASE_POS.z,
        );
        camera_transform.look_at(CAMERA_LOOK_AT, Vec3::Y);
    }
}

#[allow(clippy::type_complexity)]
fn tag_scene_objects(
    mut commands: Commands,
    untagged_objects: Query<
        (Entity, &Name, &Transform),
        (Without<ControlRod>, Without<TurbineObject>),
    >,
) {
    for (entity, name, transform) in untagged_objects.iter() {
        let name_str = name.as_str();

        // Tag control rods
        if name_str == "Control 1" || name_str == "Control 2" || name_str == "Control 3" {
            commands.entity(entity).insert(ControlRod {
                base_y: transform.translation.y,
            });
        }

        // Tag turbine
        if name_str == "Turbine" {
            commands.entity(entity).insert(TurbineObject);
        }
    }
}

fn animate_control_rods(
    controls: Res<ControlSettings>,
    refuel_animation: Res<RefuelAnimationState>,
    mut control_rods: Query<(&mut Transform, &ControlRod)>,
) {
    let reactivity_percent = controls.reactivity_applied / 100.0;
    let lift_distance = 3.0; // Maximum lift distance in Blender units
    let extra_lift = refuel_animation.extra_offset;

    for (mut transform, control_rod) in control_rods.iter_mut() {
        // Lift control rods up as reactivity increases
        transform.translation.y =
            control_rod.base_y + (reactivity_percent * lift_distance) + extra_lift;
    }
}

fn animate_turbine(
    turbine_state: Res<TurbineState>,
    mut turbines: Query<&mut Transform, With<TurbineObject>>,
    time: Res<Time>,
) {
    let rotation_speed = (turbine_state.speed / 2000.0) * 2.0 * PI; // Convert speed to radians/sec
    let delta = time.delta_secs();

    for mut transform in turbines.iter_mut() {
        transform.rotate_z(rotation_speed * delta);
    }
}

fn update_refuel_animation(mut state: ResMut<RefuelAnimationState>, time: Res<Time>) {
    match state.phase {
        RefuelPhase::Idle => {
            state.extra_offset = 0.0;
        }
        RefuelPhase::Raising => {
            state.timer.tick(time.delta());
            let progress =
                (state.timer.elapsed_secs() / state.timer.duration().as_secs_f32()).clamp(0.0, 1.0);
            state.extra_offset = progress * REFUEL_EXTRA_LIFT;
            if state.timer.is_finished() {
                state.phase = RefuelPhase::Hold;
                state.timer = Timer::from_seconds(REFUEL_HOLD_DURATION, TimerMode::Once);
            }
        }
        RefuelPhase::Hold => {
            state.timer.tick(time.delta());
            state.extra_offset = REFUEL_EXTRA_LIFT;
            if state.timer.is_finished() {
                state.phase = RefuelPhase::Lowering;
                state.timer = Timer::from_seconds(REFUEL_LOWER_DURATION, TimerMode::Once);
            }
        }
        RefuelPhase::Lowering => {
            state.timer.tick(time.delta());
            let progress =
                (state.timer.elapsed_secs() / state.timer.duration().as_secs_f32()).clamp(0.0, 1.0);
            state.extra_offset = (1.0 - progress) * REFUEL_EXTRA_LIFT;
            if state.timer.is_finished() {
                state.phase = RefuelPhase::Idle;
                state.extra_offset = 0.0;
            }
        }
    }
}
