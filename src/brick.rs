use std::ops::{Add, AddAssign, Sub, SubAssign};

use bevy::{prelude::*, utils::HashMap};
use lazy_static::*;
use rand::Rng;

use crate::app_state::AppState;

pub struct BrickPlugin;

impl Plugin for BrickPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<BrickState>()
            .add_event::<SpawnEvent>()
            .add_event::<StableEvent>()
            .add_event::<NewPosEvent>()
            .add_event::<FullLineCheckEvent>()
            .add_event::<FullLineRemoveEvent>()
            .add_event::<GameOverEvent>()
            .add_event::<RestartEvent>()
            .add_systems(Startup, (setup_board, setup_spawn, setup_fall_timer))
            .add_systems(
                Update,
                (
                    brick_gen,
                    brick_auto_fall,
                    input.after(brick_auto_fall),
                    brick_apply_new_pos.after(input),
                    brick_stable.after(brick_apply_new_pos),
                )
                    .run_if(in_state(AppState::Gaming)),
            )
            .add_systems(Update, restart.run_if(in_state(AppState::GameOver)))
            .add_systems(
                PostUpdate,
                brick_fullline_clear.run_if(in_state(AppState::Gaming)),
            );
    }
}

const BOARD_WIDTH: i8 = 10;
const BOARD_HEIGHT: i8 = 20;
const BOARD_BORDER: i8 = 5;

const GRID_WIDTH: i8 = 32;
const GRID_PADDING: i8 = 2;
const BRICK_WIDTH: i8 = GRID_WIDTH - GRID_PADDING * 2;

const START_X: i8 = -BOARD_WIDTH / 2;
const START_Y: i8 = -BOARD_HEIGHT / 2;

const SPAWN_X: i8 = BOARD_WIDTH / 2 - 2;
const SPAWN_Y: i8 = BOARD_HEIGHT - 2;

#[derive(Debug, Default, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrickPos {
    pub x: i8,
    pub y: i8,
}

impl BrickPos {
    pub fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    pub fn move_left(&mut self) {
        self.x -= 1;
    }

    pub fn move_right(&mut self) {
        self.x += 1;
    }

    pub fn move_up(&mut self) {
        self.y += 1;
    }

    pub fn move_down(&mut self) {
        self.y -= 1;
    }
}

impl Add<BrickPos> for BrickPos {
    type Output = BrickPos;

    fn add(self, rhs: BrickPos) -> Self::Output {
        BrickPos::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub<BrickPos> for BrickPos {
    type Output = BrickPos;

    fn sub(self, rhs: BrickPos) -> Self::Output {
        BrickPos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl AddAssign<BrickPos> for BrickPos {
    fn add_assign(&mut self, rhs: BrickPos) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<BrickPos> for BrickPos {
    fn sub_assign(&mut self, rhs: BrickPos) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

#[derive(Component)]
struct BrickMoveable;

#[derive(Debug)]
pub struct BrickShape {
    pub brick_pos_arr: [BrickPos; 4],
}

impl BrickShape {
    pub fn new(brick_pos_arr: [BrickPos; 4]) -> Self {
        Self { brick_pos_arr }
    }
}

pub struct BrickType {
    pub brick_shape_arr: Vec<BrickShape>,
}

impl BrickType {
    pub fn new(brick_arr: Vec<BrickShape>) -> Self {
        Self {
            brick_shape_arr: brick_arr,
        }
    }
}

#[derive(Resource, Default)]
pub struct BrickState {
    pub brick_type_index: usize,
    pub brick_shape_index: usize,
    pub brick_pos_origin: BrickPos,
}

#[derive(Event)]
pub struct SpawnEvent;
#[derive(Event)]
pub struct StableEvent;
#[derive(Event)]
pub struct NewPosEvent([BrickPos; 4]);
#[derive(Event)]
pub struct FullLineCheckEvent;
#[derive(Event)]
pub struct FullLineRemoveEvent(pub u8);
#[derive(Event)]
pub struct GameOverEvent;
#[derive(Event)]
pub struct RestartEvent;

#[derive(Debug, Resource, Default)]
pub struct FallTimer(Timer);

fn setup_board(mut commands: Commands) {
    let board_inner_width = BOARD_WIDTH as i32 * GRID_WIDTH as i32;
    let board_inner_height = BOARD_HEIGHT as i32 * GRID_WIDTH as i32;
    let board_outer_width = board_inner_width + (BOARD_BORDER as i32 * 2);
    let board_outer_height = board_inner_height + (BOARD_BORDER as i32 * 2);

    // outer board
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.8, 0.8, 0.8),
            custom_size: Some(Vec2::new(
                board_outer_width as f32,
                board_outer_height as f32,
            )),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // inner board
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.2, 0.2, 0.2),
            custom_size: Some(Vec2::new(
                board_inner_width as f32,
                board_inner_height as f32,
            )),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.1),
        ..default()
    });

    // background brick
    let brick_size = Vec2::new(BRICK_WIDTH as f32, BRICK_WIDTH as f32);
    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.2, 0.8, 0.1, 0.1),
                    custom_size: Some(brick_size),
                    ..default()
                },
                transform: get_brick_pos(x, y, 0.2),
                ..default()
            });
        }
    }
}

