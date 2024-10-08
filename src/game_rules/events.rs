use bevy::prelude::*;

#[derive(Event)]
pub struct ClearedLines {
    pub(crate) lines_count: u8,
}
