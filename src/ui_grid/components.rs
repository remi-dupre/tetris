use bevy::prelude::*;

#[derive(Component)]
pub struct BackgroundTile;

#[derive(Component)]
pub struct PieceTile;

#[derive(Component)]
pub struct PieceGhost(pub Entity);

/// Marker that indicate when a sprite is aligned from the center of cells.
#[derive(Component)]
pub struct AlignedOnCellCenter;
