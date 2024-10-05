use std::ops::DerefMut;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{GRID_HEIGHT, GRID_WIDTH};

use super::components::{GridPos, PieceKind, Spin};

// -- CellState

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellState {
    Empty,
    Full(PieceKind),
}

impl Default for CellState {
    fn default() -> Self {
        Self::Empty
    }
}

// -- GridState

#[derive(Resource)]
pub struct GridState {
    pub cells: [[CellState; GRID_HEIGHT as _]; GRID_WIDTH as _],
}

impl GridState {
    pub fn cell_state(&self, cell: &GridPos) -> Option<&CellState> {
        let GridPos { x, y } = cell;
        self.cells.get(usize::from(*x))?.get(usize::from(*y))
    }

    fn conflicts(&self, kind: PieceKind, pos: GridPos, spin: Spin) -> bool {
        !kind
            .piece_covered_cells(pos, spin)
            .all(|cell| matches!(self.cell_state(&cell), Some(CellState::Empty)))
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

impl Default for GridState {
    fn default() -> Self {
        Self {
            cells: std::array::from_fn(|_| std::array::from_fn(|_| CellState::Empty)),
        }
    }
}

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
