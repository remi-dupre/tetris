use bevy::prelude::*;

use crate::game_rules::plugin::GameUpdateSystems;

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
                    udpate_score_display,
                    udpate_level_display,
                    update_next_piece,
                )
                    .chain()
                    .after(GameUpdateSystems),
            );
    }
}