fn setup_spawn(mut event_writer: EventWriter<SpawnEvent>) {
    event_writer.send(SpawnEvent);
}

fn setup_fall_timer(mut commands: Commands) {
    commands.insert_resource(FallTimer(Timer::from_seconds(0.8, TimerMode::Repeating)));
}

fn get_brick_pos(x: i8, y: i8, z: f32) -> Transform {
    let xy = get_brick_pos_xy(x, y);
    Transform::from_xyz(xy.0 as f32, xy.1 as f32, z)
}

fn get_brick_pos_xy(x: i8, y: i8) -> (i32, i32) {
    (
        ((START_X + x) as i32 * GRID_WIDTH as i32 + BRICK_WIDTH as i32 / 2 + GRID_PADDING as i32),
        ((START_Y + y) as i32 * GRID_WIDTH as i32 + BRICK_WIDTH as i32 / 2 + GRID_PADDING as i32),
    )
}

fn restart(
    mut commands: Commands,
    query_brick: Query<Entity, With<BrickPos>>,
    mut state: ResMut<NextState<AppState>>,
    mut event_writer_spawn: EventWriter<SpawnEvent>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::R) {
        for entity in query_brick.iter() {
            commands.entity(entity).despawn();
        }

        state.set(AppState::Gaming);
        event_writer_spawn.send(SpawnEvent);
    }
}

fn brick_gen(
    mut commands: Commands,
    query_brick_stable: Query<&BrickPos, Without<BrickMoveable>>,
    mut brick_state: ResMut<BrickState>,
    mut game_state: ResMut<NextState<AppState>>,
    mut event_reader: EventReader<SpawnEvent>,
) {
    if event_reader.is_empty() {
        return;
    }
    event_reader.clear();

    let mut rng = rand::thread_rng();
    let brick_type_idx = rng.gen_range(0..BRICK_TYPE_ARRAY.len());
    let brick_shape_idx = 0;
    let brick_type = &BRICK_TYPE_ARRAY[brick_type_idx];
    let brick_shape = &brick_type.brick_shape_arr[brick_shape_idx];

    brick_state.brick_type_index = brick_type_idx;
    brick_state.brick_shape_index = brick_shape_idx;
    brick_state.brick_pos_origin = BrickPos::new(SPAWN_X, SPAWN_Y);

    let brick_pos_stable_arr = query_brick_stable.iter().collect::<Vec<&BrickPos>>();

    let brick_pos_spawn_arr = brick_shape
        .brick_pos_arr
        .iter()
        .map(|pos| BrickPos::new(SPAWN_X + pos.x, SPAWN_Y + pos.y))
        .collect::<Vec<BrickPos>>();

    let mut is_game_over = false;
    for brick_pos_spawn in brick_pos_spawn_arr.iter() {
        if brick_pos_stable_arr.contains(&brick_pos_spawn) {
            is_game_over = true;
        }
    }

    for brick_pos_spawn in brick_pos_spawn_arr {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.5, 1.0, 0.2),
                    custom_size: Some(Vec2::new(BRICK_WIDTH as f32, BRICK_WIDTH as f32)),
                    ..default()
                },
                transform: get_brick_pos(brick_pos_spawn.x, brick_pos_spawn.y, 1.0),
                ..default()
            },
            brick_pos_spawn,
            BrickMoveable,
        ));
    }

    if is_game_over {
        game_state.set(AppState::GameOver);
    }
}

