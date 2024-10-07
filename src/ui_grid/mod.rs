pub mod components;
pub mod plugin;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

use self::resources::*;

pub fn tile_translation(x: u8, y: u8, z: f32) -> Vec3 {
    Vec3::new(
        CELL_SIZE * (f32::from(x) + 0.5) + BORDER_SIZE / 2.0 - UI_GRID_VIRTUAL_WIDTH / 2.0,
        CELL_SIZE * (f32::from(y) + 0.5) + BORDER_SIZE / 2.0 - UI_GRID_VIRTUAL_HEIGHT / 2.0,
        z,
    )
}
