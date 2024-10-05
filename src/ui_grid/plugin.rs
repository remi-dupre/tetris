use bevy::prelude::*;

use crate::game_rules::plugin::UpdateGame;

use super::resources::*;
use super::systems::*;

pub struct GameWindow;

impl Plugin for GameWindow {
    fn build(&self, app: &mut App) {
        let update_ghost_state = (
            (attach_piece_ghost, remove_hanging_piece_ghost),
            (update_ghost_pos, update_ghost_spin),
        )
            .chain();

        app.init_resource::<MaterialCollection>()
            .init_resource::<MeshCollection>()
            .add_systems(Startup, (setup_camera, setup_background).chain())
            .add_systems(
                Update,
                (
                    button_pressed,
                    update_background,
                    (
                        (attach_piece_sprite, update_ghost_state),
                        (apply_sprite_pos, apply_sprite_angle),
                    )
                        .chain(),
                )
                    .after(UpdateGame),
            );
    }
}
