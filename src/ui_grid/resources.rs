use bevy::prelude::*;
use enum_map::EnumMap;

use crate::game_rules::components::PieceKind;

use super::CELL_SIZE;

#[derive(Resource)]
pub struct MeshCollection {
    pub pieces: EnumMap<PieceKind, Handle<Mesh>>,
    pub square: Handle<Mesh>,
}

impl FromWorld for MeshCollection {
    fn from_world(world: &mut World) -> Self {
        let pieces = EnumMap::from_fn(|piece_kind: PieceKind| {
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

            world.add_asset(mesh)
        });

        Self {
            square: world.add_asset(Rectangle::from_length(CELL_SIZE)),
            pieces,
        }
    }
}

#[derive(Resource)]
pub struct MaterialCollection {
    pub pieces: EnumMap<PieceKind, Handle<ColorMaterial>>,
    pub ghosts: EnumMap<PieceKind, Handle<ColorMaterial>>,
    pub background: Handle<ColorMaterial>,
}

impl FromWorld for MaterialCollection {
    fn from_world(world: &mut World) -> Self {
        let base_color = |kind| match kind {
            PieceKind::I => Color::srgb(0.0, 1.0, 1.0), // cyan
            PieceKind::O => Color::srgb(1.0, 1.0, 0.0), // yellow
            PieceKind::T => Color::srgb(0.5, 0.0, 0.5), // purple
            PieceKind::S => Color::srgb(0.0, 1.0, 0.0), // green
            PieceKind::Z => Color::srgb(1.0, 0.0, 0.0), // red
            PieceKind::J => Color::srgb(1.0, 0.5, 0.0), // orange
            PieceKind::L => Color::srgb(0.0, 0.0, 1.0), // blue
        };

        let background = world.add_asset(Color::srgb(0.2, 0.2, 0.2));
        let pieces = EnumMap::from_fn(|kind| world.add_asset(base_color(kind)));

        let ghosts = EnumMap::from_fn(|kind| {
            world.add_asset(base_color(kind).mix(&Color::WHITE, 0.8).with_alpha(0.5))
        });

        Self {
            pieces,
            ghosts,
            background,
        }
    }
}
