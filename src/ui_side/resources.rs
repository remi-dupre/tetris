use std::ops::Deref;

use bevy::prelude::*;

use super::{UI_SIDE_VIRTUAL_HEIGHT, UI_SIDE_VIRTUAL_WIDTH};

// -- Config

#[derive(Resource)]
pub struct UiSideConfig {
    pub pos: [f32; 2],
    pub size: [f32; 2],
}

// -- Root

#[derive(Resource)]
pub struct UiSideRoot(Entity);

impl Deref for UiSideRoot {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromWorld for UiSideRoot {
    fn from_world(world: &mut World) -> Self {
        let config: &UiSideConfig = world.resource();

        let root = world
            .spawn((
                Name::new("Side Panel"),
                InheritedVisibility::default(),
                Transform::default()
                    .with_translation([config.pos[0], config.pos[1], 100.0].into())
                    .with_scale(
                        [
                            config.size[0] / UI_SIDE_VIRTUAL_WIDTH,
                            config.size[1] / UI_SIDE_VIRTUAL_HEIGHT,
                            1.0,
                        ]
                        .into(),
                    ),
                GlobalTransform::default(),
            ))
            .id();

        Self(root)
    }
}

// -- FontsCollection

#[derive(Resource)]
pub struct FontsCollection {
    pub default: Handle<Font>,
    pub title: Handle<Font>,
}

impl FromWorld for FontsCollection {
    fn from_world(world: &mut World) -> Self {
        Self {
            default: world.load_asset("fonts/pixeloid/sans.ttf"),
            title: world.load_asset("fonts/pixeloid/sans-bold.ttf"),
        }
    }
}

// -- MeshCollection

#[derive(Resource)]
pub struct MeshCollection {
    pub background: Handle<Mesh>,
}

impl FromWorld for MeshCollection {
    fn from_world(world: &mut World) -> Self {
        let background = world.add_asset(Rectangle::new(
            UI_SIDE_VIRTUAL_WIDTH,
            UI_SIDE_VIRTUAL_HEIGHT,
        ));

        Self { background }
    }
}
