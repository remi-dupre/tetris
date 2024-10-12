use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PieceTile;

#[derive(Component)]
pub(crate) struct PieceGhost(pub(crate) Entity);

/// Animation that is despawned once it finised playing.
#[derive(Component)]
pub(crate) struct OneShotPlayer;

/// Marker that indicate when a sprite is aligned from the center of cells.
#[derive(Component)]
pub(crate) struct AlignedOnCellCenter;
