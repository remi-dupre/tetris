use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Component, Default)]
pub(crate) struct ResourceDisplay<R: Resource + std::fmt::Display> {
    _phantom: PhantomData<&'static R>,
}

#[derive(Component)]
pub(crate) struct NextPiece;
