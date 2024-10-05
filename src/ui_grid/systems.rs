use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use crate::game_rules::components::{Fall, FilledCell, GridPos, PieceKind, Spin};
use crate::game_rules::resources::GridState;
use crate::{GRID_VISIBLE_HEIGHT, GRID_WIDTH};

use super::components::{AlignedOnCellCenter, BackgroundTile, PieceGhost, PieceTile};
use super::resources::{MaterialCollection, MeshCollection};
use super::{tile_translation, CELL_SIZE};

// -- Camera

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
                ..Camera::default()
            },
            ..Camera2dBundle::default()
        },
    ));
}

// -- Background

pub fn setup_background(
    mut commands: Commands,
    materials: Res<MaterialCollection>,
    meshes: Res<MeshCollection>,
) {
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_VISIBLE_HEIGHT {
            commands.spawn((
                Name::new(format!("Background Tile at ({x}, {y})")),
                MaterialMesh2dBundle {
                    mesh: meshes.square.clone().into(),
                    transform: Transform::default().with_translation(tile_translation(x, y, -1.0)),
                    material: materials.background.clone(),
                    ..Default::default()
                },
                BackgroundTile,
                GridPos { x, y },
            ));
        }
    }
}

// -- Filled Cell's sprites

pub fn attach_filled_cell_sprite(
    mut commands: Commands,
    materials: Res<MaterialCollection>,
    meshes: Res<MeshCollection>,
    newly_filled_cells: Query<(Entity, &GridPos, &FilledCell), Added<FilledCell>>,
) {
    for (entity, pos, filled) in &newly_filled_cells {
        commands.entity(entity).insert(MaterialMesh2dBundle {
            mesh: meshes.square.clone().into(),
            transform: Transform::default().with_translation(tile_translation(pos.x, pos.y, 1.0)),
            material: materials.pieces[filled.color_from_kind].clone(),
            ..Default::default()
        });
    }
}

// -- Piece tile

pub fn attach_piece_sprite(
    mut commands: Commands,
    materials: Res<MaterialCollection>,
    meshes: Res<MeshCollection>,
    pieces: Query<(Entity, &PieceKind), Added<Fall>>,
) {
    for (entity, &kind) in &pieces {
        let mut cmd = commands.entity(entity);

        cmd.insert((
            MaterialMesh2dBundle {
                mesh: meshes.pieces_small_blocks[kind].clone().into(),
                material: materials.pieces[kind].clone(),
                ..Default::default()
            },
            PieceTile,
        ));

        if kind.base_width() % 2 == 0 {
            cmd.insert(AlignedOnCellCenter);
        }
    }
}

// -- Piece ghost's tile

pub fn attach_piece_ghost(
    mut commands: Commands,
    materials: Res<MaterialCollection>,
    meshes: Res<MeshCollection>,
    pieces: Query<(Entity, &PieceKind, &GridPos, &Spin), Added<Fall>>,
) {
    for (entity, &kind, &pos, &spin) in &pieces {
        let mut cmd = commands.spawn((
            Name::new("Ghost Piece"),
            MaterialMesh2dBundle {
                mesh: meshes.pieces_small_blocks[kind].clone().into(),
                material: materials.ghosts[kind].clone(),
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
    }
}

pub fn remove_hanging_piece_ghost(
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
pub fn update_ghost_pos(
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

pub fn update_ghost_spin(
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
pub fn apply_sprite_pos(
    mut pieces: Query<
        (&GridPos, Has<AlignedOnCellCenter>, &mut Transform),
        Or<(Added<Transform>, Changed<GridPos>)>,
    >,
) {
    for (pos, aligned_on_cell_center, mut transform) in &mut pieces {
        transform.translation = tile_translation(pos.x, pos.y, 100.0);

        if aligned_on_cell_center {
            transform.translation += Vec3::new(-0.5 * 1.1 * CELL_SIZE, -0.5 * 1.1 * CELL_SIZE, 0.0);
        }
    }
}

pub fn apply_sprite_angle(mut pieces: Query<(&Spin, &mut Transform), Changed<Spin>>) {
    for (Spin(angle), mut transform) in &mut pieces {
        transform.rotation = Quat::from_rotation_z(f32::from(*angle) * std::f32::consts::PI / 2.0);
    }
}

// Window controls
// TODO: they might deserve their own plugin

pub fn button_pressed(
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
