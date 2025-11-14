use bevy::ui_widgets::UiWidgetsPlugins;
use bevy::{
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
    prelude::*,
};

mod menu;
mod simulation;
mod sound;
mod ui;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Paused,
    Credits,
    GameOver,
    Settings,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        UiWidgetsPlugins,
        InputDispatchPlugin,
        TabNavigationPlugin,
        simulation::SimulationPlugin,
        ui::ReactorUiPlugin,
        sound::AudioPlugin,
    ))
    .init_state::<GameState>();
    menu::main_menu_plugin(&mut app);
    app.run();
}
