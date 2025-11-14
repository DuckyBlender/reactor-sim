use bevy::prelude::*;

use crate::GameState;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ReactorState::default())
            .insert_resource(TurbineState::default())
            .insert_resource(EnvironmentState::default())
            .insert_resource(ControlSettings::default())
            .add_systems(
                Update,
                (
                    interpolate_controls,
                    simulate_reactor.after(interpolate_controls),
                    simulate_turbine.after(simulate_reactor),
                    evaluate_loss.after(simulate_turbine),
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

pub const REACTOR_TEMP_LIMIT: f32 = 1100.0; // °C
pub const TURBINE_TEMP_LIMIT: f32 = 700.0; // °C

const REACTOR_TEMP_IDLE: f32 = 320.0;
const REACTOR_TEMP_MAX: f32 = 1200.0;
const REACTOR_PRESSURE_IDLE: f32 = 12.0; // MPa
const REACTOR_PRESSURE_MAX: f32 = 21.0; // MPa
const TURBINE_TEMP_IDLE: f32 = 230.0;
const RADIATION_BACKGROUND: f32 = 0.12; // mSv/h

#[derive(Resource, Debug)]
pub struct ReactorState {
    pub temperature: f32,
    pub pressure: f32,
}

impl Default for ReactorState {
    fn default() -> Self {
        Self {
            temperature: REACTOR_TEMP_IDLE,
            pressure: REACTOR_PRESSURE_IDLE,
        }
    }
}

#[derive(Resource, Debug)]
pub struct TurbineState {
    pub speed: f32,       // RPM * 100
    pub temperature: f32, // °C
}

impl Default for TurbineState {
    fn default() -> Self {
        Self {
            speed: 1500.0,
            temperature: TURBINE_TEMP_IDLE,
        }
    }
}

#[derive(Resource, Debug)]
pub struct EnvironmentState {
    pub money: f32,
    pub radiation: f32, // mSv/h
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            money: 1_000_000.0,
            radiation: RADIATION_BACKGROUND,
        }
    }
}

#[derive(Resource, Debug)]
pub struct ControlSettings {
    pub reactivity_target: f32,
    pub reactivity_applied: f32,
    pub turbine_target: f32,
    pub turbine_applied: f32,
}

impl Default for ControlSettings {
    fn default() -> Self {
        let initial_reactivity = 40.0;
        let initial_turbine = 40.0;
        Self {
            reactivity_target: initial_reactivity,
            reactivity_applied: initial_reactivity,
            turbine_target: initial_turbine,
            turbine_applied: initial_turbine,
        }
    }
}

fn interpolate_controls(mut controls: ResMut<ControlSettings>, time: Res<Time>) {
    let delta = time.delta_secs();
    controls.reactivity_applied = smooth_towards(
        controls.reactivity_applied,
        controls.reactivity_target,
        1.5,
        delta,
    );
    controls.turbine_applied = smooth_towards(
        controls.turbine_applied,
        controls.turbine_target,
        1.1,
        delta,
    );
}

fn simulate_reactor(
    mut reactor: ResMut<ReactorState>,
    mut environment: ResMut<EnvironmentState>,
    controls: Res<ControlSettings>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let heat = (controls.reactivity_applied / 100.0).clamp(0.0, 1.2);
    let temperature_target = REACTOR_TEMP_IDLE + heat * (REACTOR_TEMP_MAX - REACTOR_TEMP_IDLE);
    reactor.temperature = smooth_towards(reactor.temperature, temperature_target, 0.8, delta);

    let pressure_target = REACTOR_PRESSURE_IDLE
        + heat * (REACTOR_PRESSURE_MAX - REACTOR_PRESSURE_IDLE)
        + ((reactor.temperature - 600.0).max(0.0) * 0.01);
    reactor.pressure = smooth_towards(reactor.pressure, pressure_target, 1.5, delta);

    let radiation_target = RADIATION_BACKGROUND + ((reactor.temperature - 650.0).max(0.0) * 0.02);
    environment.radiation = smooth_towards(environment.radiation, radiation_target, 0.5, delta);
}

fn simulate_turbine(
    reactor: Res<ReactorState>,
    mut turbine: ResMut<TurbineState>,
    controls: Res<ControlSettings>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let steam_supply = ((reactor.temperature - REACTOR_TEMP_IDLE)
        / (REACTOR_TEMP_MAX - REACTOR_TEMP_IDLE))
        .clamp(0.0, 1.2);
    let demand = (controls.turbine_applied / 100.0).clamp(0.0, 1.2);
    let mismatch = demand - steam_supply;
    let torque = (steam_supply * 2000.0) - mismatch * 300.0;
    let base_temperature = TURBINE_TEMP_IDLE + steam_supply * 320.0 + mismatch.abs() * 280.0;
    let temperature_target = base_temperature.min(900.0);

    turbine.speed = smooth_towards(turbine.speed, torque, 2.0, delta);
    turbine.temperature = smooth_towards(turbine.temperature, temperature_target, 1.1, delta);
}

fn evaluate_loss(
    reactor: Res<ReactorState>,
    turbine: Res<TurbineState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if reactor.temperature >= REACTOR_TEMP_LIMIT || turbine.temperature >= TURBINE_TEMP_LIMIT {
        next_state.set(GameState::GameOver);
    }
}

fn smooth_towards(current: f32, target: f32, rate: f32, delta: f32) -> f32 {
    if (target - current).abs() < f32::EPSILON {
        return target;
    }
    let t = 1.0 - (-rate * delta).exp();
    current + (target - current) * t
}
