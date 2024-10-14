use bevy::animation::AnimationTarget;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::sprite::MaterialMesh2dBundle;

use crate::common::resources::ColorPalette;
use crate::game_rules::components::{Fall, FilledCell, GridPos, PieceKind, Spin};
use crate::game_rules::resources::{GridState, PausedForClear};
use crate::WINDOW_SIZE;

use super::components::*;
use super::resources::*;
use super::tile_translation;

// -- Camera

pub(crate) fn setup_camera(mut commands: Commands, palette: Res<ColorPalette>) {
    let mut camera = Camera2dBundle::default();
    camera.camera.clear_color = ClearColorConfig::Custom(palette.background_2.color);

    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: WINDOW_SIZE[0],
        min_height: WINDOW_SIZE[1],
    };

    commands.spawn((Name::new("Main Camera"), camera));
}

// -- Static decoration

pub(crate) fn draw_frame(
    mut commands: Commands,
    meshes: Res<MeshCollection>,
    palette: Res<ColorPalette>,
    root: Res<UiGridRoot>,
) {
    commands
        .spawn((
            Name::new("Grid Frame"),
            MaterialMesh2dBundle {
                mesh: meshes.frame.clone().into(),
                material: palette.background_2.material.clone(),
                ..Default::default()
            },
        ))
        .set_parent(**root);
}

pub(crate) fn draw_background_grid(
    mut commands: Commands,
    meshes: Res<MeshCollection>,
    palette: Res<ColorPalette>,
    root: Res<UiGridRoot>,
) {
    commands
        .spawn((
            Name::new("Background Color"),
            MaterialMesh2dBundle {
                mesh: meshes.grid_background.clone().into(),
                material: palette.background_1.material.clone(),
                ..Default::default()
            },
        ))
        .set_parent(**root);

    commands
        .spawn((
            Name::new("Background Grid"),
            MaterialMesh2dBundle {
                mesh: meshes.grid.clone().into(),
                material: palette.background_2.material.clone(),
                ..Default::default()
            },
        ))
        .set_parent(**root);
}

// -- Filled Cell's sprites

pub(crate) fn attach_filled_cell_sprite(
    mut commands: Commands,
    root: Res<UiGridRoot>,
    palette: Res<ColorPalette>,
    meshes: Res<MeshCollection>,
    newly_filled_cells: Query<(Entity, &GridPos, &FilledCell), Added<FilledCell>>,
    animations: Res<AnimationCollection>,
) {
    if newly_filled_cells.is_empty() {
        return;
    }

    let mut player = AnimationPlayer::default();
    player.play(animations.inflate.node);

    let player_entity = commands
        .spawn((
            Name::new("Attached Cell Animation Player"),
            OneShotPlayer,
            player,
            animations.inflate.graph.clone(),
        ))
        .id();

    for (entity, pos, filled) in &newly_filled_cells {
        commands
            .entity(entity)
            .insert((
                MaterialMesh2dBundle {
                    mesh: meshes.square.clone().into(),
                    transform: Transform::default()
                        .with_translation(tile_translation(pos.x, pos.y, 0.0)),
                    material: palette.pieces[filled.color_from_kind].material.clone(),
                    ..Default::default()
                },
                AnimationTarget {
                    id: animations.inflate.animation_target_id,
                    player: player_entity,
                },
            ))
            .set_parent(**root); // TODO: should be a separate entity ?
    }
}

// -- Run animation on cleared lines

pub(crate) fn start_clear_line_animation(
    mut commands: Commands,
    animations: Res<AnimationCollection>,
    cells: Query<(Entity, &GridPos), With<FilledCell>>,
    pause: Res<PausedForClear>,
) {
    let mut player = AnimationPlayer::default();

    player
        .play(animations.blink.node)
        .set_speed(1.0 / pause.timer.duration().as_secs_f32());

    let player_entity = commands
        .spawn((
            Name::new("Blink Completed Lines Player"),
            OneShotPlayer,
            player,
            animations.blink.graph.clone(),
        ))
        .id();

    for (entity, _) in cells
        .iter()
        .filter(|(_, pos)| pause.rows_to_delete.contains(&pos.y))
    {
        commands.entity(entity).insert(AnimationTarget {
            id: animations.blink.animation_target_id,
            player: player_entity,
        });
    }
}

