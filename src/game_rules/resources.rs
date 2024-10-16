use std::fmt::Display;
use std::ops::DerefMut;
use std::time::Duration;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{GRID_HEIGHT, GRID_WIDTH};

use super::components::{FilledCell, GridPos, PieceKind, Spin};

/// Soft drop's default behavior is to speedup time by a constant factor
pub(crate) const SOFT_DROP_SPEEDUP: u32 = 3;

/// Minimum time per tile when using soft drop
pub(crate) const SOFT_DROP_MAX_DELAY: Duration = Duration::from_millis(50);

/// Maximum time lying on the ground before locking.
/// See https://tetris.fandom.com/wiki/Lock_delay
pub(crate) const LOCK_DELAY: Duration = Duration::from_millis(500);

/// Duration for which the game pauses when lines are cleared.
pub(crate) const CLEAR_DELAY: Duration = Duration::from_millis(400);

// -- PausedForRows

/// Allows to pause the game progress for a while, this must be removed from
/// another plugin to restart.
#[derive(Resource)]
pub(crate) struct PausedForClear {
    pub(crate) timer: Timer,
    pub(crate) rows_to_delete: Vec<u8>,
}

// -- Stopwatch

#[derive(Resource, Default)]
pub(crate) struct Stopwatch {
    pub(crate) since_begining: Duration,
}

impl Display for Stopwatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hours = self.since_begining.as_secs() / 3600;
        let minutes = (self.since_begining.as_secs() % 3600) / 60;
        let seconds = self.since_begining.as_secs() % 60;

        if hours > 0 {
            write!(f, "{hours:02}")?;
        }

        write!(f, "{minutes:02}:{seconds:02}")
    }
}

// -- GridState

#[derive(Resource, Default)]
pub(crate) struct GridState {
    cells: [[Option<Entity>; GRID_HEIGHT as _]; GRID_WIDTH as _],
}

impl GridState {
    pub(crate) fn is_empty(&self, pos: &GridPos) -> bool {
        (0..GRID_WIDTH).contains(&pos.x)
            && (0..GRID_HEIGHT).contains(&pos.y)
            && !self.is_filled(pos)
    }

    pub(crate) fn is_filled(&self, pos: &GridPos) -> bool {
        self.get_filled_entity(pos).is_some()
    }

    pub(crate) fn get_filled_entity(&self, pos: &GridPos) -> Option<&Entity> {
        self.cells
            .get(usize::from(pos.x))?
            .get(usize::from(pos.y))
            .as_ref()?
            .as_ref()
    }

    pub(crate) fn spawn_cell(
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

    pub(crate) fn despawn_cell(&mut self, commands: &mut Commands, pos: &GridPos) -> bool {
        let Some(&entity) = self.get_filled_entity(pos) else {
            return false;
        };

        commands.entity(entity).despawn();
        self.cells[usize::from(pos.x)][usize::from(pos.y)] = None;
        true
    }

    pub(crate) fn move_to(
        &mut self,
        commands: &mut Commands,
        from: &GridPos,
        to: &GridPos,
    ) -> bool {
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

    pub(crate) fn try_move(
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

    fn try_rotate(
        &self,
        delta: Spin,
        kind: PieceKind,
        mut pos: Mut<GridPos>,
        mut spin: Mut<Spin>,
        kick_directions: impl IntoIterator<Item = [i8; 2]>,
    ) -> bool {
        let new_spin = Spin((spin.0 + delta.0) % 4);

        if self.conflicts(kind, *pos, new_spin)
            && !kick_directions
                .into_iter()
                .any(|dir| self.try_move(dir, kind, pos.reborrow(), new_spin))
        {
            return false;
        }

        *spin = new_spin;
        true
    }

    pub(crate) fn try_rotate_right(
        &self,
        kind: PieceKind,
        pos: Mut<GridPos>,
        spin: Mut<Spin>,
    ) -> bool {
        let kick_directions = kind.wall_kick_incr_dirs()[usize::from(spin.0 % 4)];
        self.try_rotate(Spin(1), kind, pos, spin, kick_directions)
    }

    pub(crate) fn try_rotate_left(
        &self,
        kind: PieceKind,
        pos: Mut<GridPos>,
        spin: Mut<Spin>,
    ) -> bool {
        let kick_directions = kind.wall_kick_incr_dirs()[usize::from((spin.0 + 3) % 4)]
            .into_iter()
            .map(|[x, y]| [-x, -y]);

        self.try_rotate(Spin(3), kind, pos, spin, kick_directions)
    }
}

// -- XP

#[derive(Resource, Default)]
pub(crate) struct XP(pub(crate) u32);

impl XP {
    pub(crate) fn level(&self) -> u32 {
        1 + self.0 / 10
    }

    /// See https://tetris.fandom.com/wiki/Tetris_Worlds#Gravity
    pub(crate) fn time_per_row(&self) -> Duration {
        Duration::from_secs_f64(
            (0.8 - (f64::from(self.level() - 1) * 0.007))
                .powi(i32::try_from(self.level() - 1).expect("Level Overflow")),
        )
    }
}

impl Display for XP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.level())
    }
}

// -- Score
#[derive(Resource, Default)]
pub(crate) struct Score(pub(crate) u64);

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let log_1000 = self.0.checked_ilog10().unwrap_or(0) / 3;
        write!(f, "{}", self.0 / 1000u64.pow(log_1000))?;

        for exp in (0..log_1000).rev() {
            write!(
                f,
                ",{:03}",
                (self.0 % 1000u64.pow(exp + 1)) / 1000u64.pow(exp)
            )?;
        }

        Ok(())
    }
}

// -- PieceGenerator

#[derive(Resource, Default)]
pub(crate) struct PieceGenerator {
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

    pub(crate) fn choose(&mut self) -> PieceKind {
        self.ensure_pending_is_not_empty();
        self.pending.pop().unwrap()
    }

    pub(crate) fn peek(&mut self) -> PieceKind {
        self.ensure_pending_is_not_empty();
        *self.pending.last().unwrap()
    }
}
