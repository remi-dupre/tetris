//! Guidelines : https://harddrop.com/wiki/Tetris_Guideline

pub(crate) mod common;
pub(crate) mod game_rules;
pub(crate) mod ui_controls;
pub(crate) mod ui_grid;
pub(crate) mod ui_side;

#[cfg(test)]
pub(crate) mod tests;

use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::prelude::*;
use bevy::window::WindowResolution;

const WINDOW_TITLE: &str = "Tetris (Bevy Engine)";
const WINDOW_CLASS: &str = "org.remi-dupre.testing";
const WINDOW_SIZE: [f32; 2] = [580., 800.];

const GRID_WIDTH: u8 = 10;
const GRID_HEIGHT: u8 = 22;
const GRID_VISIBLE_HEIGHT: u8 = 20;

fn main() {
    let canvas = {
        if cfg!(debug_assertions) {
            None
        } else {
            Some("#game-view".to_string())
        }
    };

    App::new()
        // .add_plugins((
        //     bevy::diagnostic::EntityCountDiagnosticsPlugin,
        //     bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        //     bevy::diagnostic::LogDiagnosticsPlugin::default(),
        // ))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                name: Some(WINDOW_CLASS.to_string()),
                resolution: WindowResolution::new(WINDOW_SIZE[0], WINDOW_SIZE[1]),
                fit_canvas_to_parent: true,
                canvas,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins((
            common::plugin::CommonPlugin,
            game_rules::plugin::GameRulesPlugin,
            ui_controls::plugin::UiControlsPlugin,
            ui_grid::plugin::UiGridPlugin {
                pos: [-95.0, 0.0], // x: -290..110 ; y: -400..400
                size: [400.0, 800.0],
            },
            ui_side::plugin::UiSidePlugin {
                pos: [195.0, 0.0], // x: 90..290 ; y: -400..400
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
