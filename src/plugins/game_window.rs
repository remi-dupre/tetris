use std::collections::HashMap;
use std::sync::LazyLock;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::WindowResolution;
use enum_map::EnumMap;
use rand::Rng;

use crate::plugins::game_rules::{
    CellState, GridState, PieceKind, GRID_VISIBLE_HEIGHT, GRID_WIDTH,
};

use crate::plugins::game_rules::{try_move, Fall, GridPos, Spin};

use super::game_rules::UpdateGame;

const WINDOW_TITLE: &str = "Tetris (Bevy Engine)";
const WINDOW_CLASS: &str = "org.remi-dupre.testing";
const WINDOW_SIZE: [f32; 2] = [400., 600.];

const GRID_POSITION: Vec3 = Vec3::new(
    -WINDOW_SIZE[0] / 2.0 + 10.0,
    -WINDOW_SIZE[1] / 2.0 + 10.0,
    0.,
);

const CELL_SIZE: f32 = 24.;

static BACKGROUND_COLOR: LazyLock<Handle<ColorMaterial>> =
    LazyLock::new(|| Handle::weak_from_u128(rand::random()));

static PIECE_MATERIAL: LazyLock<HashMap<PieceKind, Handle<ColorMaterial>>> = LazyLock::new(|| {
    let mut rng = rand::thread_rng();

    PieceKind::all()
        .into_iter()
        .map(|kind| {
            let handle = Handle::weak_from_u128(rng.gen());
            (kind, handle)
        })
        .collect()
});

static GHOST_MATERIAL: LazyLock<HashMap<PieceKind, Handle<ColorMaterial>>> = LazyLock::new(|| {
    let mut rng = rand::thread_rng();

    PieceKind::all()
        .into_iter()
        .map(|kind| {
            let handle = Handle::weak_from_u128(rng.gen());
            (kind, handle)
        })
        .collect()
});

#[derive(Resource)]
struct MeshCollection {
    square: Handle<Mesh>,
    piece_shapes: EnumMap<PieceKind, Handle<Mesh>>,
}

fn tile_translation(x: u8, y: u8, z: f32) -> Vec3 {
    GRID_POSITION
        + Vec3::new(
            CELL_SIZE / 2.0 + (1.1 * CELL_SIZE) * f32::from(x),
            CELL_SIZE / 2.0 + (1.1 * CELL_SIZE) * f32::from(y),
            z,
        )
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
            ..Camera::default()
        },
        ..Camera2dBundle::default()
    });

    let piece_shapes = EnumMap::from_fn(|piece_kind: PieceKind| {
        let mesh = piece_kind
            .base_shape()
            .into_iter()
            .map(|[x, y]| {
                Mesh::from(Rectangle::from_length(CELL_SIZE)).translated_by(Vec3::new(
                    (1.1 * CELL_SIZE) * f32::from(x),
                    (1.1 * CELL_SIZE) * f32::from(y),
                    0.0,
                ))
            })
            .reduce(|mut x, y| {
                x.merge(&y);
                x
            })
            .unwrap()
            .translated_by({
                if piece_kind.base_width() % 2 == 0 {
                    Vec3::new(0.5 * 1.1 * CELL_SIZE, 0.5 * 1.1 * CELL_SIZE, 0.0)
                } else {
                    Vec3::new(0.0, 0.0, 0.0)
                }
            });

        meshes.add(mesh)
    });

    commands.insert_resource(MeshCollection {
        square: meshes.add(Rectangle::from_length(CELL_SIZE)),
        piece_shapes,
    });

    materials.insert(&*BACKGROUND_COLOR, Color::srgb(0.2, 0.2, 0.2).into());

    for kind in PieceKind::all() {
        let color = match kind {
            PieceKind::I => Color::srgb(0.0, 1.0, 1.0), // cyan
            PieceKind::O => Color::srgb(1.0, 1.0, 0.0), // yellow
            PieceKind::T => Color::srgb(0.5, 0.0, 0.5), // purple
            PieceKind::S => Color::srgb(0.0, 1.0, 0.0), // green
            PieceKind::Z => Color::srgb(1.0, 0.0, 0.0), // red
            PieceKind::J => Color::srgb(1.0, 0.5, 0.0), // orange
            PieceKind::L => Color::srgb(0.0, 0.0, 1.0), // blue
        };

        materials.insert(&PIECE_MATERIAL[&kind], color.into());
        materials.insert(
            &GHOST_MATERIAL[&kind],
            color.mix(&Color::WHITE, 0.8).with_alpha(0.5).into(),
        );
    }
}

