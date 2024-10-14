use bevy::prelude::*;
use enum_map::EnumMap;

use crate::game_rules::components::PieceKind;

/// Pack a color with its corresponding material handle
#[derive(Clone)]
pub(crate) struct ResColor {
    pub(crate) color: Color,
    pub(crate) material: Handle<ColorMaterial>,
}

impl ResColor {
    fn register(color: Color, world: &mut World) -> Self {
        let material: Handle<ColorMaterial> = world.add_asset(color);
        Self { color, material }
    }

    fn register_hex(hex: &str, world: &mut World) -> Self {
        let color = Srgba::hex(hex)
            .unwrap_or_else(|err| panic!("Invalid hex color `{hex}`: {err}"))
            .into();

        Self::register(color, world)
    }
}

impl From<&ResColor> for Color {
    fn from(val: &ResColor) -> Self {
        val.color
    }
}

impl From<&ResColor> for Handle<ColorMaterial> {
    fn from(val: &ResColor) -> Self {
        val.material.clone()
    }
}

#[derive(Resource, Clone)]
pub(crate) struct ColorPalette {
    pub(crate) background_1: ResColor,
    pub(crate) background_2: ResColor,
    pub(crate) text_default: ResColor,
    pub(crate) text_title: ResColor,
    pub(crate) pieces: EnumMap<PieceKind, ResColor>,
    pub(crate) ghosts: EnumMap<PieceKind, ResColor>,
}

impl FromWorld for ColorPalette {
    fn from_world(world: &mut World) -> Self {
        let background_1 = ResColor::register_hex("#0a0a0b", world);
        let background_2 = ResColor::register_hex("#181e25", world);

        let pieces = EnumMap::from_fn(|kind| {
            match kind {
                PieceKind::I => ResColor::register_hex("#00ffff", world), // cyan
                PieceKind::O => ResColor::register_hex("#ffff00", world), // yellow
                PieceKind::T => ResColor::register_hex("#ff00ff", world), // purple
                PieceKind::S => ResColor::register_hex("#00ff00", world), // green
                PieceKind::Z => ResColor::register_hex("#ff0000", world), // red
                PieceKind::J => ResColor::register_hex("#0000ff", world), // blue
                PieceKind::L => ResColor::register_hex("#ff8000", world), // orange
            }
        });

        let ghosts = EnumMap::from_fn(|kind| {
            ResColor::register(pieces[kind].color.mix(&background_2.color, 0.9), world)
        });

        Self {
            background_1,
            background_2,
            text_default: ResColor::register_hex("#fafcff", world),
            text_title: ResColor::register_hex("#5699f0", world),
            pieces,
            ghosts,
        }
    }
}
