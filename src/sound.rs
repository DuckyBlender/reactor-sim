use crate::GameState;
use bevy::audio::Volume::Linear;
use bevy::prelude::*;

#[derive(Component)]
struct BackgroundMusic;

#[derive(Resource)]
pub struct AudioSettings {
    pub volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self { volume: 1.0 }
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioSettings::default())
            .add_systems(OnEnter(GameState::InGame), setup_audio)
            .add_systems(OnExit(GameState::InGame), cleanup_audio)
            .add_systems(
                Update,
                update_audio_volume.run_if(in_state(GameState::InGame)),
            );
    }
}

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(create_background_music(asset_server));
}

fn create_background_music(asset_server: Res<AssetServer>) -> impl Bundle {
    (
        AudioPlayer::new(asset_server.load("sound/backgroundmusic.mp3")),
        PlaybackSettings::LOOP,
        BackgroundMusic,
    )
}

fn cleanup_audio(mut commands: Commands, music_query: Query<Entity, With<BackgroundMusic>>) {
    for entity in music_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_audio_volume(settings: Res<AudioSettings>, mut query: Query<&mut AudioSink>) {
    for mut sink in query.iter_mut() {
        sink.set_volume(Linear(settings.volume));
    }
}
