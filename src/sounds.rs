use bevy::{math::ops, prelude::*};

pub struct SoundPlugin;


#[derive(Component)]
pub struct GamesAssetsAudio {
    audio: AudioPlayer,
    sound: Sounds,
}

fn spawn_audio(mut commands: Commands, asset_server: Res<AssetServer>, audio: Sounds) {
    match audio {
        Sounds::Explosion1Sound => {
            commands.spawn((
            AudioPlayer::new(asset_server.load("sound/explosion1.mp3")),
            Sounds::Explosion1Sound,
            )); 
        }

        Sounds::Explosion2Sound => {
            commands.spawn(GamesAssetsAudio {
            audio: AudioPlayer::new(asset_server.load("sound/explosion2.mp3")),
            sound: Sounds::Explosion2Sound,
        });
        }

        Sounds::Explosion3Sound => {
            commands.spawn(GamesAssetsAudio {
            audio: AudioPlayer::new(asset_server.load("sound/explosion3.mp3")),
            sound: Sounds::Explosion3Sound,
        });
        }

        Sounds::HissSound => {
            commands.spawn(GamesAssetsAudio {
            audio: AudioPlayer::new(asset_server.load("sound/hiss.mp3")),
            sound: Sounds::HissSound,
        });
        }

        Sounds::UpgradeSound => {
            commands.spawn(GamesAssetsAudio {
            audio: AudioPlayer::new(asset_server.load("sound/upgrade.mp3")),
            sound: Sounds::UpgradeSound,
        });
        }

        Sounds::UranekTalkingSound => {
            commands.spawn(GamesAssetsAudio {
            audio: AudioPlayer::new(asset_server.load("sound/uranek_talking.mp3")),
            sound: Sounds::UranekTalkingSound,
        });
        }
    }

}

#[derive(Component)]
enum Sounds {
    Explosion1Sound,
    Explosion2Sound,
    Explosion3Sound,
    HissSound,
    UpgradeSound,
    UranekTalkingSound,
}

// fn pause(
//     music_controller: Query<&AudioSink>,
// ) {
//     let Ok(sink) = music_controller.single() else {
//         return;
//     };
//     sink.toggle_playback();
// }

fn mute_explosions(
    mut music_controller: Query<&AudioSink, Or(With<Sounds::Explosion1Sound, With<Sounds::Explosion2Souns>)>>,
) {
    let Ok(mut sink) = music_controller.audio.single_mut() else {
        return;
    };

    sink.toggle_mute();
    }

fn  change_volume(
    mut music_controller: Query<&mut GamesAssetsAudio>,
    mut chosen_audio: Sounds,
    value: f32,
) {
    let Ok(mut sink) = music_controller.audio.single_mut() else {
        return;
    };

    match music_controller.sound {
        chosen_audio => {
            let current_volume = sink.volume();
            sink.set_volume(current_volume.increase_by_percentage(value));
        }
        _ => {}
    }
    }