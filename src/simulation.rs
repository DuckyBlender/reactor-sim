use bevy::prelude::*;
use std::f32::consts::LN_2;

use crate::GameState;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ReactorState::default())
            .insert_resource(TurbineState::default())
            .insert_resource(EnvironmentState::default())
            .insert_resource(ControlSettings::default())
            .insert_resource(GameOverReason::default())
            .add_systems(
                OnEnter(GameState::InGame),
                (reset_game_over_reason, reset_game_state),
            )
            .add_systems(
                Update,
                (
                    interpolate_controls,
                    simulate_reactor.after(interpolate_controls),
                    simulate_turbine.after(simulate_reactor),
                    decay_fuel.after(simulate_turbine),
                    evaluate_loss.after(simulate_turbine),
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

pub const REACTOR_TEMP_LIMIT: f32 = 1200.0; // °C - explosion threshold
pub const REACTOR_PRESSURE_LIMIT: f32 = 160.0; // bar - explosion threshold
pub const TURBINE_TEMP_LIMIT: f32 = 290.0; // °C
const ROOM_TEMPERATURE: f32 = 20.0; // °C
const FUEL_HALF_LIFE_SECONDS: f32 = 180.0;

#[derive(Resource, Debug)]
pub struct ReactorState {
    pub temperature: f32,
    pub pressure: f32,
}

impl Default for ReactorState {
    fn default() -> Self {
        Self {
            temperature: ROOM_TEMPERATURE,
            pressure: 1.0, // Room pressure in bar
        }
    }
}

#[derive(Resource, Debug)]
pub struct TurbineState {
    pub speed: f32,        // RPM * 100
    pub temperature: f32,  // °C
    pub durability: f32,   // 0-100
    pub is_destroyed: bool,
}

impl Default for TurbineState {
    fn default() -> Self {
        Self {
            speed: 0.0,
            temperature: ROOM_TEMPERATURE,
            durability: 100.0,
            is_destroyed: false,
        }
    }
}

#[derive(Resource, Debug)]
pub struct EnvironmentState {
    pub money: f32,
    pub power_generated: f32, // MW
    pub fuel_left: f32,       // 0-1
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            money: 1000.0,
            power_generated: 0.0,
            fuel_left: 1.0,
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

#[derive(Resource, Debug, Clone, Default)]
pub enum GameOverReason {
    #[default]
    None,
    ReactorExplosion, // Pressure exceeded limit
    ReactorMeltdown,  // Temperature exceeded limit
}

impl Default for ControlSettings {
    fn default() -> Self {
        let initial_reactivity = 0.0;
        let initial_turbine = 0.0;
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
        0.15,
        delta,
    );
    controls.turbine_applied = controls.turbine_target;
}

fn simulate_reactor(
    mut reactor: ResMut<ReactorState>,
    controls: Res<ControlSettings>,
    environment: Res<EnvironmentState>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let reactivity = (controls.reactivity_applied / 100.0).clamp(0.0, 1.2);
    let fuel_energy = fuel_energy_factor(environment.fuel_left);
    let fuel_rate = fuel_rate_factor(environment.fuel_left);

    // Nuclear reaction generates heat based on reactivity
    // Exponential growth: higher reactivity causes exponential temperature increase
    let exp_factor = (reactivity * 2.5).exp();
    let heat_generation_target = ROOM_TEMPERATURE + (exp_factor - 1.0) * 150.0 * fuel_energy;

    // Water flow removes heat from reactor
    let flow_rate = (controls.turbine_applied / 100.0).clamp(0.0, 1.0);
    let heat_removed_by_water = flow_rate * (reactor.temperature - ROOM_TEMPERATURE) * 0.4;
    
    // Final temperature target balances heat generation and water cooling
    let temperature_target = heat_generation_target - heat_removed_by_water;

    // Temperature change rate increases with temperature (positive feedback)
    let temp_normalized = ((reactor.temperature - ROOM_TEMPERATURE) / 1000.0).max(0.0);
    let rate_multiplier = (1.0 + (temp_normalized * 2.0).exp() * 0.5) * fuel_rate;
    let change_rate = 0.016 * rate_multiplier; // 5x slower natural cooldown

    reactor.temperature = smooth_towards(reactor.temperature, temperature_target, change_rate, delta);

    // Pressure increases with both reactivity and temperature
    let pressure_from_reactivity = reactivity * 15.0 * fuel_energy;
    let pressure_exponential_boost = ((reactivity * 1.8).exp() - 1.0) * 2.5 * fuel_energy;
    let pressure_from_temperature = (reactor.temperature - 800.0).max(0.0) * 0.015 * fuel_energy;
    let pressure_target = pressure_from_reactivity + pressure_exponential_boost + pressure_from_temperature;

    // Pressure change rate increases with reactivity and current pressure
    let pressure_normalized = (reactor.pressure / 100.0).max(0.0);
    let pressure_rate = (0.04 + reactivity * 0.08 + pressure_normalized * 0.01) * fuel_rate; // Much slower pressure buildup
    reactor.pressure = smooth_towards(reactor.pressure, pressure_target, pressure_rate, delta);
}

fn simulate_turbine(
    reactor: Res<ReactorState>,
    mut turbine: ResMut<TurbineState>,
    mut environment: ResMut<EnvironmentState>,
    controls: Res<ControlSettings>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    // If turbine is destroyed, no operation
    if turbine.is_destroyed {
        turbine.speed = smooth_towards(turbine.speed, 0.0, 2.0, delta);
        environment.power_generated = smooth_towards(environment.power_generated, 0.0, 2.0, delta);
        turbine.temperature = smooth_towards(turbine.temperature, ROOM_TEMPERATURE, 0.075, delta);
        return;
    }

    let flow_rate = (controls.turbine_applied / 100.0).clamp(0.0, 1.0);

    // Water flow carries heat from reactor to turbine
    // Heat transfer rate depends on: flow rate, temperature difference, and transfer efficiency
    let heat_transfer_rate = flow_rate * 0.36; // How quickly water transfers heat (80% faster)
    let heat_carried_by_water = reactor.temperature * flow_rate * 0.95; // Heat capacity of water flow
    
    // Turbine naturally cools toward room temperature (always active)
    const NATURAL_COOLING_RATE: f32 = 0.075; // 2x slower natural cooldown
    
    // Calculate equilibrium temperature:
    // Heat input from water flow vs natural cooling
    // When these balance, turbine reaches steady state
    let heat_input_target = ROOM_TEMPERATURE + heat_carried_by_water;
    
    // Apply both heating (from water) and natural cooling
    // First apply water heating
    turbine.temperature = smooth_towards(
        turbine.temperature,
        heat_input_target,
        heat_transfer_rate,
        delta,
    );
    
    // Then apply natural cooling (always active)
    turbine.temperature = smooth_towards(
        turbine.temperature,
        ROOM_TEMPERATURE,
        NATURAL_COOLING_RATE,
        delta,
    );

    // Calculate turbine speed based on flow rate
    let target_speed = flow_rate * 2000.0;
    turbine.speed = smooth_towards(turbine.speed, target_speed, 2.0, delta);

    // Power generation based on turbine temperature
    // Sweet spot: 60-85% of TURBINE_TEMP_LIMIT (174-246°C)
    let temp_percentage = (turbine.temperature / TURBINE_TEMP_LIMIT).clamp(0.0, 1.0);

    let power_efficiency = if temp_percentage < 0.60 {
        // Below 60%: low efficiency
        temp_percentage / 0.60 * 0.7
    } else if temp_percentage < 0.85 {
        // Sweet spot (60-85%): high efficiency
        0.7 + (temp_percentage - 0.60) / 0.25 * 0.3
    } else {
        // Above 85%: efficiency drops due to stress
        1.0 - (temp_percentage - 0.85) / 0.15 * 0.3
    };

    // Power generation requires minimum temperature and flow
    if turbine.temperature < 100.0 || flow_rate < 0.01 {
        environment.power_generated = 0.0;
    } else {
        let base_power = power_efficiency * 100.0;
        environment.power_generated = base_power;
    }

    // Money generation from power
    environment.money += environment.power_generated * delta * 0.1;

    // Durability damage in red zone (80%+ of limit)
    if temp_percentage >= 0.80 {
        let damage_factor = ((temp_percentage - 0.80) / 0.15).clamp(0.0, 1.0);
        let damage = damage_factor * 5.0 * delta;
        turbine.durability = (turbine.durability - damage).max(0.0);

        if turbine.durability <= 0.0 {
            turbine.is_destroyed = true;
        }
    }
}

fn decay_fuel(mut environment: ResMut<EnvironmentState>, time: Res<Time>) {
    if environment.fuel_left <= 0.0 {
        environment.fuel_left = 0.0;
        return;
    }

    let delta = time.delta_secs();
    let decay_constant = LN_2 / FUEL_HALF_LIFE_SECONDS;
    let multiplier = (-decay_constant * delta).exp();
    environment.fuel_left = (environment.fuel_left * multiplier).clamp(0.0, 1.0);
}

fn reset_game_over_reason(mut game_over_reason: ResMut<GameOverReason>) {
    *game_over_reason = GameOverReason::None;
}

fn reset_game_state(
    mut reactor: ResMut<ReactorState>,
    mut turbine: ResMut<TurbineState>,
    mut environment: ResMut<EnvironmentState>,
    mut controls: ResMut<ControlSettings>,
) {
    *reactor = ReactorState::default();
    *turbine = TurbineState::default();
    *environment = EnvironmentState::default();
    *controls = ControlSettings::default();
}

fn evaluate_loss(
    reactor: Res<ReactorState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_over_reason: ResMut<GameOverReason>,
) {
    // Check temperature first (meltdown)
    if reactor.temperature >= REACTOR_TEMP_LIMIT {
        info!(
            "REACTOR MELTDOWN! Temperature: {:.1}°C exceeded limit of {}°C",
            reactor.temperature, REACTOR_TEMP_LIMIT
        );
        *game_over_reason = GameOverReason::ReactorMeltdown;
        next_state.set(GameState::GameOver);
    }
    // Check pressure (explosion)
    else if reactor.pressure >= REACTOR_PRESSURE_LIMIT {
        info!(
            "REACTOR EXPLOSION! Pressure: {:.1} bar exceeded limit of {} bar",
            reactor.pressure, REACTOR_PRESSURE_LIMIT
        );
        *game_over_reason = GameOverReason::ReactorExplosion;
        next_state.set(GameState::GameOver);
    }
    // Turbine overheating no longer causes game over - it just breaks (handled in simulate_turbine)
}

fn smooth_towards(current: f32, target: f32, rate: f32, delta: f32) -> f32 {
    if (target - current).abs() < f32::EPSILON {
        return target;
    }
    let t = 1.0 - (-rate * delta).exp();
    current + (target - current) * t
}

fn fuel_energy_factor(fuel_left: f32) -> f32 {
    let clamped = fuel_left.clamp(0.0, 1.0);
    0.35 + clamped * 0.65
}

fn fuel_rate_factor(fuel_left: f32) -> f32 {
    let clamped = fuel_left.clamp(0.0, 1.0);
    0.5 + clamped * 0.5
}
