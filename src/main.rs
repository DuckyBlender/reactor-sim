use bevy::prelude::*;

mod main_menu;
use main_menu::{main_menu_plugin, GameState};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(main_menu_plugin)
        .add_systems(OnEnter(GameState::InGame), setup_game)
        .add_systems(Update, 
            (sprite_movement, handle_back_to_menu)
                .run_if(in_state(GameState::InGame))
        )
        .run();
}

#[derive(Component)]
enum Direction {
    Left,
    Right,
}

#[derive(Component)]
struct GameEntity;

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_image(asset_server.load("rust.png")),
        Transform::from_xyz(0., 0., 0.),
        Direction::Right,
        GameEntity,
    ));
}

fn handle_back_to_menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Clean up game entities
        for entity in game_entities.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::MainMenu);
    }
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in sprite_position.iter_mut() {
        match *logo {
            Direction::Right => transform.translation.x += 150. * time.delta_secs(),
            Direction::Left => transform.translation.x -= 150. * time.delta_secs(),
        }

        if transform.translation.x > 200. {
            *logo = Direction::Left;
        } else if transform.translation.x < -200. {
            *logo = Direction::Right;
        }
    }
}
