use bevy::prelude::*;

use super::resources::*;

pub(crate) struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorPalette>();
    }
}
