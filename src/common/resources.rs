use bevy::prelude::*;

/// Pack a color with its corresponding material handle
#[derive(Clone)]
pub struct ResColor {
    pub color: Color,
    pub material: Handle<ColorMaterial>,
}

impl ResColor {
    fn register_hex(hex: &str, world: &mut World) -> Self {
        let color: Color = Srgba::hex(hex)
            .unwrap_or_else(|err| panic!("Invalid hex color `{hex}`: {err}"))
            .into();

        let material: Handle<ColorMaterial> = world.add_asset(color);
        Self { color, material }
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
pub struct ColorPalette {
    pub background_1: ResColor,
    pub background_2: ResColor,
    pub text_default: ResColor,
    pub text_title: ResColor,
}

impl FromWorld for ColorPalette {
    fn from_world(world: &mut World) -> Self {
        Self {
            background_1: ResColor::register_hex("#0a0a0b", world),
            background_2: ResColor::register_hex("#181e25", world),
            text_default: ResColor::register_hex("#fafcff", world),
            text_title: ResColor::register_hex("#5699f0", world),
        }
    }
}
