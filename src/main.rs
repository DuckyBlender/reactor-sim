use bevy::ui_widgets::UiWidgetsPlugins;
use bevy::{
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
    prelude::*,
};

mod simulation;
mod ui;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    InGame,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            UiWidgetsPlugins,
            InputDispatchPlugin,
            TabNavigationPlugin,
            simulation::SimulationPlugin,
            ui::ReactorUiPlugin,
        ))
        .init_state::<GameState>()
        .run();
}