pub(crate) fn cleanup_finished_oneshot_players(
    mut commands: Commands,
    players: Query<(Entity, &AnimationPlayer), With<OneShotPlayer>>,
) {
    for (entity, player) in &players {
        if player.all_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// -- Piece tile

pub(crate) fn attach_piece_sprite(
    mut commands: Commands,
    root: Res<UiGridRoot>,
    palette: Res<ColorPalette>,
    meshes: Res<MeshCollection>,
    pieces: Query<(Entity, &PieceKind), Added<Fall>>,
) {
    for (entity, &kind) in &pieces {
        let mut cmd = commands.entity(entity);

        cmd.insert((
            PieceTile,
            MaterialMesh2dBundle {
                mesh: meshes.pieces_small_blocks[kind].clone().into(),
                material: palette.pieces[kind].material.clone(),
                transform: Transform::from_translation([0.0, 0.0, 100.0].into()),
                ..Default::default()
            },
        ))
        .set_parent(**root); // TODO: should be a separate entity

        if kind.base_width() % 2 == 0 {
            cmd.insert(AlignedOnCellCenter);
        }
    }
}

// -- Piece ghost's tile

pub(crate) fn attach_piece_ghost(
    mut commands: Commands,
    root: Res<UiGridRoot>,
    palette: Res<ColorPalette>,
    meshes: Res<MeshCollection>,
    pieces: Query<(Entity, &PieceKind, &GridPos, &Spin), Added<Fall>>,
) {
    for (entity, &kind, &pos, &spin) in &pieces {
        let mut cmd = commands.spawn((
            Name::new("Ghost Piece"),
            MaterialMesh2dBundle {
                mesh: meshes.pieces_small_blocks[kind].clone().into(),
                material: palette.ghosts[kind].material.clone(),
                ..Default::default()
            },
            kind,
            pos,
            spin,
            PieceGhost(entity),
        ));

        if kind.base_width() % 2 == 0 {
            cmd.insert(AlignedOnCellCenter);
        }

        cmd.set_parent(**root);
    }
}

pub(crate) fn remove_hanging_piece_ghost(
    mut commands: Commands,
    ghosts: Query<(Entity, &PieceGhost)>,
    entities: Query<Entity>,
) {
    for (entity, PieceGhost(parent)) in &ghosts {
        if !entities.contains(*parent) {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn update_ghost_pos(
    grid: Res<GridState>,
    pieces: Query<
        (&PieceKind, &GridPos, &Spin),
        (
            With<Fall>,
            Or<(Changed<GridPos>, Changed<Spin>, Added<Fall>)>,
        ),
    >,
    mut ghosts: Query<(&PieceGhost, &mut GridPos), Without<Fall>>,
) {
    for (ghost, mut pos) in &mut ghosts {
        let Ok((&kind, &piece_pos, &spin)) = pieces.get(ghost.0) else {
            continue;
        };

        *pos = piece_pos;
        while grid.try_move([0, -1], kind, pos.reborrow(), spin) {}
    }
}

pub(crate) fn update_ghost_spin(
    pieces: Query<&Spin, (With<Fall>, Changed<Spin>)>,
    mut ghosts: Query<(&PieceGhost, &mut Spin), Without<Fall>>,
) {
    for (&PieceGhost(parent), mut spin) in &mut ghosts {
        let Ok(piece_spin) = pieces.get(parent) else {
            continue;
        };

        *spin = *piece_spin;
    }
}

// -- Update transformations

#[allow(clippy::type_complexity)]
pub(crate) fn apply_sprite_pos(
    mut pieces: Query<
        (&GridPos, Has<AlignedOnCellCenter>, &mut Transform),
        Or<(Added<Transform>, Changed<GridPos>)>,
    >,
) {
    for (pos, aligned_on_cell_center, mut transform) in &mut pieces {
        transform.translation = tile_translation(pos.x, pos.y, transform.translation.z);

        if aligned_on_cell_center {
            transform.translation += Vec3::new(-0.5 * CELL_SIZE, -0.5 * CELL_SIZE, 0.0);
        }
    }
}

pub(crate) fn apply_sprite_angle(mut pieces: Query<(&Spin, &mut Transform), Changed<Spin>>) {
    for (&Spin(spin), mut transform) in &mut pieces {
        transform.rotation = Quat::from_rotation_z(-f32::from(spin) * std::f32::consts::PI / 2.0);
    }
}

// Window controls
// TODO: they might deserve their own plugin

pub(crate) fn button_pressed(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut exit: EventWriter<AppExit>,
) {
    for event in keyboard_input_events.read() {
        match &event.logical_key {
            Key::Character(c) if ["q", "Q"].contains(&c.as_str()) => {
                exit.send(AppExit::Success);
            }
            _ => {}
        }
    }
}
