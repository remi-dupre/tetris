use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct ClearedLines {
    pub(crate) lines_count: u8,
}
