use crate::GameState;
use bevy::prelude::*;

#[derive(Component)]
struct BackgroundMusic;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_audio)
            .add_systems(OnExit(GameState::InGame), cleanup_music);
    }
}

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(music(asset_server));
}

fn music(asset_server: Res<AssetServer>) -> impl Bundle {
    (
        AudioPlayer::new(asset_server.load("sound/backgroundmusic.mp3")),
        BackgroundMusic,
    )
}

fn cleanup_music(mut commands: Commands, music_query: Query<Entity, With<BackgroundMusic>>) {
    for entity in music_query.iter() {
        commands.entity(entity).despawn();
    }
}
