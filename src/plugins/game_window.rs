use std::collections::HashMap;
use std::sync::LazyLock;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::WindowResolution;
use rand::Rng;

use crate::plugins::game_rules::{
    Cell, CellState, FallingPiece, GridState, PieceKind, GRID_VISIBLE_HEIGHT, GRID_WIDTH,
};

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

static PIECE_COLOR: LazyLock<HashMap<PieceKind, Color>> = LazyLock::new(|| {
    PieceKind::all()
        .into_iter()
        .map(|kind| {
            let color = match kind {
                PieceKind::I => Color::srgb(0.0, 1.0, 1.0), // cyan
                PieceKind::O => Color::srgb(1.0, 1.0, 0.0), // yellow
                PieceKind::T => Color::srgb(0.5, 0.0, 0.5), // purple
                PieceKind::S => Color::srgb(0.0, 1.0, 0.0), // green
                PieceKind::Z => Color::srgb(1.0, 0.0, 0.0), // red
                PieceKind::J => Color::srgb(1.0, 0.5, 0.0), // orange
                PieceKind::L => Color::srgb(0.0, 0.0, 1.0), // blue
            };

            (kind, color)
        })
        .collect()
});

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
}

fn tile_offset(x: u8, y: u8, z: f32) -> Transform {
    Transform::default().with_translation(
        GRID_POSITION
            + Vec3::new(
                CELL_SIZE / 2.0 + (1.1 * CELL_SIZE) * f32::from(x),
                CELL_SIZE / 2.0 + (1.1 * CELL_SIZE) * f32::from(y),
                z,
            ),
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

    commands.insert_resource(MeshCollection {
        square: meshes.add(Rectangle::from_length(CELL_SIZE)),
    });

    materials.insert(&*BACKGROUND_COLOR, Color::srgb(0.2, 0.2, 0.2).into());

    for kind in PieceKind::all() {
        let color = PIECE_COLOR[&kind];
        materials.insert(&PIECE_MATERIAL[&kind], color.into());
        materials.insert(
            &GHOST_MATERIAL[&kind],
            color.mix(&Color::WHITE, 0.8).with_alpha(0.5).into(),
        );
    }
}

fn setup_background(mut commands: Commands, meshes: Res<MeshCollection>) {
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_VISIBLE_HEIGHT {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.square.clone().into(),
                    transform: tile_offset(x, y, -1.0),
                    material: BACKGROUND_COLOR.clone(),
                    ..Default::default()
                },
                Cell(x, y),
            ));
        }
    }
}

fn update_background(mut cells: Query<(Mut<Handle<ColorMaterial>>, &Cell)>, grid: Res<GridState>) {
    if !grid.is_changed() {
        return;
    }

    for (mut material, cell) in cells.iter_mut() {
        let Cell(x, y) = cell;

        if let CellState::Full(kind) = &grid.cells[usize::from(*x)][usize::from(*y)] {
            *material = PIECE_MATERIAL[kind].clone();
        } else {
            *material = BACKGROUND_COLOR.clone();
        }
    }
}

#[derive(Component)]
pub struct PieceTile;

fn update_piece(
    mut commands: Commands,
    meshes: Res<MeshCollection>,
    piece: Option<Res<FallingPiece>>,
    tiles: Query<Entity, With<PieceTile>>,
) {
    let Some(piece) = piece else {
        for old_tile in &tiles {
            commands.entity(old_tile).despawn();
        }

        return;
    };

    if !piece.is_changed() {
        return;
    }

    for old_tile in &tiles {
        commands.entity(old_tile).despawn();
    }

    for cell in piece.iter_cells() {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.square.clone().into(),
                transform: tile_offset(cell.0, cell.1, 100.0),
                material: PIECE_MATERIAL[&piece.kind].clone(),
                ..Default::default()
            },
            PieceTile,
        ));
    }
}

#[derive(Component)]
pub struct GhostTile;

fn update_ghost(
    mut commands: Commands,
    meshes: Res<MeshCollection>,
    piece: Option<Res<FallingPiece>>,
    grid: Res<GridState>,
    tiles: Query<Entity, With<GhostTile>>,
) {
    let Some(piece) = piece else {
        for old_tile in &tiles {
            commands.entity(old_tile).despawn();
        }

        return;
    };

    if !piece.is_changed() {
        return;
    }

    let mut ghost = piece.clone();
    while ghost.try_move([0, -1], &grid) {}

    for old_tile in &tiles {
        commands.entity(old_tile).despawn();
    }

    for cell in ghost.iter_cells() {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.square.clone().into(),
                transform: tile_offset(cell.0, cell.1, 50.0),
                material: GHOST_MATERIAL[&piece.kind].clone(),
                ..Default::default()
            },
            GhostTile,
        ));
    }
}

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
                update_background,
                update_piece,
                update_ghost,
                button_pressed,
            ),
        );
    }
}
