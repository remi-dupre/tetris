use bevy::prelude::*;

// -- Spin

#[derive(Component, Clone, Copy, Default)]
pub(crate) struct Spin(pub(crate) u8);

// -- PieceKind

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Debug, enum_map::Enum)]
pub(crate) enum PieceKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceKind {
    pub(crate) const fn all() -> [Self; 7] {
        [
            Self::I,
            Self::O,
            Self::T,
            Self::S,
            Self::Z,
            Self::J,
            Self::L,
        ]
    }

    pub(crate) const fn base_shape(self) -> [[i8; 2]; 4] {
        match self {
            PieceKind::I => [[-2, 0], [-1, 0], [0, 0], [1, 0]],
            PieceKind::O => [[-1, -1], [0, -1], [-1, 0], [0, 0]],
            PieceKind::T => [[-1, 0], [0, 0], [1, 0], [0, 1]],
            PieceKind::S => [[-1, 0], [0, 0], [0, 1], [1, 1]],
            PieceKind::Z => [[-1, 1], [0, 0], [0, 1], [1, 0]],
            PieceKind::J => [[-1, 0], [0, 0], [1, 0], [-1, 1]],
            PieceKind::L => [[-1, 0], [0, 0], [1, 0], [1, 1]],
        }
    }

    /// Wall kick directions when increasing angle.
    /// See https://tetris.fandom.com/wiki/SRS
    pub(crate) const fn wall_kick_incr_dirs(self) -> [[[i8; 2]; 4]; 4] {
        match self {
            Self::I => [
                [[-2, 0], [1, 0], [-2, -1], [1, 2]],
                [[-1, 0], [2, 0], [-1, 2], [2, -1]],
                [[2, 0], [-1, 0], [2, 1], [-1, 2]],
                [[1, 0], [-2, 0], [1, -2], [-2, 1]],
            ],
            _ => [
                [[-1, 0], [-1, 1], [0, -2], [-1, -2]],
                [[1, 0], [1, -1], [0, 2], [1, 2]],
                [[1, 0], [1, 1], [0, -2], [1, -2]],
                [[-1, 0], [-1, -1], [0, 2], [-1, 2]],
            ],
        }
    }

    const fn rotation(self, spin: Spin) -> [[i8; 2]; 4] {
        let mut cells = self.base_shape();
        let mut steps = spin.0 % 4;
        let bbox_is_even = (1 - self.base_width() % 2) as i8;

        while steps > 0 {
            let mut i = 0;

            while i < 4 {
                cells[i] = [cells[i][1], -cells[i][0] - bbox_is_even];
                i += 1;
            }

            steps -= 1;
        }

        cells
    }

    pub(crate) const fn base_width(self) -> u8 {
        match self {
            PieceKind::I => 4,
            PieceKind::O => 2,
            PieceKind::T | PieceKind::S | PieceKind::Z | PieceKind::J | PieceKind::L => 3,
        }
    }

    pub(crate) fn piece_covered_cells(self, pos: GridPos, spin: Spin) -> impl Iterator<Item = GridPos> {
        self.rotation(spin).into_iter().map(move |[x, y]| GridPos {
            x: pos.x.wrapping_add_signed(x),
            y: pos.y.wrapping_add_signed(y),
        })
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GridPos {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

impl std::fmt::Display for GridPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Component)]
pub(crate) struct FilledCell {
    pub(crate) color_from_kind: PieceKind,
}

#[derive(Component, Clone)]
pub(crate) struct Fall {
    pub(crate) down_timer: Timer,
    pub(crate) lock_timer: Timer,
}

#[derive(Bundle, Clone)]
pub(crate) struct FallingPieceBundle {
    pub(crate) kind: PieceKind,
    pub(crate) pos: GridPos,
    pub(crate) spin: Spin,
    pub(crate) fall: Fall,
}
