use std::{collections::btree_map::Range, print, process::id, vec};

// First we'll import the crates we need for our game;
// in this case that is just `ggez` and `oorandom` (and `getrandom`
// to seed the RNG.)
use oorandom::Rand32;

// Next we need to actually `use` the pieces of ggez that we are going
// to need frequently.
use ggez::{
    Context, GameResult, context::{ContextFields, HasMut}, event, graphics::{self, Mesh, MeshBuilder}, input::keyboard::KeyInput,
};
use winit::keyboard::{Key, NamedKey::{self, New}};

const GRID_SIZE: (u16, u16) = (60, 50);

const PIXEL_SIZE: (i16, i16) = (16, 16);

const MARGIN: f32 = 1.1;

const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * PIXEL_SIZE.0 as f32 * MARGIN,
    GRID_SIZE.1 as f32 * PIXEL_SIZE.1 as f32 * MARGIN,
);

const DESIRED_FPS: u32 = 2;

const STRARTING_AWAKE_NUMBER: u16 = 700;

#[derive(Clone, Copy, PartialEq, Debug)]
struct GridPosition {
    x: f32,
    y: f32,
}

impl GridPosition {
    /// We make a standard helper function so that we can create a new `GridPosition`
    /// more easily.
    pub fn new(x: f32, y: f32) -> Self {
        GridPosition { x, y}
    }
}

/// We implement the `From` trait, which in this case allows us to convert easily between
/// a `GridPosition` and a ggez `graphics::Rect` which fills that grid cell.
/// Now we can just call `.into()` on a `GridPosition` where we want a
/// `Rect` that represents that grid cell.
impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> Self {
        graphics::Rect::new_i32(
            pos.x as i32,
            pos.y as i32,
            PIXEL_SIZE.0 as i32,
            PIXEL_SIZE.1 as i32,
        )
    }
}

impl From<(u16, u16)> for GridPosition {
    fn from(pos: (u16, u16)) -> Self {
        GridPosition { x: pos.0 as f32 * PIXEL_SIZE.0 as f32 * MARGIN, y: pos.1 as f32 * PIXEL_SIZE.1 as f32 * MARGIN}
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Cell {
    position : GridPosition,
    awake : bool,
}

impl Cell {
    pub fn new(position : GridPosition, awake: bool) -> Self {
        Cell { position, awake }
    }

    pub fn awake(&mut self) -> (){
        self.awake = true;
    }

    pub fn update(&mut self, awake_neighbour:u8) -> bool{
        self.awake = awake_neighbour == 3 || self.awake && awake_neighbour == 2;
        self.awake
    }
}

//now we create the grid that contain all of the grid pos
#[derive (Debug)]
struct Grid{
    vec_vec_cell_grid: Vec<Vec<Cell>>,
}

impl Grid{
    //first the helper that is used to create the grid
    //TODO passer les vec en tableaux 
    pub fn new(col_x: u16, col_y: u16) -> Self{
        let mut grid:Vec<Vec<Cell>> = Vec::new();
        for x in 0..col_x{
            let mut current_row:Vec<Cell> = Vec::new();
            for y in 0..col_y{
                current_row.push(
                    Cell::new(
                        (x, y).into(), 
                        false)
                    );
            }
            grid.push(current_row);
        }
        Grid{ vec_vec_cell_grid: grid }
    }

    // get return a borrowed position in the grid
    fn get(&self, x: usize, y:usize) -> Option<&Cell>{
        self.vec_vec_cell_grid.get(x).and_then(|row| row.get(y))
    }

    fn get_mut(&mut self, x: usize, y:usize) -> Option<&mut Cell>{
        self.vec_vec_cell_grid.get_mut(x).and_then(|row| row.get_mut(y))
    }

    fn neighbour(&self, x: u16, y: u16) -> [Option<&Cell>; 8] {
        const OFFSETS: [(i16, i16); 8] = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
        ];

        std::array::from_fn(|idx| {
            let (dx, dy) = (OFFSETS[idx].0 + x as i16, OFFSETS[idx].1 + y as i16);
            self.get(dx as usize, dy as usize)
        })
    }

    pub fn awake_neighbour(&self, x: u16, y: u16) -> u8{
        let mut count = 0;
        let neighbour = self.neighbour(x, y);
        for cell in neighbour{
            match cell {
                Some(x) => if x.awake {count += 1},
                None => continue
            }
        }
        count
    }
}

pub fn grid_mesh_builder(ctx: &mut Context, to_render: Vec<Cell>, color: graphics::Color) -> Mesh{
    let mut builder = graphics::MeshBuilder::new();

    for cell in to_render{
        builder.rectangle(graphics::DrawMode::fill(), 
                        cell.position.into(), 
                        color);
    }

    let mesh_data = builder.build();
    graphics::Mesh::from_data(ctx, mesh_data)
}

struct GameState {
    game_paused: bool,
    /// Our RNG state
    rng: Rand32,
    grid: Grid,
    background_mesh: Mesh,
    vec_active_cells: Vec<Cell>,
}

impl GameState {
    /// Our new function will set up the initial state of our game.
    pub fn new(ctx: &mut Context) -> Self {
        // on initialise la partie 
        let mut seed: [u8; 8] = [0; 8];
        getrandom::fill(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));

