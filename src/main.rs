use bevy::{prelude::*, window::close_on_esc, DefaultPlugins};
use bevy_tetris::{
    app_state::AppState, brick::BrickPlugin, gameover::GameoverPlugin, score::ScorePlugin,
};

fn main() {
    App::new()
        .add_state::<AppState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(BrickPlugin)
        .add_plugins(ScorePlugin)
        .add_plugins(GameoverPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
