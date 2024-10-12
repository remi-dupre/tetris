use bevy::prelude::*;

#[derive(Event, Debug)]
pub(crate) struct ClearedLines {
    pub(crate) lines_count: u8,
}