fn input(
    query_brick_movable: Query<&mut BrickPos, With<BrickMoveable>>,
    query_brick_stable: Query<&BrickPos, Without<BrickMoveable>>,
    keys: Res<Input<KeyCode>>,
    mut brick_state: ResMut<BrickState>,
    mut event_writer_stable: EventWriter<StableEvent>,
    mut event_writer_move: EventWriter<NewPosEvent>,
) {
    if query_brick_movable.is_empty() {
        return;
    }

    let brick_move_arr = query_brick_movable.iter().collect::<Vec<&BrickPos>>();
    let brick_stable_arr = query_brick_stable.iter().collect::<Vec<&BrickPos>>();
    let mut brick_pos_move = BrickPos::default();

    // shift
    if keys.just_pressed(KeyCode::W) {
        let brick_shape_idx_new = (brick_state.brick_shape_index + 1)
            % BRICK_TYPE_ARRAY[brick_state.brick_type_index]
                .brick_shape_arr
                .len();
        let brick_shape =
            &BRICK_TYPE_ARRAY[brick_state.brick_type_index].brick_shape_arr[brick_shape_idx_new];

        let brick_pos_new_arr = brick_shape
            .brick_pos_arr
            .iter()
            .map(|&pos| pos + brick_state.brick_pos_origin)
            .collect::<Vec<BrickPos>>();
        if !is_legal(&brick_pos_new_arr, &brick_stable_arr) {
            return;
        }

        brick_state.brick_shape_index = brick_shape_idx_new;
        event_writer_move.send(NewPosEvent(brick_pos_new_arr.try_into().unwrap()));
        return;
    }

    // move
    if keys.just_pressed(KeyCode::S) {
        brick_pos_move.y = -1;
    } else if keys.just_pressed(KeyCode::A) {
        brick_pos_move.x = -1;
    } else if keys.just_pressed(KeyCode::D) {
        brick_pos_move.x = 1;
    } else if keys.just_pressed(KeyCode::Space) {
        let mut max_down = 0;
        loop {
            let down = max_down + 1;
            let brick_pos_move = BrickPos::new(0, -down);
            let brick_pos_new_arr = brick_move_arr
                .iter()
                .map(|&&pos| pos + brick_pos_move)
                .collect::<Vec<BrickPos>>();
            if !is_legal(&brick_pos_new_arr, &brick_stable_arr) {
                break;
            }
            max_down = down;
        }
        brick_pos_move.y = -max_down;
    } else {
        return;
    }

    let brick_pos_new_arr = brick_move_arr
        .iter()
        .map(|&&pos| pos + brick_pos_move)
        .collect::<Vec<BrickPos>>();

    if !is_legal(&brick_pos_new_arr, &brick_stable_arr) {
        // force down when can't move, stable all brick
        if brick_pos_move.y == -1 {
            event_writer_stable.send(StableEvent);
        }
        return;
    }

    brick_state.brick_pos_origin += brick_pos_move;

    event_writer_move.send(NewPosEvent(brick_pos_new_arr.try_into().unwrap()));
}

fn brick_auto_fall(
    query_brick_movable: Query<&BrickPos, With<BrickMoveable>>,
    query_brick_stable: Query<&BrickPos, Without<BrickMoveable>>,
    mut fall_timer: ResMut<FallTimer>,
    time: Res<Time>,
    mut brick_state: ResMut<BrickState>,
    mut event_writer_stable: EventWriter<StableEvent>,
    mut event_writer_new_pos: EventWriter<NewPosEvent>,
) {
    fall_timer.0.tick(time.delta());

    if fall_timer.0.finished() {
        if query_brick_movable.is_empty() {
            return;
        }

        let brick_move_arr = query_brick_movable.iter().collect::<Vec<&BrickPos>>();
        let brick_stable_arr = query_brick_stable.iter().collect::<Vec<&BrickPos>>();

        let brick_pos_move = BrickPos::new(0, -1);

        let brick_pos_new_arr = brick_move_arr
            .iter()
            .map(|&&pos| pos + brick_pos_move)
            .collect::<Vec<BrickPos>>();

        if !is_legal(&brick_pos_new_arr, &brick_stable_arr) {
            event_writer_stable.send(StableEvent);
            return;
        }

        brick_state.brick_pos_origin += brick_pos_move;

        event_writer_new_pos.send(NewPosEvent(brick_pos_new_arr.try_into().unwrap()));
    }
}

