use bevy::prelude::*;

use crate::game_rules::plugin::GameUpdateSystems;
use crate::game_rules::resources::Score;
use crate::game_rules::resources::Stopwatch;
use crate::game_rules::resources::XP;

use super::resources::*;
use super::systems::*;

pub(crate) struct UiSidePlugin {
    pub(crate) pos: [f32; 2],
    pub(crate) size: [f32; 2],
}

impl Plugin for UiSidePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiSideConfig {
            pos: self.pos,
            size: self.size,
        });

        app.init_resource::<UiSideRoot>()
            .init_resource::<FontsCollection>()
            .init_resource::<MeshCollection>()
            .add_systems(
                Startup,
                (setup_background, setup_preview, setup_score_pannel),
            )
            .add_systems(
                Update,
                (
                    update_resource_display::<Score>,
                    update_resource_display::<XP>,
                    update_resource_display::<Stopwatch>,
                    update_next_piece,
                )
                    .chain()
                    .after(GameUpdateSystems),
            );
    }
}
