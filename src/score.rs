use bevy::prelude::*;

use crate::brick::FullLineRemoveEvent;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_event::<FullLineRemoveEvent>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, score_up);
    }
}

#[derive(Debug, Component)]
pub struct ScoreText;

#[derive(Debug, Resource, Default)]
pub struct Score(u32);

pub fn setup_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Score: 0",
            TextStyle {
                font_size: 50.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            top: Val::Px(300.0),
            left: Val::Px(100.0),
            ..default()
        }),
        ScoreText,
    ));
}

pub fn score_up(
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    mut event_reader: EventReader<FullLineRemoveEvent>,
    mut score: ResMut<Score>,
) {
    if event_reader.is_empty() {
        return;
    }
    let fullline_remove_event = event_reader.iter().next().unwrap();

    score.0 += if fullline_remove_event.0 == 4 {
        2_u32.pow(fullline_remove_event.0 as u32) as u32
    } else {
        2_u32.pow((fullline_remove_event.0 - 1) as u32) as u32
    };

    let mut text = score_text_query.single_mut();
    text.sections[0].value = format!("Score: {}", score.0);
}
