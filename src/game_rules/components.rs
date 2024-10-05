use bevy::prelude::*;

// -- Spin

#[derive(Component, Clone, Copy, Default)]
pub struct Spin(pub u8);

// -- PieceKind

#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Debug, enum_map::Enum)]
pub enum PieceKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceKind {
    pub const fn all() -> [Self; 7] {
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

    pub const fn base_shape(self) -> [[i8; 2]; 4] {
        match self {
            PieceKind::I => [[-2, 0], [-1, 0], [0, 0], [1, 0]],
            PieceKind::O => [[-1, -1], [0, -1], [-1, 0], [0, 0]],
            PieceKind::T => [[-1, 0], [0, 0], [1, 0], [0, 1]],
            PieceKind::S => [[-1, -1], [0, -1], [0, 0], [1, 0]],
            PieceKind::Z => [[-1, 0], [0, -1], [0, 0], [1, -1]],
            PieceKind::J => [[-1, 0], [0, 0], [1, 0], [-1, 1]],
            PieceKind::L => [[-1, 0], [0, 0], [1, 0], [1, 1]],
        }
    }

    const fn rotation(self, spin: Spin) -> [[i8; 2]; 4] {
        let mut cells = self.base_shape();
        let mut steps = spin.0 % 4;
        let bbox_is_even = (1 - self.base_width() % 2) as i8;

        while steps > 0 {
            let mut i = 0;

            while i < 4 {
                cells[i] = [-cells[i][1] - bbox_is_even, cells[i][0]];
                i += 1;
            }

            steps -= 1;
        }

        cells
    }

    pub const fn base_width(self) -> u8 {
        match self {
            PieceKind::I => 4,
            PieceKind::O => 2,
            PieceKind::T | PieceKind::S | PieceKind::Z | PieceKind::J | PieceKind::L => 3,
        }
    }

    pub fn piece_covered_cells(self, pos: GridPos, spin: Spin) -> impl Iterator<Item = GridPos> {
        self.rotation(spin).into_iter().map(move |[x, y]| GridPos {
            x: pos.x.wrapping_add_signed(x),
            y: pos.y.wrapping_add_signed(y),
        })
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct GridPos {
    pub x: u8,
    pub y: u8,
}

#[derive(Component, Clone, Default)]
pub struct Fall {
    pub next_trigger: Timer,
}

#[derive(Bundle, Clone)]
pub struct FallingPieceBundle {
    pub kind: PieceKind,
    pub pos: GridPos,
    pub spin: Spin,
    pub fall: Fall,
}
