use bevy::input::keyboard::keyboard_input_system;
use bevy::prelude::*;

use super::resources::*;
use super::systems::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameUpdateSystems;

pub struct GameRulesPlugin;

impl Plugin for GameRulesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceGenerator>()
            .init_resource::<Score>()
            .init_resource::<GridState>()
            .add_systems(Update, bevy::input::keyboard::keyboard_input_system)
            .add_systems(
                Update,
                (
                    piece_spawn,
                    piece_move.after(keyboard_input_system),
                    piece_fall,
                    register_completed_lines,
                )
                    .chain()
                    .in_set(GameUpdateSystems),
            );
    }
}
