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

pub const REACTOR_TEMP_LIMIT: f32 = 1200.0; // °C
pub const TURBINE_TEMP_LIMIT: f32 = 290.0; // °C
const ROOM_TEMPERATURE: f32 = 20.0; // °C
const REACTOR_TEMP_MAX: f32 = 1200.0;
const REACTOR_PRESSURE_MAX: f32 = 160.0;

#[derive(Resource, Debug)]
pub struct ReactorState {
    pub temperature: f32,
    pub pressure: f32,
}

impl Default for ReactorState {
    fn default() -> Self {
        Self {
            temperature: ROOM_TEMPERATURE,
            pressure: 0.0,
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
            speed: 0.0,
            temperature: ROOM_TEMPERATURE,
        }
    }
}

#[derive(Resource, Debug)]
pub struct EnvironmentState {
    pub money: f32,
    pub power_generated: f32, // MW
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            money: 1_000_000.0,
            power_generated: 0.0,
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
    controls: Res<ControlSettings>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let heat = (controls.reactivity_applied / 100.0).clamp(0.0, 1.2);
    let temperature_target = ROOM_TEMPERATURE + heat * (REACTOR_TEMP_MAX - ROOM_TEMPERATURE);
    reactor.temperature = smooth_towards(reactor.temperature, temperature_target, 0.8, delta);

    let pressure_target = heat * REACTOR_PRESSURE_MAX
        + ((reactor.temperature - 600.0).max(0.0) * 0.01);
    reactor.pressure = smooth_towards(reactor.pressure, pressure_target, 1.5, delta);
}

fn simulate_turbine(
    reactor: Res<ReactorState>,
    mut turbine: ResMut<TurbineState>,
    mut environment: ResMut<EnvironmentState>,
    controls: Res<ControlSettings>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let steam_supply = ((reactor.temperature - ROOM_TEMPERATURE)
        / (REACTOR_TEMP_MAX - ROOM_TEMPERATURE))
        .clamp(0.0, 1.2);
    let demand = (controls.turbine_applied / 100.0).clamp(0.0, 1.2);
    let mismatch = demand - steam_supply;
    let torque = (steam_supply * 2000.0) - mismatch * 300.0;
    let base_temperature = ROOM_TEMPERATURE + steam_supply * 320.0 + mismatch.abs() * 280.0;
    let temperature_target = base_temperature.min(900.0);

    turbine.speed = smooth_towards(turbine.speed, torque, 2.0, delta);
    turbine.temperature = smooth_towards(turbine.temperature, temperature_target, 1.1, delta);

    // Calculate power generation (MW) based on turbine speed
    // Assuming optimal speed is around 1800-2000 RPM * 100
    let power_target = (turbine.speed / 2000.0).max(0.0) * 1000.0; // Max 1000 MW at full speed
    environment.power_generated = smooth_towards(environment.power_generated, power_target, 2.0, delta);
}

fn evaluate_loss(
    reactor: Res<ReactorState>,
    turbine: Res<TurbineState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if reactor.temperature >= REACTOR_TEMP_LIMIT {
        info!("Game Over: Reactor temperature exceeded limit ({}°C >= {}°C)", reactor.temperature, REACTOR_TEMP_LIMIT);
        next_state.set(GameState::GameOver);
    } else if turbine.temperature >= TURBINE_TEMP_LIMIT {
        info!("Game Over: Turbine temperature exceeded limit ({}°C >= {}°C)", turbine.temperature, TURBINE_TEMP_LIMIT);
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
