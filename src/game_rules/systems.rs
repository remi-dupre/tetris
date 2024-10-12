use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

use crate::{GRID_VISIBLE_HEIGHT, GRID_WIDTH};

use super::components::*;
use super::events::*;
use super::resources::*;

pub(crate) fn piece_spawn(
    mut commands: Commands,
    mut piece_generator: ResMut<PieceGenerator>,
    pieces: Query<(), (With<PieceKind>, With<Fall>)>,
    xp: Res<XP>,
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
    );

    commands.spawn((
        Name::new("Falling Piece"),
        FallingPieceBundle {
            pos: GridPos { x, y },
            kind,
            spin: Spin(0),
            fall: Fall {
                down_timer: Timer::new(xp.time_per_row(), TimerMode::Repeating),
                lock_timer: Timer::new(LOCK_DELAY, TimerMode::Once),
            },
        },
    ));
}

pub(crate) fn piece_lock(
    mut grid: ResMut<GridState>,
    mut commands: Commands,
    mut piece: Query<(Entity, &PieceKind, &GridPos, &Spin, &mut Fall)>,
    time: Res<Time>,
) {
    let Ok((entity, &kind, pos, &spin, mut fall)) = piece.get_single_mut() else {
        return;
    };

    // Check if the piece is lying on the ground
    if grid.try_move([0, -1], kind, &mut pos.clone(), spin) {
        fall.lock_timer.reset();
        return;
    }

    fall.lock_timer.tick(time.delta());

    if fall.lock_timer.finished() {
        for cell in kind.piece_covered_cells(*pos, spin) {
            grid.spawn_cell(&mut commands, &cell, kind);
        }

        commands.entity(entity).despawn_recursive();
    }
}

pub(crate) fn piece_fall(
    grid: Res<GridState>,
    mut piece: Query<(&PieceKind, &mut GridPos, &Spin, &mut Fall)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((&kind, mut pos, &spin, mut fall)) = piece.get_single_mut() else {
        return;
    };

    let delta = {
        if keyboard.any_pressed([KeyCode::ArrowDown]) {
            let min_speedup = (fall.down_timer.duration()).div_duration_f64(SOFT_DROP_MAX_DELAY);
            time.delta()
                .mul_f64(f64::from(SOFT_DROP_SPEEDUP).max(min_speedup))
        } else {
            time.delta()
        }
    };

    fall.down_timer.tick(delta);

    for _ in 0..fall.down_timer.times_finished_this_tick() {
        grid.try_move([0, -1], kind, pos.reborrow(), spin);
    }
}

pub(crate) fn piece_move(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut commands: Commands,
    mut grid: ResMut<GridState>,
    mut pieces: Query<(Entity, &PieceKind, &mut GridPos, &mut Spin), With<Fall>>,
) {
    for (entity, &kind, mut pos, mut spin) in &mut pieces {
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
                    grid.try_rotate_right(kind, pos.reborrow(), spin.reborrow());
                }
                KeyCode::ControlLeft | KeyCode::ControlRight | KeyCode::KeyZ => {
                    grid.try_rotate_left(kind, pos.reborrow(), spin.reborrow());
                }
                KeyCode::Space => {
                    while grid.try_move([0, -1], kind, pos.reborrow(), *spin) {}

                    for cell in kind.piece_covered_cells(*pos.reborrow(), *spin) {
                        grid.spawn_cell(&mut commands, &cell, kind);
                    }

                    commands.entity(entity).despawn_recursive();
                    break;
                }
                _ => {}
            }
        }
    }
}

pub(crate) fn register_completed_lines(
    mut commands: Commands,
    mut cleared_lines: EventWriter<ClearedLines>,
    grid: ResMut<GridState>,
) {
    if !grid.is_changed() {
        return;
    }

    let mut rows_to_delete = Vec::new();

    for y in 0..GRID_VISIBLE_HEIGHT {
        if (0..GRID_WIDTH).all(|x| grid.is_filled(&GridPos { x, y })) {
            rows_to_delete.push(y);
            continue;
        }
    }

    if !rows_to_delete.is_empty() {
        cleared_lines.send(ClearedLines {
            lines_count: u8::try_from(rows_to_delete.len()).unwrap(),
        });

        commands.insert_resource(PausedForClear {
            timer: Timer::new(CLEAR_DELAY, TimerMode::Once),
            rows_to_delete,
        });
    }
}

pub(crate) fn resume_after_clear(
    mut commands: Commands,
    mut pause: ResMut<PausedForClear>,
    mut grid: ResMut<GridState>,
    time: Res<Time>,
) {
    pause.timer.tick(time.delta());

    if !pause.timer.finished() {
        return;
    }

    let mut target_line = 0;

    for y in 0..GRID_VISIBLE_HEIGHT {
        if pause.rows_to_delete.contains(&y) {
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

    for y in target_line..GRID_VISIBLE_HEIGHT {
        for x in 0..GRID_WIDTH {
            grid.despawn_cell(&mut commands, &GridPos { x, y });
        }
    }

    commands.remove_resource::<PausedForClear>()
}

// -- Score and Leveling

pub(crate) fn update_score(
    mut cleared_lines: EventReader<ClearedLines>,
    mut score: ResMut<Score>,
    xp: Res<XP>,
) {
    for clear in cleared_lines.read() {
        let base_delta = match clear.lines_count {
            0 => 0,
            1 => 40,
            2 => 100,
            3 => 300,
            _ => 1200,
        };

        score.0 += u64::from(xp.level()) * base_delta;
    }
}

pub(crate) fn update_xp(mut cleared_lines: EventReader<ClearedLines>, mut xp: ResMut<XP>) {
    for clear in cleared_lines.read() {
        xp.0 += u32::from(clear.lines_count);
    }
}
