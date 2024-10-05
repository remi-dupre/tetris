use bevy::prelude::*;

#[derive(Component)]
pub struct BackgroundTile;

#[derive(Component)]
pub struct PieceTile;

#[derive(Component)]
pub struct PieceGhost(pub Entity);
