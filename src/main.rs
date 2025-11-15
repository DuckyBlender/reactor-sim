use bevy::ui_widgets::UiWidgetsPlugins;
use bevy::{
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
    prelude::*,
};

mod menu;
mod model;
mod simulation;
mod sound;
mod tutorial;
mod ui;

pub const FONT_REGULAR: &str = "fonts/LTSuperior-Regular.ttf";
pub const FONT_MEDIUM: &str = "fonts/LTSuperior-Medium.ttf";

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Paused,
    Credits,
    GameOver,
    Tutorial,
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
        tutorial::TutorialPlugin,
        model::Reactor3dPlugin,
        menu::MenuPlugin,
    ))
    .init_state::<GameState>();
    app.run();
}
