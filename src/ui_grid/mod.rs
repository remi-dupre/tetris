pub mod components;
pub mod plugin;
pub mod resources;
pub mod systems;

use bevy::prelude::*;

pub const WINDOW_TITLE: &str = "Tetris (Bevy Engine)";
pub const WINDOW_CLASS: &str = "org.remi-dupre.testing";
pub const WINDOW_SIZE: [f32; 2] = [400., 600.];
pub const CELL_SIZE: f32 = 24.;

pub const GRID_POSITION: Vec3 = Vec3::new(
    -WINDOW_SIZE[0] / 2.0 + 10.0,
    -WINDOW_SIZE[1] / 2.0 + 10.0,
    0.,
);

pub fn tile_translation(x: u8, y: u8, z: f32) -> Vec3 {
    GRID_POSITION
        + Vec3::new(
            CELL_SIZE / 2.0 + (1.1 * CELL_SIZE) * f32::from(x),
            CELL_SIZE / 2.0 + (1.1 * CELL_SIZE) * f32::from(y),
            z,
        )
}
