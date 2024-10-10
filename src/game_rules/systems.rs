use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

use crate::{GRID_VISIBLE_HEIGHT, GRID_WIDTH};

use super::components::*;
use super::events::*;
use super::resources::*;

pub fn piece_spawn(
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
                next_trigger: Timer::new(xp.time_per_row(), TimerMode::Repeating),
            },
        },
    ));
}

pub fn piece_fall(
    mut grid: ResMut<GridState>,
    mut commands: Commands,
    mut piece: Query<(Entity, &PieceKind, &mut GridPos, &Spin, &mut Fall)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((entity, &kind, mut pos, &spin, mut fall)) = piece.get_single_mut() else {
        return;
    };

    let delta = {
        if keyboard.any_pressed([KeyCode::ArrowDown]) {
            let min_speedup = (fall.next_trigger.duration()).div_duration_f64(SOFT_DROP_MAX_DELAY);
            time.delta()
                .mul_f64(f64::from(SOFT_DROP_SPEEDUP).max(min_speedup))
        } else {
            time.delta()
        }
    };

    fall.next_trigger.tick(delta);

    for _ in 0..fall.next_trigger.times_finished_this_tick() {
        if !grid.try_move([0, -1], kind, pos.reborrow(), spin) {
            for cell in kind.piece_covered_cells(*pos.reborrow(), spin) {
                grid.spawn_cell(&mut commands, &cell, kind);
            }

            commands.entity(entity).despawn_recursive();
            break;
        }
    }
}

pub fn piece_move(
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

pub fn register_completed_lines(
    mut commands: Commands,
    mut cleared_lines: EventWriter<ClearedLines>,
    mut rows_to_delete: ResMut<RowsToDelete>,
    grid: ResMut<GridState>,
) {
    if !grid.is_changed() {
        return;
    }

    let mut lines_count = 0;

    for y in 0..GRID_VISIBLE_HEIGHT {
        if (0..GRID_WIDTH).all(|x| grid.is_filled(&GridPos { x, y })) {
            rows_to_delete.0.push(y);
            lines_count += 1;
            continue;
        }
    }

    if !rows_to_delete.0.is_empty() {
        commands.init_resource::<PausedForRows>();

        cleared_lines.send(ClearedLines { lines_count });
    }
}

pub fn consume_queued_lines(
    mut commands: Commands,
    mut grid: ResMut<GridState>,
    mut rows_to_delete: ResMut<RowsToDelete>,
) {
    let mut target_line = 0;

    for y in 0..GRID_VISIBLE_HEIGHT {
        if rows_to_delete.0.contains(&y) {
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

    rows_to_delete.0.clear();
}

// -- Score and Leveling

pub fn update_score(
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

pub fn update_xp(mut cleared_lines: EventReader<ClearedLines>, mut xp: ResMut<XP>) {
    for clear in cleared_lines.read() {
        xp.0 += u32::from(clear.lines_count);
    }
}
