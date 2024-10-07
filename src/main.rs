//! Guidelines : https://harddrop.com/wiki/Tetris_Guideline

pub mod common;
pub mod game_rules;
pub mod ui_grid;
pub mod ui_side;

use std::time::Duration;

use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::prelude::*;
use bevy::window::WindowResolution;

const WINDOW_TITLE: &str = "Tetris (Bevy Engine)";
const WINDOW_CLASS: &str = "org.remi-dupre.testing";
const WINDOW_SIZE: [f32; 2] = [600., 800.];

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
        .add_plugins((
            common::plugin::CommonPlugin,
            game_rules::plugin::GameRulesPlugin,
            ui_grid::plugin::UiGridPlugin {
                pos: [-100.0, 0.0], // x: -300..100 ; y: -400..400
                size: [400.0, 800.0],
            },
            ui_side::plugin::UiSidePlugin {
                pos: [200.0, 0.0], // x: 100..300 ; y: -400..400
                size: [200.0, 800.0],
            },
        ))
        .edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .run();
}
