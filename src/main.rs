use bevy::ui_widgets::UiWidgetsPlugins;
use bevy::{
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
    prelude::*,
};

mod menu;
mod model;
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
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: bevy::window::WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current),
                ..default()
            }),
            ..default()
        }),
        UiWidgetsPlugins,
        InputDispatchPlugin,
        TabNavigationPlugin,
        simulation::SimulationPlugin,
        ui::ReactorUiPlugin,
        sound::AudioPlugin,
        model::Reactor3dPlugin,
    ))
    .init_state::<GameState>();
    menu::main_menu(&mut app);
    app.run();
}
