//! Guidelines : https://harddrop.com/wiki/Tetris_Guideline

pub mod game_rules;
pub mod ui_grid;

use std::time::Duration;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::ui_grid::{WINDOW_CLASS, WINDOW_SIZE, WINDOW_TITLE};

const GRID_WIDTH: u8 = 10;
const GRID_HEIGHT: u8 = 22;
const GRID_VISIBLE_HEIGHT: u8 = 20;

const DROP_DELAY: Duration = Duration::from_millis(300);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                name: Some(WINDOW_CLASS.to_string()),
                resizable: false,
                resolution: WindowResolution::new(WINDOW_SIZE[0], WINDOW_SIZE[1]),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins((game_rules::plugin::GameRules, ui_grid::plugin::GameWindow))
        .run();
}
