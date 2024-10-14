use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

// -- PlayerInputQueue

#[derive(Debug, Clone, Copy)]
pub(crate) enum PlayerInput {
    MoveLeft,
    MoveRight,
    HardDrop,
    RotateRight,
    RotateLeft,
    // SoftDrop,
}

#[derive(Resource, Default)]
pub(crate) struct PlayerInputQueue {
    pub(crate) queue: VecDeque<PlayerInput>,
}

impl Deref for PlayerInputQueue {
    type Target = VecDeque<PlayerInput>;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl DerefMut for PlayerInputQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.queue
    }
}

// -- TouchState

pub(crate) struct TouchState {
    pub(crate) start_position: Vec2,
    pub(crate) drawn_input: Option<PlayerInput>,
}

#[derive(Resource, Default)]
pub(crate) struct TouchStateRegistry {
    pub(crate) touch_start: HashMap<u64, TouchState>,
}
