use std::ops::Deref;

use bevy::prelude::*;
use enum_map::EnumMap;

use crate::common::resources::ColorPalette;
use crate::game_rules::components::PieceKind;
use crate::ui_grid::BORDER_SIZE;
use crate::{GRID_VISIBLE_HEIGHT, GRID_WIDTH};

use super::{CELL_SIZE, UI_GRID_VIRTUAL_HEIGHT, UI_GRID_VIRTUAL_WIDTH};

const BLOCK_SQUARE_SIZE: f32 = 0.9;
const BLOCK_SQUARE_SMALL_SIZE: f32 = 0.75;

// Config

#[derive(Resource)]
pub struct UiGridConfig {
    pub pos: [f32; 2],
    pub size: [f32; 2],
}

// -- Root
#[derive(Resource)]
pub struct UiGridRoot(Entity);

impl Deref for UiGridRoot {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromWorld for UiGridRoot {
    fn from_world(world: &mut World) -> Self {
        let config: &UiGridConfig = world.resource();

        let root = world
            .spawn((
                Name::new("Grid Panel"),
                InheritedVisibility::default(),
                Transform::default()
                    .with_translation([config.pos[0], config.pos[1], 100.0].into())
                    .with_scale(
                        [
                            config.size[0] / UI_GRID_VIRTUAL_WIDTH,
                            config.size[1] / UI_GRID_VIRTUAL_HEIGHT,
                            0.0,
                        ]
                        .into(),
                    ),
                GlobalTransform::default(),
            ))
            .id();

        Self(root)
    }
}

// MeshCollection

#[derive(Resource)]
pub struct MeshCollection {
    pub square: Handle<Mesh>,
    pub frame: Handle<Mesh>,
    pub grid: Handle<Mesh>,
    pub pieces: EnumMap<PieceKind, Handle<Mesh>>,
    pub pieces_small_blocks: EnumMap<PieceKind, Handle<Mesh>>,
}

impl FromWorld for MeshCollection {
    fn from_world(world: &mut World) -> Self {
        let frame = {
            let vertical_bar = Rectangle::new(BORDER_SIZE / 2.0, UI_GRID_VIRTUAL_HEIGHT);

            let horizontal_bar = Rectangle::new(UI_GRID_VIRTUAL_WIDTH, BORDER_SIZE / 2.0);

            let mut mesh: Mesh = Mesh::from(vertical_bar)
                .translated_by([BORDER_SIZE / 4.0 - UI_GRID_VIRTUAL_WIDTH / 2.0, 0.0, 0.0].into());

            mesh.merge(
                &Mesh::from(vertical_bar).translated_by(
                    [UI_GRID_VIRTUAL_WIDTH / 2.0 - BORDER_SIZE / 4.0, 0.0, 0.0].into(),
                ),
            );

            mesh.merge(&Mesh::from(horizontal_bar).translated_by(
                [0.0, BORDER_SIZE / 4.0 - UI_GRID_VIRTUAL_HEIGHT / 2.0, 0.0].into(),
            ));

            mesh.merge(&Mesh::from(horizontal_bar).translated_by(
                [0.0, UI_GRID_VIRTUAL_HEIGHT / 2.0 - BORDER_SIZE / 4.0, 0.0].into(),
            ));

            world.add_asset(mesh)
        };

        fn mesh_piece(
            coords: impl Iterator<Item = [i8; 2]>,
            square_size: f32,
            align_on_cell_center: bool,
        ) -> Mesh {
            coords
                .map(|[x, y]| {
                    Mesh::from(Rectangle::from_length(CELL_SIZE * square_size)).translated_by(
                        [CELL_SIZE * f32::from(x), CELL_SIZE * f32::from(y), 0.0].into(),
                    )
                })
                .reduce(|mut x, y| {
                    x.merge(&y);
                    x
                })
                .unwrap()
                .translated_by({
                    if align_on_cell_center {
                        [0.5 * CELL_SIZE, 0.5 * CELL_SIZE, 0.0].into()
                    } else {
                        [0.0, 0.0, 0.0].into()
                    }
                })
        }

        let grid = world.add_asset(
            mesh_piece(
                (0..GRID_WIDTH)
                    .flat_map(|x| (0..GRID_VISIBLE_HEIGHT).map(move |y| [x as _, y as _])),
                0.1,
                false,
            )
            .translated_by(Vec3::new(
                (BORDER_SIZE - UI_GRID_VIRTUAL_WIDTH + CELL_SIZE) / 2.0,
                (BORDER_SIZE - UI_GRID_VIRTUAL_HEIGHT + CELL_SIZE) / 2.0,
                0.0,
            )),
        );

        let pieces = EnumMap::from_fn(|piece_kind: PieceKind| {
            world.add_asset(mesh_piece(
                piece_kind.base_shape().into_iter(),
                BLOCK_SQUARE_SIZE,
                piece_kind.base_width() % 2 == 0,
            ))
        });

        let pieces_small_blocks = EnumMap::from_fn(|piece_kind: PieceKind| {
            world.add_asset(mesh_piece(
                piece_kind.base_shape().into_iter(),
                BLOCK_SQUARE_SMALL_SIZE,
                piece_kind.base_width() % 2 == 0,
            ))
        });

        Self {
            square: world.add_asset(Rectangle::from_length(CELL_SIZE * BLOCK_SQUARE_SIZE)),
            frame,
            grid,
            pieces_small_blocks,
            pieces,
        }
    }
}

#[derive(Resource)]
pub struct MaterialCollection {
    pub pieces: EnumMap<PieceKind, Handle<ColorMaterial>>,
    pub ghosts: EnumMap<PieceKind, Handle<ColorMaterial>>,
}

impl FromWorld for MaterialCollection {
    fn from_world(world: &mut World) -> Self {
        let palette: ColorPalette = world.resource::<ColorPalette>().clone();

        let base_color = |kind| match kind {
            PieceKind::I => Color::srgb(0.0, 1.0, 1.0), // cyan
            PieceKind::O => Color::srgb(1.0, 1.0, 0.0), // yellow
            PieceKind::T => Color::srgb(0.5, 0.0, 0.5), // purple
            PieceKind::S => Color::srgb(0.0, 1.0, 0.0), // green
            PieceKind::Z => Color::srgb(1.0, 0.0, 0.0), // red
            PieceKind::J => Color::srgb(1.0, 0.5, 0.0), // orange
            PieceKind::L => Color::srgb(0.0, 0.0, 1.0), // blue
        };

        let ghosts = EnumMap::from_fn(|kind| {
            world.add_asset(base_color(kind).mix(&palette.background_2.color, 0.9))
        });

        let pieces = EnumMap::from_fn(|kind| world.add_asset(base_color(kind)));
        Self { pieces, ghosts }
    }
}
