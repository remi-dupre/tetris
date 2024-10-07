use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

use crate::{DROP_DELAY, GRID_VISIBLE_HEIGHT, GRID_WIDTH};

use super::components::{Fall, FallingPieceBundle, GridPos, PieceKind, Spin};
use super::resources::{GridState, PieceGenerator, Score};

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
                    grid.spawn_cell(&mut commands, &cell, kind);
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
                            grid.spawn_cell(&mut commands, &cell, kind);
                        }

                        commands.entity(entity).despawn();
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn register_completed_lines(
    mut commands: Commands,
    mut grid: ResMut<GridState>,
    mut score: ResMut<Score>,
) {
    if !grid.is_changed() {
        return;
    }

    let mut target_line = 0;

    for y in 0..GRID_VISIBLE_HEIGHT {
        if (0..GRID_WIDTH).all(|x| grid.is_filled(&GridPos { x, y })) {
            continue;
        }

        for x in 0..GRID_WIDTH {
            grid.move_to(
                &mut commands,
                &GridPos { x, y },
                &GridPos { x, y: target_line },
            );
        }

        target_line += 1;
    }

    let cleared_lines = GRID_VISIBLE_HEIGHT - target_line;

    for y in target_line..GRID_VISIBLE_HEIGHT {
        for x in 0..GRID_WIDTH {
            grid.despawn_cell(&mut commands, &GridPos { x, y });
        }
    }

    // TODO: Handle scoring through event

    match cleared_lines {
        0 => {}
        1 => score.0 += 40,
        2 => score.0 += 100,
        3 => score.0 += 300,
        _ => score.0 += 1200,
    }
}
