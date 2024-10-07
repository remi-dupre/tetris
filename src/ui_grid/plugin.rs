use bevy::prelude::*;

use crate::game_rules::plugin::GameUpdateSystems;

use super::resources::*;
use super::systems::*;

pub struct UiGridPlugin {
    pub pos: [f32; 2],
    pub size: [f32; 2],
}

impl Plugin for UiGridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiGridConfig {
            pos: self.pos,
            size: self.size,
        });

        let update_ghost_state = (
            (attach_piece_ghost, remove_hanging_piece_ghost),
            (update_ghost_pos, update_ghost_spin),
        )
            .chain();

        app.init_resource::<MaterialCollection>()
            .init_resource::<MeshCollection>()
            .init_resource::<UiGridRoot>()
            .add_systems(Startup, (setup_camera, draw_background, draw_frame).chain())
            .add_systems(
                Update,
                (
                    button_pressed,
                    (
                        (
                            attach_piece_sprite,
                            attach_filled_cell_sprite,
                            update_ghost_state,
                        ),
                        (apply_sprite_pos, apply_sprite_angle),
                    )
                        .chain()
                        .after(GameUpdateSystems),
                ),
            );
    }
}
