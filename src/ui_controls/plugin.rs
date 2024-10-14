use bevy::prelude::*;

use super::resources::*;
use super::systems::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct UiControlsSystems;

pub(crate) struct UiControlsPlugin;

impl Plugin for UiControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputQueue>()
            .init_resource::<TouchStateRegistry>()
            .add_systems(
                Update,
                (
                    bevy::input::keyboard::keyboard_input_system,
                    bevy::input::touch::touch_screen_input_system,
                    collect_keyboard_presses,
                    debug_touchscreen,
                    touch_start,
                    collect_touch_moves,
                    touch_end,
                )
                    .chain()
                    .in_set(UiControlsSystems),
            );
    }
}
