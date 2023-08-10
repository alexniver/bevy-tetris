use std::ops::{Add, AddAssign, Sub, SubAssign};

use bevy::prelude::*;
use lazy_static::*;
use rand::Rng;

pub struct BrickPlugin;

impl Plugin for BrickPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<BrickState>()
            .add_event::<SpawnEvent>()
            .add_event::<StableEvent>()
            .add_event::<ShiftEvent>()
            .add_systems(Startup, setup_board)
            .add_systems(Startup, setup_spawn)
            .add_systems(Startup, setup_fall_timer)
            .add_systems(Update, input)
            .add_systems(Update, brick_auto_fall)
            .add_systems(Update, brick_shift)
            .add_systems(Update, brick_stable)
            .add_systems(Update, brick_gen);
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

#[derive(Debug, Default, Component, Clone, Copy, PartialEq, Eq)]
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
pub struct ShiftEvent([BrickPos; 4]);

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

fn brick_gen(
    mut commands: Commands,
    mut brick_state: ResMut<BrickState>,
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

    for brick_pos in brick_shape.brick_pos_arr.iter() {
        let brick_pos_spawn = BrickPos::new(SPAWN_X + brick_pos.x, SPAWN_Y + brick_pos.y);
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
}

fn input(
    query_brick_movable: Query<&mut BrickPos, With<BrickMoveable>>,
    query_brick_stable: Query<&BrickPos, Without<BrickMoveable>>,
    keys: Res<Input<KeyCode>>,
    mut brick_state: ResMut<BrickState>,
    mut event_writer_stable: EventWriter<StableEvent>,
    mut event_writer_move: EventWriter<ShiftEvent>,
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
        println!(
            "brick shape: {:?}, idx: {:?}, idx_new: {:?}, brick_origin_pos: {:?}",
            brick_shape,
            brick_state.brick_shape_index,
            brick_shape_idx_new,
            brick_state.brick_pos_origin,
        );
        brick_state.brick_shape_index = brick_shape_idx_new;
        let brick_pos_new_arr = brick_shape
            .brick_pos_arr
            .iter()
            .map(|&pos| pos + brick_state.brick_pos_origin)
            .collect::<Vec<BrickPos>>();
        if !is_legal(&brick_pos_new_arr, &brick_stable_arr) {
            return;
        }

        event_writer_move.send(ShiftEvent(brick_pos_new_arr.try_into().unwrap()));
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

    event_writer_move.send(ShiftEvent(brick_pos_new_arr.try_into().unwrap()));
}

fn brick_auto_fall(
    query_brick_movable: Query<&BrickPos, With<BrickMoveable>>,
    query_brick_stable: Query<&BrickPos, Without<BrickMoveable>>,
    mut fall_timer: ResMut<FallTimer>,
    time: Res<Time>,
    mut brick_state: ResMut<BrickState>,
    mut event_writer_stable: EventWriter<StableEvent>,
    mut event_writer_shift: EventWriter<ShiftEvent>,
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

        event_writer_shift.send(ShiftEvent(brick_pos_new_arr.try_into().unwrap()));
    }
}

fn brick_shift(
    mut query_brick_movable: Query<(&mut Transform, &mut BrickPos), With<BrickMoveable>>,
    mut shift_event: EventReader<ShiftEvent>,
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
    query: Query<Entity, With<BrickMoveable>>,
    mut stable_event_reader: EventReader<StableEvent>,
    mut spawn_event_writer: EventWriter<SpawnEvent>,
) {
    if stable_event_reader.is_empty() {
        return;
    }
    stable_event_reader.clear();

    for entity in query.iter() {
        commands.entity(entity).remove::<BrickMoveable>();
    }

    spawn_event_writer.send(SpawnEvent);
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
