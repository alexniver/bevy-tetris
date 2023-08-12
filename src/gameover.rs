use bevy::prelude::*;

use crate::app_state::AppState;

pub struct GameoverPlugin;

impl Plugin for GameoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(OnEnter(AppState::GameOver), on_gameover)
            .add_systems(OnEnter(AppState::Gaming), on_gaming);
    }
}

#[derive(Debug, Component)]
pub struct Gameover;

pub fn setup(mut commands: Commands) {
    let mut textbundle = TextBundle::from_section(
        "Game Over, press r to restart",
        TextStyle {
            font_size: 80.0,
            color: Color::rgb(1.0, 0.5, 0.0),
            ..default()
        },
    )
    .with_style(Style {
        top: Val::Px(50.0),
        left: Val::Px(250.0),
        ..default()
    });
    textbundle.visibility = Visibility::Hidden;
    commands.spawn((textbundle, Gameover));
}

pub fn on_gameover(mut query_style: Query<&mut Visibility, With<Gameover>>) {
    let mut v = query_style.single_mut();
    *v = Visibility::Visible;
}

pub fn on_gaming(mut query_style: Query<&mut Visibility, With<Gameover>>) {
    let mut v = query_style.single_mut();
    *v = Visibility::Hidden;
}
