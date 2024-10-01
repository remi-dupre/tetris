//! Guidelines : https://harddrop.com/wiki/Tetris_Guideline

mod plugins;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            plugins::game_rules::GameRules,
            plugins::game_window::GameWindow,
        ))
        .run();
}
