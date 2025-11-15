use crate::GameState;
use crate::simulation::{REACTOR_PRESSURE_LIMIT, ReactorState};
use bevy::audio::Volume::Linear;
use bevy::prelude::*;

#[derive(Component)]
struct BackgroundMusic;

#[derive(Component)]
struct HissingSound;

#[derive(Resource)]
pub struct AudioSettings {
    pub volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self { volume: 1.0 }
    }
}

#[derive(Resource)]
struct HissingState {
    pub active: bool,
}

impl Default for HissingState {
    fn default() -> Self {
        Self { active: false }
    }
}

#[derive(Component)]
struct ExplosionSound;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AudioSettings::default())
            .insert_resource(HissingState::default())
            .add_systems(OnEnter(GameState::InGame), setup_audio)
            .add_systems(
                Update,
                (
                    update_audio_volume,
                    create_hissing_system,
                    hissing_activating_system,
                ).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnEnter(GameState::GameOver), explosion_audio_system);
    }
}

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(create_background_music(asset_server));
}

fn create_background_music(asset_server: Res<AssetServer>) -> impl Bundle {
    (
        AudioPlayer::new(asset_server.load("sound/background_music.mp3")),
        PlaybackSettings::LOOP,
        BackgroundMusic,
        DespawnOnExit(GameState::InGame),
    )
}


fn update_audio_volume(
    settings: Res<AudioSettings>,
    mut query: Query<&mut AudioSink>
) {
    for mut sink in query.iter_mut() {
        sink.set_volume(Linear(settings.volume));
    }
}

fn explosion_audio_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>
) {
    commands.spawn(create_explosion_sound(asset_server, settings));
}

fn create_explosion_sound(
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>
) -> impl Bundle {
    (
        DespawnOnExit(GameState::GameOver),
        AudioPlayer::new(asset_server.load("sound/explosion.mp3")),
        PlaybackSettings {
            volume: Linear((settings.volume - 0.5).clamp(0.0, 1.0)),
            ..default()
        },
        ExplosionSound,
    )
}

fn create_hissing_system(
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    hissing: Res<HissingState>,
    mut commands: Commands,
    existing: Query<Entity, With<HissingSound>>,
) {
    // If alarm should play but no alarm entity exists, spawn it
    if hissing.active && existing.is_empty() {
        commands.spawn((
            DespawnOnExit(GameState::InGame),
            AudioPlayer::new(asset_server.load("sound/hissing.mp3")),
            PlaybackSettings {
                volume: Linear((settings.volume - 0.35).clamp(0.0, 1.0)),
                mode: bevy::audio::PlaybackMode::Loop,
                ..default()
            },
            HissingSound,
        ));
        return;
    }

    // If alarm should stop but exists, despawn it
    if !hissing.active {
        for e in existing.iter() {
            commands.entity(e).despawn();
        }
    }
}

fn hissing_activating_system(
    reactor: Res<ReactorState>,
    mut hissing: ResMut<HissingState>,
) {
    let reactor_pct = reactor.pressure / REACTOR_PRESSURE_LIMIT;
    let should_hiss = reactor_pct >= 0.70;

    if should_hiss && !hissing.active {
        hissing.active = true;
        info!("Hissing activated");
    } else if !should_hiss && hissing.active {
        hissing.active = false;
        info!("Hissing deactivated");
    }
}