        // initialisation de la grille de cellule
        let mut grid = Grid::new(GRID_SIZE.0, GRID_SIZE.1);

        // creation du mesh qui sert de background

        let background_mesh: Mesh = grid_mesh_builder(ctx, 
                                            grid.vec_vec_cell_grid.concat(), 
                                            graphics::Color::from_rgb(20, 20, 20));


        // population initiale aleatoire de cellule eveiller
        let mut vec_active_cells: Vec<Cell> = Vec::new();
        for i in 0..STRARTING_AWAKE_NUMBER{
            let current_cell:&mut Cell = grid.get_mut(
                rng.rand_range(0..(GRID_SIZE.0 as u32)) as usize,
                rng.rand_range(0..(GRID_SIZE.1 as u32)) as usize
            ).unwrap();

            current_cell.awake();
            vec_active_cells.push(*current_cell)
        }

        GameState {
            game_paused: false,
            rng,
            grid,
            background_mesh,
            vec_active_cells,
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(DESIRED_FPS) {
            if !self.game_paused {
                // we first loop throught the entire grid to update the cells
                let mut neighbour_counts: vec::Vec<Vec<u8>> = Vec::new();
                for x in 0..self.grid.vec_vec_cell_grid.len(){
                    let mut row_count = Vec::new();
                    for y in 0..self.grid.vec_vec_cell_grid[x].len(){
                        row_count.push(self.grid.awake_neighbour(x as u16, y as u16));
                    }
                    neighbour_counts.push(row_count);
                }

                for (x, row) in self.grid.vec_vec_cell_grid.iter_mut().enumerate(){
                    for (y, cell) in row.iter_mut().enumerate(){
                        cell.update(neighbour_counts[x][y]);
                    }
                }

                //then we clear the active cell vec
                self.vec_active_cells = self.grid.vec_vec_cell_grid
                    .iter()
                    .flatten()
                    .filter(|cell| cell.awake)
                    .copied()
                    .collect();
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        canvas.draw(&self.background_mesh, graphics::DrawParam::default());
        let active_mesh = grid_mesh_builder(ctx, self.vec_active_cells.clone(), graphics::Color::GREEN);
        canvas.draw(&active_mesh, graphics::DrawParam::default());

        canvas.finish(ctx)?;
        Ok(())
    }

    /// `key_down_event` gets fired when a key gets pressed.
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        //TODO implement click
        Ok(())
    }
}

fn main() -> GameResult {
    let (mut ctx, events_loop) = ggez::ContextBuilder::new("gameoflife", "moi")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Game of life"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    let state = GameState::new(&mut ctx);
    event::run(ctx, events_loop, state)
}

#[cfg(test)]
mod tests {
    use std::print;

use super::*;

    #[test]
    fn grid_has_correct_dimensions() {
        let grid = Grid::new(10, 5);
        assert_eq!(grid.vec_vec_cell_grid.len(), 10);
        assert_eq!(grid.vec_vec_cell_grid[0].len(), 5);
    }

    #[test]
    fn awake_neighbour_counts_correctly() {
        let mut grid = Grid::new(3, 3);
        grid.get_mut(0, 0).unwrap().awake();
        grid.get_mut(1, 0).unwrap().awake();
        assert_eq!(grid.awake_neighbour(1, 1), 2);
    }

    #[test]
    fn grid_position() {
        let mut grid = Grid::new(3, 3);
        // for cell in grid.vec_vec_cell_grid.concat(){
        //     println!("{:#?}", cell);
        // }
        print!("{:#?}", grid);
    }
}