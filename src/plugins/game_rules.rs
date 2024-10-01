use std::time::Duration;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

pub const GRID_WIDTH: u8 = 10;
pub const GRID_HEIGHT: u8 = 22;
pub const GRID_VISIBLE_HEIGHT: u8 = 20;

const DROP_DELAY: Duration = Duration::from_millis(300);

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
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

    const fn base_shape(self) -> [[i8; 2]; 4] {
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

    const fn rotation(self, angle: u8) -> [[i8; 2]; 4] {
        let mut cells = self.base_shape();
        let mut steps = angle % 4;
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

    const fn base_width(self) -> u8 {
        match self {
            PieceKind::I => 4,
            PieceKind::O => 2,
            PieceKind::T | PieceKind::S | PieceKind::Z | PieceKind::J | PieceKind::L => 3,
        }
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
pub struct Cell(pub u8, pub u8);

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

#[derive(Resource)]
pub struct GridState {
    pub cells: [[CellState; GRID_HEIGHT as _]; GRID_WIDTH as _],
}

impl GridState {
    fn cell_state(&self, cell: &Cell) -> Option<&CellState> {
        let Cell(x, y) = cell;
        self.cells.get(usize::from(*x))?.get(usize::from(*y))
    }
}

impl Default for GridState {
    fn default() -> Self {
        Self {
            cells: std::array::from_fn(|_| std::array::from_fn(|_| CellState::Empty)),
        }
    }
}

#[derive(Resource, Clone)]
pub struct FallingPiece {
    pub kind: PieceKind,
    pos: [u8; 2],
    angle: u8,
    time_until_next_fall: Timer,
}

impl FallingPiece {
    fn new(kind: PieceKind) -> Self {
        let pos_x = if kind.base_width() % 2 == 0 { 5 } else { 4 };

        // Position just in view of the grid
        let pos_y = GRID_VISIBLE_HEIGHT.wrapping_add_signed(
            -kind
                .base_shape()
                .into_iter()
                .map(|[_, y]| y)
                .min()
                .unwrap_or(0),
        ) - 1;

        Self {
            pos: [pos_x, pos_y],
            kind,
            angle: 0,
            time_until_next_fall: Timer::new(DROP_DELAY, TimerMode::Repeating),
        }
    }

    fn conflicts(&self, grid: &GridState) -> bool {
        !self
            .iter_cells()
            .all(|cell| matches!(grid.cell_state(&cell), Some(CellState::Empty)))
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        self.kind.rotation(self.angle).into_iter().map(|[x, y]| {
            Cell(
                self.pos[0].wrapping_add_signed(x),
                self.pos[1].wrapping_add_signed(y),
            )
        })
    }

    pub fn try_move(&mut self, delta: [i8; 2], grid: &GridState) -> bool {
        self.pos = std::array::from_fn(|i| self.pos[i].wrapping_add_signed(delta[i]));

        if self.conflicts(grid) {
            self.pos = std::array::from_fn(|i| self.pos[i].wrapping_add_signed(-delta[i]));
            return false;
        }

        true
    }

    fn rotate(&mut self, angle: u8, grid: &GridState) -> bool {
        self.angle = (self.angle + angle) % 4;

        if self.conflicts(grid) && !self.try_move([1, 0], grid) && !self.try_move([-1, 0], grid) {
            self.angle = (self.angle + 4 - angle) % 4;
            return false;
        }

        true
    }
}

fn piece_spawn(mut commands: Commands, piece: Option<ResMut<FallingPiece>>) {
    use rand::seq::SliceRandom;

    if piece.is_none() {
        let mut rng = rand::thread_rng();
        let mut pool = PieceKind::all();
        pool.shuffle(&mut rng);
        commands.insert_resource(FallingPiece::new(pool.into_iter().next().unwrap()));
    }
}

fn piece_fall(
    mut grid: ResMut<GridState>,
    mut commands: Commands,
    piece: Option<ResMut<FallingPiece>>,
    time: Res<Time>,
) {
    let Some(mut piece) = piece else { return };
    piece.time_until_next_fall.tick(time.delta());

    for _ in 0..piece.time_until_next_fall.times_finished_this_tick() {
        if !piece.try_move([0, -1], &grid) {
            for Cell(x, y) in piece.iter_cells() {
                grid.cells[usize::from(x)][usize::from(y)] = CellState::Full(piece.kind);
            }

            commands.remove_resource::<FallingPiece>();
            break;
        }
    }
}

fn piece_move(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut commands: Commands,
    mut grid: ResMut<GridState>,
    piece: Option<ResMut<FallingPiece>>,
) {
    let Some(mut piece) = piece else { return };

    for event in keyboard_input_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.key_code {
            KeyCode::ArrowLeft => {
                piece.try_move([-1, 0], &grid);
            }
            KeyCode::ArrowRight => {
                piece.try_move([1, 0], &grid);
            }
            KeyCode::ArrowUp | KeyCode::KeyX => {
                piece.rotate(3, &grid);
            }
            KeyCode::ControlLeft | KeyCode::ControlRight | KeyCode::KeyZ => {
                piece.rotate(1, &grid);
            }
            KeyCode::Space => {
                if piece.try_move([0, -1], &grid) {
                    piece.time_until_next_fall.reset();
                }

                while piece.try_move([0, -1], &grid) {}
            }
            KeyCode::ArrowDown => {
                if piece.try_move([0, -1], &grid) {
                    piece.time_until_next_fall.reset();
                } else {
                    for Cell(x, y) in piece.iter_cells() {
                        grid.cells[usize::from(x)][usize::from(y)] = CellState::Full(piece.kind);
                    }

                    commands.remove_resource::<FallingPiece>();
                }
            }
            _ => {}
        }
    }
}

fn register_completed_lines(mut grid: ResMut<GridState>) {
    if !grid.is_changed() {
        return;
    }

    let mut target_line = 0;

    for y in 0..usize::from(GRID_VISIBLE_HEIGHT) {
        if (0..usize::from(GRID_WIDTH)).all(|x| matches!(grid.cells[x][y], CellState::Full(_))) {
            continue;
        }

        for x in 0..usize::from(GRID_WIDTH) {
            grid.cells[x][target_line] = std::mem::take(&mut grid.cells[x][y]);
        }

        target_line += 1;
    }

    for y in target_line..usize::from(GRID_VISIBLE_HEIGHT) {
        for x in 0..usize::from(GRID_WIDTH) {
            grid.cells[x][y] = CellState::Empty;
        }
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(GridState::default());
}

pub struct GameRules;

impl Plugin for GameRules {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (bevy::input::keyboard::keyboard_input_system).chain(),
            )
            .add_systems(
                Update,
                (
                    piece_spawn,
                    piece_move,
                    piece_fall,
                    register_completed_lines,
                )
                    .chain(),
            );
    }
}
