use bevy::{prelude::*, window::close_on_esc, DefaultPlugins};
use bevy_tetris::brick::BrickPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BrickPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
