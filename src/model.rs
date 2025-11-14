use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*
};
use std::f32::consts::*;

use crate::GameState;

pub struct Reactor3dPlugin;

impl Plugin for Reactor3dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DirectionalLightShadowMap { size: 4096 })
            .add_systems(OnEnter(GameState::InGame), setup_3d_scene)
            .add_systems(Update, (animate_light_direction).run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
struct GameScene;

fn setup_3d_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
        GameScene,
    ));

    commands.spawn((
        SceneRoot(asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("models/reactor.glb"),
        )),
        GameScene,
    ));
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_secs() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

