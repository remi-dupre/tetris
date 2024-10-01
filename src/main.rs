//! Guidelines : https://harddrop.com/wiki/Tetris_Guideline

use std::time::Duration;

use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::WindowResolution;
use const_random::const_random;

const WINDOW_TITLE: &str = "Tetris (Bevy Engine)";
const WINDOW_CLASS: &str = "org.remi-dupre.testing";
const WINDOW_SIZE: [f32; 2] = [400., 600.];
const CELL_SIZE: f32 = 24.;

const GRID_WIDTH: u8 = 10;
const GRID_HEIGHT: u8 = 22;
const GRID_VISIBLE_HEIGHT: u8 = 20;
const DROP_DELAY: Duration = Duration::from_millis(300);

const BACKGROUND_COLOR: Handle<ColorMaterial> = Handle::weak_from_u128(const_random!(u128));
const COLOR_CYAN: Handle<ColorMaterial> = Handle::weak_from_u128(const_random!(u128));
const COLOR_YELLOW: Handle<ColorMaterial> = Handle::weak_from_u128(const_random!(u128));
const COLOR_PURPLE: Handle<ColorMaterial> = Handle::weak_from_u128(const_random!(u128));
const COLOR_GREEN: Handle<ColorMaterial> = Handle::weak_from_u128(const_random!(u128));
const COLOR_RED: Handle<ColorMaterial> = Handle::weak_from_u128(const_random!(u128));
const COLOR_BLUE: Handle<ColorMaterial> = Handle::weak_from_u128(const_random!(u128));
const COLOR_ORANGE: Handle<ColorMaterial> = Handle::weak_from_u128(const_random!(u128));

const GRID_POSITION: Vec3 = Vec3::new(
    -WINDOW_SIZE[0] / 2.0 + 10.0,
    -WINDOW_SIZE[1] / 2.0 + 10.0,
    0.,
);

#[derive(Component, Debug, PartialEq, Eq)]
struct Cell(u8, u8);

#[derive(Clone, Debug)]
enum CellState {
    Empty,
    Full(Handle<ColorMaterial>),
}

