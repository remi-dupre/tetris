use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

use crate::{DROP_DELAY, GRID_VISIBLE_HEIGHT, GRID_WIDTH};

use super::components::{Fall, FallingPieceBundle, GridPos, PieceKind, Spin};
use super::resources::{CellState, GridState, PieceGenerator};

pub fn piece_spawn(
    mut commands: Commands,
    mut piece_generator: ResMut<PieceGenerator>,
    pieces: Query<(), (With<PieceKind>, With<Fall>)>,
) {
    if !pieces.is_empty() {
        return;
    }

    let kind = piece_generator.choose();
    let x = if kind.base_width() % 2 == 0 { 5 } else { 4 };

    let y = GRID_VISIBLE_HEIGHT.wrapping_add_signed(
        -kind
            .base_shape()
            .into_iter()
            .map(|[_, y]| y)
            .min()
            .unwrap_or(0),
    ) - 1;

    commands.spawn({
        FallingPieceBundle {
            pos: GridPos { x, y },
            kind,
            spin: Spin(0),
            fall: Fall {
                next_trigger: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
        }
    });
}

pub fn piece_fall(
    mut grid: ResMut<GridState>,
    mut commands: Commands,
    mut pieces: Query<(Entity, &PieceKind, &mut GridPos, &Spin, &mut Fall)>,
    time: Res<Time>,
) {
    for (entity, &kind, mut pos, &spin, mut fall) in &mut pieces {
        fall.next_trigger.tick(time.delta());

        for _ in 0..fall.next_trigger.times_finished_this_tick() {
            if !grid.try_move([0, -1], kind, pos.reborrow(), spin) {
                for cell in kind.piece_covered_cells(*pos.reborrow(), spin) {
                    grid.cells[usize::from(cell.x)][usize::from(cell.y)] = CellState::Full(kind);
                }

                commands.entity(entity).despawn();
                break;
            }
        }
    }
}

pub fn piece_move(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut commands: Commands,
    mut grid: ResMut<GridState>,
    mut pieces: Query<(Entity, &PieceKind, &mut GridPos, &mut Spin, &mut Fall)>,
) {
    for (entity, &kind, mut pos, mut spin, mut fall) in &mut pieces {
        for event in keyboard_input_events.read() {
            if event.state != ButtonState::Pressed {
                continue;
            }

            match &event.key_code {
                KeyCode::ArrowLeft => {
                    grid.try_move([-1, 0], kind, pos.reborrow(), *spin.reborrow());
                }
                KeyCode::ArrowRight => {
                    grid.try_move([1, 0], kind, pos.reborrow(), *spin.reborrow());
                }
                KeyCode::ArrowUp | KeyCode::KeyX => {
                    grid.try_rotate(Spin(3), kind, pos.reborrow(), spin.reborrow());
                }
                KeyCode::ControlLeft | KeyCode::ControlRight | KeyCode::KeyZ => {
                    grid.try_rotate(Spin(1), kind, pos.reborrow(), spin.reborrow());
                }
                KeyCode::Space => {
                    while grid.try_move([0, -1], kind, pos.reborrow(), *spin.reborrow()) {
                        fall.next_trigger.reset();
                    }
                }
                KeyCode::ArrowDown => {
                    if grid.try_move([0, -1], kind, pos.reborrow(), *spin.reborrow()) {
                        fall.next_trigger.reset();
                    } else {
                        for cell in kind.piece_covered_cells(*pos.reborrow(), *spin.reborrow()) {
                            grid.cells[usize::from(cell.x)][usize::from(cell.y)] =
                                CellState::Full(kind);
                        }

                        commands.entity(entity).despawn();
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn register_completed_lines(mut grid: ResMut<GridState>) {
    if !grid.is_changed() {
        return;
    }

    let mut target_line = 0;

    for y in 0..usize::from(GRID_VISIBLE_HEIGHT) {
        if (0..usize::from(GRID_WIDTH)).all(|x| matches!(grid.cells[x][y], CellState::Full(_))) {
            continue;
        }

        for x in 0..usize::from(GRID_WIDTH) {
            grid.cells[x][target_line] = std::mem::take(&mut grid.cells[x][y]);
        }

        target_line += 1;
    }

    for y in target_line..usize::from(GRID_VISIBLE_HEIGHT) {
        for x in 0..usize::from(GRID_WIDTH) {
            grid.cells[x][y] = CellState::Empty;
        }
    }
}
