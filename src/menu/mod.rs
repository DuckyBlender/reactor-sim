mod credits;
mod settings;
mod main_menu;

use bevy::prelude::*;

use crate::menu::{credits::CreditsMenuPlugin, main_menu::MainMenuPlugin, settings::SettingsMenuPlugin};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(CreditsMenuPlugin)
        .add_plugins(SettingsMenuPlugin)
        .add_plugins(MainMenuPlugin);
    }
}