impl Default for CellState {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Resource)]
struct GridState {
    cells: [[CellState; GRID_HEIGHT as _]; GRID_WIDTH as _],
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
struct FallingPiece {
    pos: [u8; 2],
    shape: [[i8; 2]; 4],
    color: Handle<ColorMaterial>,
    time_until_next_fall: Timer,
}

impl FallingPiece {
    fn pool() -> [FallingPiece; 7] {
        [
            // I
            FallingPiece {
                pos: [3, (GRID_VISIBLE_HEIGHT - 2) as _],
                shape: [[-2, 0], [-1, 0], [0, 0], [1, 0]],
                color: COLOR_CYAN,
                time_until_next_fall: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
            // O
            FallingPiece {
                pos: [4, (GRID_VISIBLE_HEIGHT - 1) as _],
                shape: [[-1, -1], [0, -1], [-1, 0], [0, 0]],
                color: COLOR_YELLOW,
                time_until_next_fall: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
            // T
            FallingPiece {
                pos: [3, (GRID_VISIBLE_HEIGHT - 2) as _],
                shape: [[-1, 0], [0, 0], [1, 0], [0, 1]],
                color: COLOR_PURPLE,
                time_until_next_fall: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
            // S
            FallingPiece {
                pos: [3, (GRID_VISIBLE_HEIGHT - 1) as _],
                shape: [[-1, -1], [0, -1], [0, 0], [1, 0]],
                color: COLOR_GREEN,
                time_until_next_fall: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
            // Z
            FallingPiece {
                pos: [3, (GRID_VISIBLE_HEIGHT - 1) as _],
                shape: [[-1, 0], [0, -1], [0, 0], [1, -1]],
                color: COLOR_RED,
                time_until_next_fall: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
            // J
            FallingPiece {
                pos: [3, (GRID_VISIBLE_HEIGHT - 1) as _],
                shape: [[-1, 0], [0, 0], [1, 0], [-1, 1]],
                color: COLOR_BLUE,
                time_until_next_fall: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
            // L
            FallingPiece {
                pos: [4, (GRID_VISIBLE_HEIGHT - 1) as _],
                shape: [[-1, 0], [0, 0], [1, 0], [1, 1]],
                color: COLOR_ORANGE,
                time_until_next_fall: Timer::new(DROP_DELAY, TimerMode::Repeating),
            },
        ]
    }

    fn conflicts(&self, grid: &GridState) -> bool {
        !self
            .iter_cells()
            .all(|cell| matches!(grid.cell_state(&cell), Some(CellState::Empty)))
    }

    fn iter_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        self.shape.iter().map(|[x, y]| {
            Cell(
                self.pos[0].wrapping_add_signed(*x),
                self.pos[1].wrapping_add_signed(*y),
            )
        })
    }

    fn covers_cell(&self, cell: &Cell) -> bool {
        self.iter_cells().any(|self_cell| &self_cell == cell)
    }

    fn move_x(&mut self, delta: i8, grid: &GridState) -> bool {
        if (delta < 0 && (-delta as u8) > self.min_x())
            || (delta > 0 && delta as u8 >= (GRID_WIDTH - self.max_x()))
        {
            return false;
        }

        self.pos[0] = self.pos[0].wrapping_add_signed(delta);

        if self.conflicts(grid) {
            self.pos[0] = self.pos[0].wrapping_add_signed(-delta);
            return false;
        }

        true
    }

    fn move_y(&mut self, delta: i8, grid: &GridState) -> bool {
        if (delta < 0 && (-delta as u8) > self.min_y())
            || (delta > 0 && delta as u8 >= (GRID_HEIGHT - self.max_y()))
        {
            return false;
        }

        self.pos[1] = self.pos[1].wrapping_add_signed(delta);

        if self.conflicts(grid) {
            self.pos[1] = self.pos[1].wrapping_add_signed(-delta);
            return false;
        }

        true
    }

    fn rotate(&mut self, angle: u8, grid: &GridState) -> bool {
        let bbox_size_is_even = (1 - self.bbox_size() % 2) as i8;
        let old_shape = self.shape;

        for _ in 0..angle {
            self.shape = self.shape.map(|[x, y]| [-y - bbox_size_is_even, x]);
        }

        if self.conflicts(grid) {
            self.shape = old_shape;
            return false;
        }

        true
    }

    fn min_x(&self) -> u8 {
        self.iter_cells()
            .map(|Cell(x, _)| x)
            .min()
            .unwrap_or(GRID_WIDTH - 1)
    }

    fn max_x(&self) -> u8 {
        self.iter_cells().map(|Cell(x, _)| x).max().unwrap_or(0)
    }

    fn min_y(&self) -> u8 {
        self.iter_cells()
            .map(|Cell(_, y)| y)
            .min()
            .unwrap_or(GRID_HEIGHT - 1)
    }

    fn max_y(&self) -> u8 {
        self.iter_cells().map(|Cell(_, y)| y).max().unwrap_or(0)
    }

    fn bbox_size(&self) -> u8 {
        1 + std::cmp::max(self.max_y() - self.min_y(), self.max_x() - self.min_x())
    }
}

fn piece_spawn(mut commands: Commands, piece: Option<ResMut<FallingPiece>>) {
    use rand::seq::SliceRandom;

    if piece.is_none() {
        let mut rng = rand::thread_rng();
        let mut pool = FallingPiece::pool();
        pool.shuffle(&mut rng);
        commands.insert_resource(pool.into_iter().next().unwrap());
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
        if !piece.move_y(-1, &grid) {
            for Cell(x, y) in piece.iter_cells() {
                grid.cells[usize::from(x)][usize::from(y)] = CellState::Full(piece.color.clone());
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

        match &event.logical_key {
            Key::ArrowLeft => {
                piece.move_x(-1, &grid);
            }
            Key::ArrowRight => {
                piece.move_x(1, &grid);
            }
            Key::ArrowUp => {
                piece.rotate(1, &grid);
            }
            Key::ArrowDown => {
                if !piece.move_y(-1, &grid) {
                    for Cell(x, y) in piece.iter_cells() {
                        grid.cells[usize::from(x)][usize::from(y)] =
                            CellState::Full(piece.color.clone());
                    }

                    commands.remove_resource::<FallingPiece>();
                    break;
                }

                piece.time_until_next_fall.reset();
            }
            _ => {}
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(GridState::default());

    for (handle, color) in [
        (&BACKGROUND_COLOR, bevy::color::palettes::css::LIGHT_GRAY),
        (&COLOR_CYAN, bevy::color::palettes::css::DARK_CYAN),
        (&COLOR_YELLOW, bevy::color::palettes::css::YELLOW),
        (&COLOR_PURPLE, bevy::color::palettes::css::PURPLE),
        (&COLOR_GREEN, bevy::color::palettes::css::GREEN),
        (&COLOR_RED, bevy::color::palettes::css::RED),
        (&COLOR_BLUE, bevy::color::palettes::css::BLUE),
        (&COLOR_ORANGE, bevy::color::palettes::css::ORANGE),
    ] {
        materials.insert(handle, Color::from(color).into());
    }

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_VISIBLE_HEIGHT {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(Rectangle::from_length(CELL_SIZE)).into(),
                    transform: Transform::default().with_translation(
                        GRID_POSITION
                            + Vec3::new(
                                CELL_SIZE / 2.0 + (1.1 * CELL_SIZE) * f32::from(x),
                                CELL_SIZE / 2.0 + (1.1 * CELL_SIZE) * f32::from(y),
                                0.,
                            ),
                    ),
                    material: BACKGROUND_COLOR,
                    ..Default::default()
                },
                Cell(x, y),
            ));
        }
    }
}

fn update_cells(
    mut cells: Query<(Mut<Handle<ColorMaterial>>, &Cell)>,
    piece: Option<Res<FallingPiece>>,
    grid: Res<GridState>,
) {
    for (mut material, cell) in cells.iter_mut() {
        let Cell(x, y) = cell;

        if let CellState::Full(color) = &grid.cells[usize::from(*x)][usize::from(*y)] {
            if material.as_ref() != color {
                *material = color.clone();
            }

            continue;
        }

        if let Some(piece) = &piece {
            if piece.covers_cell(cell) {
                *material = piece.color.clone();
                continue;
            }
        }

        *material = BACKGROUND_COLOR;
    }
}

fn button_pressed(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut exit: EventWriter<AppExit>,
) {
    for event in keyboard_input_events.read() {
        match &event.logical_key {
            Key::Character(c) if ["q", "Q"].contains(&c.as_str()) => {
                exit.send(AppExit::Success);
            }
            _ => {}
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                name: Some(WINDOW_CLASS.to_string()),
                resizable: false,
                resolution: WindowResolution::new(WINDOW_SIZE[0], WINDOW_SIZE[1]),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextStyle {
                    font_size: 10.0,
                    ..Default::default()
                },
            },
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (bevy::input::keyboard::keyboard_input_system, button_pressed).chain(),
        )
        .add_systems(Update, (update_cells, piece_spawn, piece_fall, piece_move))
        .run();
}