fn brick_apply_new_pos(
    mut query_brick_movable: Query<(&mut Transform, &mut BrickPos), With<BrickMoveable>>,
    mut shift_event: EventReader<NewPosEvent>,
) {
    if query_brick_movable.is_empty() || shift_event.is_empty() {
        return;
    }

    for shift_event in shift_event.iter() {
        let brick_pos_new_arr = shift_event.0;

        for (idx, (mut transform, mut brick_pos)) in query_brick_movable.iter_mut().enumerate() {
            brick_pos.x = brick_pos_new_arr[idx].x;
            brick_pos.y = brick_pos_new_arr[idx].y;

            let xy = get_brick_pos_xy(brick_pos.x, brick_pos.y);

            transform.translation.x = xy.0 as f32;
            transform.translation.y = xy.1 as f32;
        }
    }
}

fn brick_stable(
    mut commands: Commands,
    query_movable: Query<Entity, With<BrickMoveable>>,
    mut stable_event_reader: EventReader<StableEvent>,
    mut spawn_event_writer: EventWriter<SpawnEvent>,
    mut full_line_check_event_writer: EventWriter<FullLineCheckEvent>,
) {
    if stable_event_reader.is_empty() {
        return;
    }
    stable_event_reader.clear();

    for entity in query_movable.iter() {
        commands.entity(entity).remove::<BrickMoveable>();
    }

    spawn_event_writer.send(SpawnEvent);
    full_line_check_event_writer.send(FullLineCheckEvent);
}

fn brick_fullline_clear(
    mut commands: Commands,
    mut query_brick_stable: Query<(Entity, &mut Transform, &mut BrickPos), Without<BrickMoveable>>,
    mut full_line_check_event_reader: EventReader<FullLineCheckEvent>,
    mut full_line_remove_event_writer: EventWriter<FullLineRemoveEvent>,
) {
    if full_line_check_event_reader.is_empty() || query_brick_stable.is_empty() {
        return;
    }
    full_line_check_event_reader.clear();

    let brick_stable_arr = query_brick_stable
        .iter()
        .map(|(_, _, pos)| pos)
        .collect::<Vec<&BrickPos>>();

    // get all y to remove
    let mut y_to_remove = vec![];
    for y in 0..BOARD_HEIGHT {
        let mut is_full_line = true;
        for x in 0..BOARD_WIDTH {
            let brick_pos_tmp = BrickPos::new(x, y);
            if !brick_stable_arr.contains(&&brick_pos_tmp) {
                is_full_line = false;
                break;
            }
        }

        if is_full_line {
            y_to_remove.push(y);
        }
    }

    if y_to_remove.len() == 0 {
        return;
    }

    // remove all y line
    for (entity, _, brick_pos) in query_brick_stable.iter() {
        if y_to_remove.contains(&brick_pos.y) {
            commands.entity(entity).despawn();
        }
    }

    // get all new brick_pos for left brick_pos
    let mut left_brick_pos_new_pos_map = HashMap::new();
    let mut target_y = 0_i8;
    for y in 0..BOARD_HEIGHT {
        if y_to_remove.contains(&y) {
            continue;
        }

        let mut pos_assigned = false; // if new pos assigned in this target_y, target_y ++
        for x in 0..BOARD_WIDTH {
            let brick_pos_tmp = BrickPos::new(x, y);
            if brick_stable_arr.contains(&&brick_pos_tmp) {
                let brick_pos_new = BrickPos::new(x, target_y);
                left_brick_pos_new_pos_map.insert(brick_pos_tmp, brick_pos_new);
                pos_assigned = true;
            }
        }

        if pos_assigned {
            target_y += 1;
        }
    }

    // move left brick pos to new pos
    for (_, mut transform, mut brick_pos) in query_brick_stable.iter_mut() {
        let brick_pos = brick_pos.as_mut();
        if left_brick_pos_new_pos_map.contains_key(brick_pos) {
            brick_pos.x = left_brick_pos_new_pos_map[brick_pos].x;
            brick_pos.y = left_brick_pos_new_pos_map[brick_pos].y;

            let xy = get_brick_pos_xy(brick_pos.x, brick_pos.y);

            transform.translation.x = xy.0 as f32;
            transform.translation.y = xy.1 as f32;
        }
    }

    full_line_remove_event_writer.send(FullLineRemoveEvent(y_to_remove.len() as u8));
}

