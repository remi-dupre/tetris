use std::ops::Deref;

use bevy::animation::AnimationTargetId;
use bevy::prelude::*;
use enum_map::EnumMap;

use crate::game_rules::components::PieceKind;
use crate::{GRID_VISIBLE_HEIGHT, GRID_WIDTH};

// Shape of the area
pub(crate) const UI_GRID_VIRTUAL_HEIGHT: f32 = 800.0;
pub(crate) const UI_GRID_VIRTUAL_WIDTH: f32 = 400.0;

// Size of elements
pub(crate) const CELL_SIZE: f32 = (UI_GRID_VIRTUAL_WIDTH - BORDER_SIZE) / 10.0;
pub(crate) const BORDER_SIZE: f32 = 20.0;
pub(crate) const BLOCK_SQUARE_RATIO: f32 = 0.9;
pub(crate) const BLOCK_SQUARE_SMALL_RATIO: f32 = 0.75;

// Config

#[derive(Resource)]
pub(crate) struct UiGridConfig {
    pub(crate) pos: [f32; 2],
    pub(crate) size: [f32; 2],
}

// -- Root
#[derive(Resource)]
pub(crate) struct UiGridRoot(Entity);

impl Deref for UiGridRoot {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromWorld for UiGridRoot {
    fn from_world(world: &mut World) -> Self {
        let config: &UiGridConfig = world.resource();

        let root = world
            .spawn((
                Name::new("Grid Panel"),
                InheritedVisibility::default(),
                Transform::default()
                    .with_translation([config.pos[0], config.pos[1], 0.0].into())
                    .with_scale(
                        [
                            config.size[0] / UI_GRID_VIRTUAL_WIDTH,
                            config.size[1] / UI_GRID_VIRTUAL_HEIGHT,
                            0.0,
                        ]
                        .into(),
                    ),
                GlobalTransform::default(),
            ))
            .id();

        Self(root)
    }
}

// MeshCollection

#[derive(Resource)]
pub(crate) struct MeshCollection {
    pub(crate) square: Handle<Mesh>,
    pub(crate) frame: Handle<Mesh>,
    pub(crate) grid: Handle<Mesh>,
    pub(crate) grid_background: Handle<Mesh>,
    pub(crate) pieces_small_blocks: EnumMap<PieceKind, Handle<Mesh>>,
}

impl FromWorld for MeshCollection {
    fn from_world(world: &mut World) -> Self {
        let frame = {
            let vertical_bar = Rectangle::new(BORDER_SIZE / 2.0, UI_GRID_VIRTUAL_HEIGHT);

            let horizontal_bar = Rectangle::new(UI_GRID_VIRTUAL_WIDTH, BORDER_SIZE / 2.0);

            let mut mesh: Mesh = Mesh::from(vertical_bar)
                .translated_by([BORDER_SIZE / 4.0 - UI_GRID_VIRTUAL_WIDTH / 2.0, 0.0, 0.0].into());

            mesh.merge(
                &Mesh::from(vertical_bar).translated_by(
                    [UI_GRID_VIRTUAL_WIDTH / 2.0 - BORDER_SIZE / 4.0, 0.0, 0.0].into(),
                ),
            );

            mesh.merge(&Mesh::from(horizontal_bar).translated_by(
                [0.0, BORDER_SIZE / 4.0 - UI_GRID_VIRTUAL_HEIGHT / 2.0, 0.0].into(),
            ));

            mesh.merge(&Mesh::from(horizontal_bar).translated_by(
                [0.0, UI_GRID_VIRTUAL_HEIGHT / 2.0 - BORDER_SIZE / 4.0, 0.0].into(),
            ));

            world.add_asset(mesh)
        };

        fn mesh_piece(
            coords: impl Iterator<Item = [i8; 2]>,
            square_size: f32,
            align_on_cell_center: bool,
        ) -> Mesh {
            coords
                .map(|[x, y]| {
                    Mesh::from(Rectangle::from_length(CELL_SIZE * square_size)).translated_by(
                        [CELL_SIZE * f32::from(x), CELL_SIZE * f32::from(y), 0.0].into(),
                    )
                })
                .reduce(|mut x, y| {
                    x.merge(&y);
                    x
                })
                .unwrap()
                .translated_by({
                    if align_on_cell_center {
                        [0.5 * CELL_SIZE, 0.5 * CELL_SIZE, 0.0].into()
                    } else {
                        [0.0, 0.0, 0.0].into()
                    }
                })
        }

        let grid_background = world.add_asset(Rectangle::new(
            UI_GRID_VIRTUAL_WIDTH,
            UI_GRID_VIRTUAL_HEIGHT,
        ));

        let grid = world.add_asset(
            mesh_piece(
                (0..GRID_WIDTH)
                    .flat_map(|x| (0..GRID_VISIBLE_HEIGHT).map(move |y| [x as _, y as _])),
                0.1,
                false,
            )
            .translated_by(Vec3::new(
                (BORDER_SIZE - UI_GRID_VIRTUAL_WIDTH + CELL_SIZE) / 2.0,
                (BORDER_SIZE - UI_GRID_VIRTUAL_HEIGHT + CELL_SIZE) / 2.0,
                0.0,
            )),
        );

        let pieces_small_blocks = EnumMap::from_fn(|piece_kind: PieceKind| {
            world.add_asset(mesh_piece(
                piece_kind.base_shape().into_iter(),
                BLOCK_SQUARE_SMALL_RATIO,
                piece_kind.base_width() % 2 == 0,
            ))
        });

        Self {
            square: world.add_asset(Rectangle::from_length(CELL_SIZE * BLOCK_SQUARE_RATIO)),
            frame,
            grid,
            grid_background,
            pieces_small_blocks,
        }
    }
}

// AnimationCollection

pub(crate) struct AnimationMeta {
    /// The target of this animation
    pub(crate) animation_target_id: AnimationTargetId,
    /// The graph holding the animation
    pub(crate) graph: Handle<AnimationGraph>,
    /// The node that must be played
    pub(crate) node: AnimationNodeIndex,
}

impl AnimationMeta {
    fn animation_inflate(world: &mut World) -> Self {
        let ratio = BLOCK_SQUARE_SMALL_RATIO / BLOCK_SQUARE_RATIO;
        let animation_target_id = AnimationTargetId::from_name(&Name::new("block-inflate"));
        let mut animation = AnimationClip::default();

        animation.add_curve_to_target(
            animation_target_id,
            VariableCurve {
                keyframe_timestamps: vec![0.0, 0.05, 0.2],
                keyframes: Keyframes::Scale(vec![
                    Vec3::new(ratio, ratio, 1.0),
                    Vec3::new(1.0 / BLOCK_SQUARE_RATIO, 1.0 / BLOCK_SQUARE_RATIO, 1.0),
                    Vec3::new(1.0, 1.0, 1.0),
                ]),
                interpolation: Interpolation::Linear,
            },
        );

        let (graph, node) = AnimationGraph::from_clip(world.add_asset(animation));

        Self {
            animation_target_id,
            graph: world.add_asset(graph),
            node,
        }
    }

    fn animation_blink(world: &mut World) -> Self {
        let hide = Vec3::new(0.0, 0.0, 1.0);
        let show = Vec3::new(1.0, 1.0, 1.0);
        let animation_target_id = AnimationTargetId::from_name(&Name::new("blink"));
        let mut animation = AnimationClip::default();

        animation.add_curve_to_target(
            animation_target_id,
            VariableCurve {
                keyframe_timestamps: vec![0.0, 0.3, 0.6, 0.8],
                keyframes: Keyframes::Scale(vec![hide, show, hide, show]),
                interpolation: Interpolation::Step,
            },
        );

        let (graph, node) = AnimationGraph::from_clip(world.add_asset(animation));

        Self {
            animation_target_id,
            graph: world.add_asset(graph),
            node,
        }
    }
}

#[derive(Resource)]
pub(crate) struct AnimationCollection {
    pub(crate) inflate: AnimationMeta,
    // Blink lines before clearing them. Base animation is 1s long and must be scaled.
    pub(crate) blink: AnimationMeta,
}

impl FromWorld for AnimationCollection {
    fn from_world(world: &mut World) -> Self {
        Self {
            inflate: AnimationMeta::animation_inflate(world),
            blink: AnimationMeta::animation_blink(world),
        }
    }
}
