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

pub const REACTOR_TEMP_LIMIT: f32 = 1200.0; // °C - explosion threshold
pub const REACTOR_PRESSURE_LIMIT: f32 = 160.0; // bar - explosion threshold
pub const TURBINE_TEMP_LIMIT: f32 = 290.0; // °C
const ROOM_TEMPERATURE: f32 = 20.0; // °C

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
    let reactivity = (controls.reactivity_applied / 100.0).clamp(0.0, 1.2);

    // Exponential temperature growth: higher reactivity causes exponential increase
    // Temperature can now go beyond the explosion limit (no artificial cap)
    let exp_factor = (reactivity * 2.5).exp();
    // Scale exponentially without capping
    let temperature_target = ROOM_TEMPERATURE + (exp_factor - 1.0) * 150.0;

    // Water flow through turbine cools the reactor
    let flow_rate = (controls.turbine_applied / 100.0).clamp(0.0, 1.0);
    let cooling_effect = flow_rate * 0.15;

    // When water flows, reactor cools toward a lower equilibrium
    let effective_target =
        temperature_target - (flow_rate * (reactor.temperature - ROOM_TEMPERATURE) * 0.3);

    // Exponential rate increase: temperature changes faster as it gets higher
    let temp_normalized = ((reactor.temperature - ROOM_TEMPERATURE) / 1000.0).max(0.0);
    let base_rate = 0.08;
    // Rate increases exponentially with temperature (positive feedback loop)
    let rate_multiplier = 1.0 + (temp_normalized * 2.0).exp() * 0.5;
    let change_rate = (base_rate + cooling_effect) * rate_multiplier;

    reactor.temperature = smooth_towards(reactor.temperature, effective_target, change_rate, delta);

    // Pressure increases exponentially with reactivity
    // Reduced coupling with temperature so pressure doesn't limit temperature growth
    // No artificial cap - can exceed explosion limit
    let pressure_base = reactivity * 150.0;
    let pressure_exp_boost = ((reactivity * 2.0).exp() - 1.0) * 20.0;
    // Small temperature contribution (reduced from 0.05 to 0.01 to reduce coupling)
    let temp_pressure_contribution = (reactor.temperature - 800.0).max(0.0) * 0.01;
    let pressure_target = pressure_base + pressure_exp_boost + temp_pressure_contribution;

    // Pressure change rate increases with reactivity and current pressure
    let pressure_normalized = (reactor.pressure / 100.0).max(0.0);
    let pressure_rate = 1.2 + reactivity * 1.8 + pressure_normalized * 0.4;
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

    // Generator requires minimum 100°C to produce power
    if turbine.temperature < 100.0 {
        environment.power_generated = 0.0;
    } else {
        environment.power_generated = base_power * flow_multiplier;
    }

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