fn is_legal(brick_pos_arr_new: &Vec<BrickPos>, brick_stable_arr: &Vec<&BrickPos>) -> bool {
    for brick_pos in brick_pos_arr_new {
        if brick_pos.x < 0
            || brick_pos.x >= BOARD_WIDTH
            || brick_pos.y < 0
            || brick_pos.y >= BOARD_HEIGHT
            || brick_stable_arr.contains(&&brick_pos)
        {
            return false;
        }
    }

    return true;
}

lazy_static! {
    pub static ref BRICK_TYPE_ARRAY: Vec<BrickType> = vec![
        // quard
        BrickType::new(vec![BrickShape::new([BrickPos::new(1, 0), BrickPos::new(1, 1), BrickPos::new(2, 0), BrickPos::new(2, 1)])]),
        // line
        BrickType::new(vec![
           BrickShape::new([BrickPos::new(0, 1), BrickPos::new(1, 1), BrickPos::new(2, 1), BrickPos::new(3, 1)]),
           BrickShape::new([BrickPos::new(2, 0), BrickPos::new(2, 1), BrickPos::new(2, 2), BrickPos::new(2, 3)]),
        ]),

        // J
        BrickType::new(vec![
           BrickShape::new([BrickPos::new(0, 1), BrickPos::new(1, 1), BrickPos::new(2, 1), BrickPos::new(2, 0)]),
           BrickShape::new([BrickPos::new(1, 0), BrickPos::new(1, 1), BrickPos::new(1, 2), BrickPos::new(0, 0)]),
           BrickShape::new([BrickPos::new(0, 1), BrickPos::new(1, 1), BrickPos::new(2, 1), BrickPos::new(0, 2)]),
           BrickShape::new([BrickPos::new(1, 0), BrickPos::new(1, 1), BrickPos::new(1, 2), BrickPos::new(2, 2)]),
        ]),

        // L
        BrickType::new(vec![
           BrickShape::new([BrickPos::new(0, 1), BrickPos::new(1, 1), BrickPos::new(2, 1), BrickPos::new(0, 0)]),
           BrickShape::new([BrickPos::new(1, 0), BrickPos::new(1, 1), BrickPos::new(1, 2), BrickPos::new(0, 2)]),
           BrickShape::new([BrickPos::new(0, 1), BrickPos::new(1, 1), BrickPos::new(2, 1), BrickPos::new(2, 2)]),
           BrickShape::new([BrickPos::new(1, 0), BrickPos::new(1, 1), BrickPos::new(1, 2), BrickPos::new(2, 0)]),
        ]),

        // S
        BrickType::new(vec![
           BrickShape::new([BrickPos::new(0, 0), BrickPos::new(1, 0), BrickPos::new(1, 1), BrickPos::new(2, 1)]),
           BrickShape::new([BrickPos::new(1, 2), BrickPos::new(1, 1), BrickPos::new(2, 1), BrickPos::new(2, 0)]),
        ]),

        // Z
        BrickType::new(vec![
           BrickShape::new([BrickPos::new(0, 1), BrickPos::new(1, 1), BrickPos::new(1, 0), BrickPos::new(2, 0)]),
           BrickShape::new([BrickPos::new(2, 2), BrickPos::new(2, 1), BrickPos::new(1, 1), BrickPos::new(1, 0)]),
        ]),

        // T
        BrickType::new(vec![
           BrickShape::new([BrickPos::new(0, 1), BrickPos::new(1, 1), BrickPos::new(2, 1), BrickPos::new(1, 0)]),
           BrickShape::new([BrickPos::new(1, 0), BrickPos::new(1, 1), BrickPos::new(1, 2), BrickPos::new(0, 1)]),
           BrickShape::new([BrickPos::new(0, 1), BrickPos::new(1, 1), BrickPos::new(2, 1), BrickPos::new(1, 2)]),
           BrickShape::new([BrickPos::new(1, 0), BrickPos::new(1, 1), BrickPos::new(1, 2), BrickPos::new(2, 1)]),
        ]),


    ];
}