// Background handling

fn setup_background(mut commands: Commands, meshes: Res<MeshCollection>) {
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_VISIBLE_HEIGHT {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.square.clone().into(),
                    transform: Transform::default().with_translation(tile_translation(x, y, -1.0)),
                    material: BACKGROUND_COLOR.clone(),
                    ..Default::default()
                },
                GridPos { x, y },
            ));
        }
    }
}

fn update_background(
    mut cells: Query<(Mut<Handle<ColorMaterial>>, &GridPos)>,
    grid: Res<GridState>,
) {
    if !grid.is_changed() {
        return;
    }

    for (mut material, cell) in cells.iter_mut() {
        let GridPos { x, y } = cell;

        if let CellState::Full(kind) = &grid.cells[usize::from(*x)][usize::from(*y)] {
            *material = PIECE_MATERIAL[kind].clone();
        } else {
            *material = BACKGROUND_COLOR.clone();
        }
    }
}

// Piece tile

#[derive(Component)]
pub struct PieceTile;

fn attach_piece_tiles(
    mut commands: Commands,
    meshes: Res<MeshCollection>,
    pieces: Query<(Entity, &PieceKind), Added<Fall>>,
) {
    for (entity, &kind) in &pieces {
        commands.entity(entity).insert((
            MaterialMesh2dBundle {
                mesh: meshes.piece_shapes[kind].clone().into(),
                material: PIECE_MATERIAL[&kind].clone(),
                ..Default::default()
            },
            PieceTile,
        ));
    }
}

// Ghost

#[derive(Component)]
pub struct PieceGhost(Entity);

fn attach_piece_ghost(
    mut commands: Commands,
    meshes: Res<MeshCollection>,
    pieces: Query<(Entity, &PieceKind, &GridPos, &Spin), Added<Fall>>,
) {
    for (entity, &kind, &pos, &spin) in &pieces {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.piece_shapes[kind].clone().into(),
                material: GHOST_MATERIAL[&kind].clone(),
                ..Default::default()
            },
            kind,
            pos,
            spin,
            PieceGhost(entity),
        ));
    }
}

fn remove_hanging_ghosts(
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
fn update_ghost_pos(
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
        while try_move(&grid, [0, -1], kind, pos.reborrow(), spin) {}
    }
}

fn update_ghost_spin(
    pieces: Query<&Spin, (With<Fall>, Changed<Spin>)>,
    mut ghosts: Query<(&PieceGhost, &mut Spin), Without<Fall>>,
) {
    for (ghost, mut spin) in &mut ghosts {
        let Ok(piece_spin) = pieces.get(ghost.0) else {
            continue;
        };

        *spin = *piece_spin;
    }
}

// Update transformations

#[allow(clippy::type_complexity)]
fn apply_piece_pos(
    mut pieces: Query<
        (&GridPos, &PieceKind, &mut Transform),
        Or<(Added<Transform>, Changed<GridPos>)>,
    >,
) {
    for (pos, kind, mut transform) in &mut pieces {
        transform.translation = tile_translation(pos.x, pos.y, 100.0);

        if kind.base_width() % 2 == 0 {
            transform.translation += Vec3::new(-0.5 * 1.1 * CELL_SIZE, -0.5 * 1.1 * CELL_SIZE, 0.0);
        }
    }
}

fn apply_piece_angle(mut pieces: Query<(&Spin, &mut Transform), Changed<Spin>>) {
    for (Spin(angle), mut transform) in &mut pieces {
        transform.rotation = Quat::from_rotation_z(f32::from(*angle) * std::f32::consts::PI / 2.0);
    }
}

// Window controls

fn button_pressed(
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

pub struct GameWindow;

impl Plugin for GameWindow {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                name: Some(WINDOW_CLASS.to_string()),
                resizable: false,
                resolution: WindowResolution::new(WINDOW_SIZE[0], WINDOW_SIZE[1]),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, (setup, setup_background).chain())
        .add_systems(
            Update,
            (
                (
                    update_background,
                    (
                        attach_piece_tiles,
                        attach_piece_ghost,
                        remove_hanging_ghosts,
                    ),
                    (update_ghost_pos, update_ghost_spin),
                    (apply_piece_pos, apply_piece_angle),
                )
                    .chain()
                    .after(UpdateGame),
                button_pressed,
            ),
        );
    }
}
