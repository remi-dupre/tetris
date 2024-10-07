use std::ops::DerefMut;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{GRID_HEIGHT, GRID_WIDTH};

use super::components::{FilledCell, GridPos, PieceKind, Spin};

// GridState

#[derive(Resource, Default)]
pub struct GridState {
    cells: [[Option<Entity>; GRID_HEIGHT as _]; GRID_WIDTH as _],
}

impl GridState {
    pub fn is_empty(&self, pos: &GridPos) -> bool {
        (0..GRID_WIDTH).contains(&pos.x)
            && (0..GRID_HEIGHT).contains(&pos.y)
            && !self.is_filled(pos)
    }

    pub fn is_filled(&self, pos: &GridPos) -> bool {
        self.get_filled_entity(pos).is_some()
    }

    pub fn get_filled_entity(&self, pos: &GridPos) -> Option<&Entity> {
        self.cells
            .get(usize::from(pos.x))?
            .get(usize::from(pos.y))
            .as_ref()?
            .as_ref()
    }

    pub fn spawn_cell(
        &mut self,
        commands: &mut Commands,
        pos: &GridPos,
        color_from_kind: PieceKind,
    ) {
        assert!(self.is_empty(pos));

        let entity = commands
            .spawn((
                Name::new(format!("Filled Cell at {pos}")),
                *pos,
                FilledCell { color_from_kind },
            ))
            .id();

        self.cells[usize::from(pos.x)][usize::from(pos.y)] = Some(entity);
    }

    pub fn despawn_cell(&mut self, commands: &mut Commands, pos: &GridPos) -> bool {
        let Some(&entity) = self.get_filled_entity(pos) else {
            return false;
        };

        commands.entity(entity).despawn();
        self.cells[usize::from(pos.x)][usize::from(pos.y)] = None;
        true
    }

    pub fn move_to(&mut self, commands: &mut Commands, from: &GridPos, to: &GridPos) -> bool {
        if from == to {
            return false;
        }

        if let Some(&removed_entity) = self.get_filled_entity(to) {
            commands.entity(removed_entity).despawn();
        }

        if let Some(&moved_entity) = self.get_filled_entity(from) {
            commands.entity(moved_entity).insert(*to);
        };

        self.cells[usize::from(to.x)][usize::from(to.y)] =
            self.cells[usize::from(from.x)][usize::from(from.y)].take();

        true
    }

    fn conflicts(&self, kind: PieceKind, pos: GridPos, spin: Spin) -> bool {
        !kind
            .piece_covered_cells(pos, spin)
            .all(|pos| self.is_empty(&pos))
    }

    pub fn try_move(
        &self,
        delta: [i8; 2],
        kind: PieceKind,
        mut pos: impl DerefMut<Target = GridPos>,
        spin: Spin,
    ) -> bool {
        let new_pos = GridPos {
            x: pos.x.wrapping_add_signed(delta[0]),
            y: pos.y.wrapping_add_signed(delta[1]),
        };

        if self.conflicts(kind, new_pos, spin) {
            return false;
        }

        *pos = new_pos;
        true
    }

    pub fn try_rotate(
        &self,
        delta: Spin,
        kind: PieceKind,
        mut pos: Mut<GridPos>,
        mut spin: Mut<Spin>,
    ) -> bool {
        let new_spin = Spin((spin.0 + delta.0) % 4);

        if self.conflicts(kind, *pos, new_spin)
            && !self.try_move([1, 0], kind, pos.reborrow(), new_spin)
            && !self.try_move([-1, 0], kind, pos, new_spin)
        {
            return false;
        }

        *spin = new_spin;
        true
    }
}

// -- Score

#[derive(Resource, Default)]
pub struct Score(pub u64);

// -- PieceGenerator

#[derive(Resource, Default)]
pub struct PieceGenerator {
    pending: Vec<PieceKind>,
}

impl PieceGenerator {
    fn ensure_pending_is_not_empty(&mut self) {
        if !self.pending.is_empty() {
            return;
        }

        let mut rng = rand::thread_rng();
        let mut pool = PieceKind::all();
        pool.shuffle(&mut rng);
        self.pending.extend_from_slice(&pool);
    }

    pub fn choose(&mut self) -> PieceKind {
        self.ensure_pending_is_not_empty();
        self.pending.pop().unwrap()
    }

    pub fn _peek(&mut self) -> PieceKind {
        self.ensure_pending_is_not_empty();
        self.pending.pop().unwrap()
    }
}
