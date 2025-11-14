use bevy::prelude::*;

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
            pressure: 1.0, // Room pressure in bar
        }
    }
}

#[derive(Resource, Debug)]
pub struct TurbineState {
    pub speed: f32,              // RPM * 100
    pub temperature: f32,        // °C
    pub target_temperature: f32, // °C
    pub durability: f32,         // 0-100
    pub is_destroyed: bool,
}

impl Default for TurbineState {
    fn default() -> Self {
        Self {
            speed: 0.0,
            temperature: ROOM_TEMPERATURE,
            target_temperature: ROOM_TEMPERATURE,
            durability: 100.0,
            is_destroyed: false,
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
            money: 0.0,
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

#[derive(Resource, Debug, Clone, Default)]
pub enum GameOverReason {
    #[default]
    None,
    ReactorOverheat,
    TurbineOverheat,
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
    controls.turbine_applied = smooth_towards(
        controls.turbine_applied,
        controls.turbine_target,
        0.15,
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

    // Water flow through turbine cools the reactor
    let flow_rate = (controls.turbine_applied / 100.0).clamp(0.0, 1.0);
    let cooling_effect = flow_rate * 0.15;

    // When water flows, reactor cools toward a lower equilibrium
    // The more flow, the more it pulls heat away
    let effective_target =
        temperature_target - (flow_rate * (reactor.temperature - ROOM_TEMPERATURE) * 0.3);

    reactor.temperature = smooth_towards(
        reactor.temperature,
        effective_target,
        0.08 + cooling_effect,
        delta,
    );

    let pressure_target =
        heat * REACTOR_PRESSURE_MAX + ((reactor.temperature - 600.0).max(0.0) * 0.01);
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

    // If turbine is destroyed, no operation
    if turbine.is_destroyed {
        turbine.speed = smooth_towards(turbine.speed, 0.0, 2.0, delta);
        let power_target = 0.0;
        environment.power_generated =
            smooth_towards(environment.power_generated, power_target, 2.0, delta);
        turbine.temperature = smooth_towards(turbine.temperature, ROOM_TEMPERATURE, 0.048, delta);
        return;
    }

    let flow_rate = (controls.turbine_applied / 100.0).clamp(0.0, 1.0);

    // When flow is active, water carries heat from reactor to turbine
    if flow_rate > 0.01 {
        // Water picks up heat from reactor and carries it to turbine
        // Turbine can't be hotter than reactor
        turbine.target_temperature = reactor.temperature.min(reactor.temperature);

        // Map 0-100% slider to 0.003-0.048 transfer rate
        let transfer_speed = 0.003 + flow_rate * 0.045;

        // Smoothly move turbine temperature toward reactor temperature
        turbine.temperature = smooth_towards(
            turbine.temperature,
            turbine.target_temperature,
            transfer_speed,
            delta,
        );
    } else {
        // No water flow - turbine cools naturally
        turbine.temperature = smooth_towards(turbine.temperature, ROOM_TEMPERATURE, 0.048, delta);
    }

    // Calculate turbine speed based on flow rate
    // Speed represents mechanical rotation from water flow
    let torque = flow_rate * 2000.0;
    turbine.speed = smooth_towards(turbine.speed, torque, 2.0, delta);

    // Power generation is based on turbine TEMPERATURE, not just speed
    // Sweet spot is in yellow-red zone (60-95% of TURBINE_TEMP_LIMIT = 174-275.5°C)
    let temp_percentage = (turbine.temperature / TURBINE_TEMP_LIMIT).clamp(0.0, 1.0);

    // Power efficiency curve: peaks around 70-85% temp (yellow-red zone)
    let power_efficiency = if temp_percentage < 0.60 {
        // Below 60%: low efficiency, ramping up
        temp_percentage / 0.60 * 0.7
    } else if temp_percentage < 0.85 {
        // Sweet spot (60-85%): high efficiency
        0.7 + (temp_percentage - 0.60) / 0.25 * 0.3
    } else {
        // Above 85%: efficiency drops due to stress and damage
        1.0 - (temp_percentage - 0.85) / 0.15 * 0.3
    };

    // Power generation based on temperature efficiency
    // Turbine strength reduced to 10% of original
    let base_power = power_efficiency * 100.0;
    let flow_multiplier = if flow_rate > 0.01 { 1.0 } else { 0.5 }; // Reduced multiplier when no flow
    environment.power_generated = base_power * flow_multiplier;

    // Money generation: power = money directly
    environment.money += environment.power_generated * delta * 0.1;

    // Durability system: damage when in red zone (80-95% of limit)
    let temp_percentage = turbine.temperature / TURBINE_TEMP_LIMIT;
    if temp_percentage >= 0.80 {
        // Damage increases the deeper into red zone
        // At 80% = 0 damage, at 95% = 5 damage/sec
        let damage_factor = ((temp_percentage - 0.80) / 0.15).clamp(0.0, 1.0);
        let damage = damage_factor * 5.0 * delta;
        turbine.durability = (turbine.durability - damage).max(0.0);

        if turbine.durability <= 0.0 {
            turbine.is_destroyed = true;
        }
    }
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
    turbine: Res<TurbineState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_over_reason: ResMut<GameOverReason>,
) {
    if reactor.temperature >= REACTOR_TEMP_LIMIT {
        info!(
            "Game Over: Reactor temperature exceeded limit ({}°C >= {}°C)",
            reactor.temperature, REACTOR_TEMP_LIMIT
        );
        *game_over_reason = GameOverReason::ReactorOverheat;
        next_state.set(GameState::GameOver);
    } else if turbine.temperature >= TURBINE_TEMP_LIMIT {
        info!(
            "Game Over: Turbine temperature exceeded limit ({}°C >= {}°C)",
            turbine.temperature, TURBINE_TEMP_LIMIT
        );
        *game_over_reason = GameOverReason::TurbineOverheat;
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
