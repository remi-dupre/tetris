use bevy::prelude::*;

use super::resources::*;
use super::systems::*;

pub struct GameRules;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpdateGame;

impl Plugin for GameRules {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceGenerator>()
            .init_resource::<GridState>()
            .add_systems(Update, bevy::input::keyboard::keyboard_input_system)
            .add_systems(
                Update,
                (
                    piece_spawn,
                    piece_move,
                    piece_fall,
                    register_completed_lines,
                )
                    .in_set(UpdateGame)
                    .chain(),
            );
    }
}
