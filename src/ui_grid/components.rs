use bevy::prelude::*;

#[derive(Component)]
pub struct BackgroundTile;

#[derive(Component)]
pub struct PieceTile;

#[derive(Component)]
pub struct PieceGhost(pub Entity);

/// Animation that is despawned once it finised playing.
#[derive(Component)]
pub struct OneShotPlayer;

/// Marker that indicate when a sprite is aligned from the center of cells.
#[derive(Component)]
pub struct AlignedOnCellCenter;
