use std::ops::DerefMut;
use std::time::Duration;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use rand::seq::SliceRandom;

pub const GRID_WIDTH: u8 = 10;
pub const GRID_HEIGHT: u8 = 22;
pub const GRID_VISIBLE_HEIGHT: u8 = 20;

const DROP_DELAY: Duration = Duration::from_millis(300);

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
}

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
    fn cell_state(&self, cell: &GridPos) -> Option<&CellState> {
        let GridPos { x, y } = cell;
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

#[derive(Component, Clone, Copy, Default)]
pub struct Spin(pub u8);

#[derive(Component, Clone, Copy, Default)]
pub struct GridPos {
    pub x: u8,
    pub y: u8,
}

#[derive(Component, Clone, Default)]
pub struct Fall {
    pub next_trigger: Timer,
}

#[derive(Bundle, Clone)]
pub struct FallingPiece {
    pub kind: PieceKind,
    pub pos: GridPos,
    pub spin: Spin,
    pub fall: Fall,
}

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

    pub fn next(&mut self) -> PieceKind {
        self.ensure_pending_is_not_empty();
        self.pending.pop().unwrap()
    }

    pub fn _peek(&mut self) -> PieceKind {
        self.ensure_pending_is_not_empty();
        self.pending.pop().unwrap()
    }
}

pub fn piece_covered_cells(
    kind: PieceKind,
    pos: GridPos,
    spin: Spin,
) -> impl Iterator<Item = GridPos> {
    kind.rotation(spin).into_iter().map(move |[x, y]| GridPos {
        x: pos.x.wrapping_add_signed(x),
        y: pos.y.wrapping_add_signed(y),
    })
}

fn conflicts(grid: &GridState, kind: PieceKind, pos: GridPos, spin: Spin) -> bool {
    !piece_covered_cells(kind, pos, spin)
        .all(|cell| matches!(grid.cell_state(&cell), Some(CellState::Empty)))
}

pub fn try_move(
    grid: &GridState,
    delta: [i8; 2],
    kind: PieceKind,
    mut pos: impl DerefMut<Target = GridPos>,
    spin: Spin,
) -> bool {
    let new_pos = GridPos {
        x: pos.x.wrapping_add_signed(delta[0]),
        y: pos.y.wrapping_add_signed(delta[1]),
    };

    if conflicts(grid, kind, new_pos, spin) {
        return false;
    }

    *pos = new_pos;
    true
}

fn try_rotate(
    grid: &GridState,
    delta: Spin,
    kind: PieceKind,
    mut pos: Mut<GridPos>,
    mut spin: Mut<Spin>,
) -> bool {
    let new_spin = Spin((spin.0 + delta.0) % 4);

    if conflicts(grid, kind, *pos, new_spin)
        && !try_move(grid, [1, 0], kind, pos.reborrow(), new_spin)
        && !try_move(grid, [-1, 0], kind, pos, new_spin)
    {
        return false;
    }

    *spin = new_spin;
    true
}

fn piece_spawn(
    mut commands: Commands,
    mut piece_generator: ResMut<PieceGenerator>,
    pieces: Query<(), (With<PieceKind>, With<Fall>)>,
) {
    if !pieces.is_empty() {
        return;
    }

    let kind = piece_generator.next();
    let x = if kind.base_width() % 2 == 0 { 5 } else { 4 };

    let y = GRID_VISIBLE_HEIGHT.wrapping_add_signed(
        -kind
            .base_shape()
            .into_iter()
            .map(|[_, y]| y)
            .min()
            .unwrap_or(0),
    ) - 1;

    commands.spawn({
        FallingPiece {
            pos: GridPos { x, y },
            kind,
            spin: Spin(0),
            fall: Fall {
                next_trigger: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
        }
    });
}

fn piece_fall(
    mut grid: ResMut<GridState>,
    mut commands: Commands,
    mut pieces: Query<(Entity, &PieceKind, &mut GridPos, &Spin, &mut Fall)>,
    time: Res<Time>,
) {
    for (entity, &kind, mut pos, &spin, mut fall) in &mut pieces {
        fall.next_trigger.tick(time.delta());

        for _ in 0..fall.next_trigger.times_finished_this_tick() {
            if !try_move(&grid, [0, -1], kind, pos.reborrow(), spin) {
                for cell in piece_covered_cells(kind, *pos.reborrow(), spin) {
                    grid.cells[usize::from(cell.x)][usize::from(cell.y)] = CellState::Full(kind);
                }

                commands.entity(entity).despawn_recursive();
                break;
            }
        }
    }
}

fn piece_move(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut commands: Commands,
    mut grid: ResMut<GridState>,
    mut pieces: Query<(Entity, &PieceKind, &mut GridPos, &mut Spin, &mut Fall)>,
) {
    for (entity, &kind, mut pos, mut spin, mut fall) in &mut pieces {
        for event in keyboard_input_events.read() {
            if event.state != ButtonState::Pressed {
                continue;
            }

            match &event.key_code {
                KeyCode::ArrowLeft => {
                    try_move(&grid, [-1, 0], kind, pos.reborrow(), *spin.reborrow());
                }
                KeyCode::ArrowRight => {
                    try_move(&grid, [1, 0], kind, pos.reborrow(), *spin.reborrow());
                }
                KeyCode::ArrowUp | KeyCode::KeyX => {
                    try_rotate(&grid, Spin(3), kind, pos.reborrow(), spin.reborrow());
                }
                KeyCode::ControlLeft | KeyCode::ControlRight | KeyCode::KeyZ => {
                    try_rotate(&grid, Spin(1), kind, pos.reborrow(), spin.reborrow());
                }
                KeyCode::Space => {
                    while try_move(&grid, [0, -1], kind, pos.reborrow(), *spin.reborrow()) {
                        fall.next_trigger.reset();
                    }
                }
                KeyCode::ArrowDown => {
                    if try_move(&grid, [0, -1], kind, pos.reborrow(), *spin.reborrow()) {
                        fall.next_trigger.reset();
                    } else {
                        for cell in piece_covered_cells(kind, *pos.reborrow(), *spin.reborrow()) {
                            grid.cells[usize::from(cell.x)][usize::from(cell.y)] =
                                CellState::Full(kind);
                        }

                        commands.entity(entity).despawn_recursive();
                    }
                }
                _ => {}
            }
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpdateGame;

impl Plugin for GameRules {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceGenerator>()
            .add_systems(Startup, setup)
            .add_systems(Update, bevy::input::keyboard::keyboard_input_system)
            .add_systems(
                Update,
                (
                    piece_spawn,
                    piece_move,
                    piece_fall,
                    register_completed_lines,
                )
                    .in_set(UpdateGame)
                    .chain(),
            );
    }
}
