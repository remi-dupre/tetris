use bevy::input::keyboard::keyboard_input_system;
use bevy::prelude::*;

use super::events::*;
use super::resources::*;
use super::systems::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameUpdateSystems;

pub(crate) struct GameRulesPlugin;

impl Plugin for GameRulesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceGenerator>()
            .init_resource::<Score>()
            .init_resource::<GridState>()
            .init_resource::<XP>()
            .init_resource::<Events<ClearedLines>>()
            .add_systems(
                Update,
                (
                    (
                        bevy::input::keyboard::keyboard_input_system,
                        resume_after_clear.run_if(resource_exists::<PausedForClear>),
                    ),
                    (
                        piece_spawn,
                        piece_move.after(keyboard_input_system),
                        piece_lock,
                        piece_fall,
                        register_completed_lines,
                        update_score,
                        update_xp,
                    )
                        .chain()
                        .run_if(not(resource_exists::<PausedForClear>))
                        .in_set(GameUpdateSystems),
                )
                    .chain(),
            );
    }
}
