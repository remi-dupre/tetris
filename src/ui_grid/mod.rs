pub mod components;
pub mod plugin;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

pub const UI_GRID_VIRTUAL_WIDTH: f32 = 400.0;
pub const UI_GRID_VIRTUAL_HEIGHT: f32 = 800.0;
pub const BORDER_SIZE: f32 = 20.0;
pub const CELL_SIZE: f32 = (UI_GRID_VIRTUAL_WIDTH - BORDER_SIZE) / 10.0;

pub const GRID_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);

pub fn tile_translation(x: u8, y: u8, z: f32) -> Vec3 {
    Vec3::new(
        CELL_SIZE * (f32::from(x) + 0.5) + BORDER_SIZE / 2.0 - UI_GRID_VIRTUAL_WIDTH / 2.0,
        CELL_SIZE * (f32::from(y) + 0.5) + BORDER_SIZE / 2.0 - UI_GRID_VIRTUAL_HEIGHT / 2.0,
        z,
    )
}
