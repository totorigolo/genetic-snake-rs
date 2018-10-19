#![allow(dead_code)]

use rand::prelude::*;

use std::fmt;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum SnakeBodyType {
    HEAD,
    BODY,
}

type SnakeId = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    EMPTY,
    FOOD,
    OBSTACLE,
    WALL,

    /// (Snake ID, Type)
    SNAKE(SnakeId, SnakeBodyType),
}

impl Cell {
    fn to_chr(&self) -> char {
        match self {
            Cell::EMPTY => ' ',
            Cell::FOOD => 'o',
            Cell::OBSTACLE => '@',
            Cell::WALL => '#',
            Cell::SNAKE(_, body_type) => match body_type {
                SnakeBodyType::HEAD => 'H',
                SnakeBodyType::BODY => 'B',
            }
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_chr())
    }
}

pub type Position = i32;

#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    x: i32,
    y: i32,
}

impl Coordinate {
    pub fn to_pos(&self, width: i32) -> i32 {
        // The position isn't checked because out-of-bounds means WALL.
        // assert!(...);

        self.x + self.y * width
    }

    pub fn from_pos(position: i32, width: i32) -> Self {
        // The position isn't checked because out-of-bounds means WALL.
        // assert!(self >= 0 && self < board.width * board.height);

        Coordinate {
            x: position % width,
            y: position / width,
        }
    }

    pub fn is_out_of_bounds(&self, board: &GameBoard) -> bool {
        return self.x < 0
            || self.x >= board.width
            || self.y < 0
            || self.y >= board.height;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    LEFT,
    FRONT,
    RIGHT,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Orientation {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

pub fn next_orientation(current_orientation: &Orientation, action: &Action) -> Orientation {
    match *current_orientation {
        Orientation::NORTH => match *action {
            Action::LEFT => Orientation::WEST,
            Action::FRONT => Orientation::NORTH,
            Action::RIGHT => Orientation::EAST,
        },
        Orientation::EAST => match *action {
            Action::LEFT => Orientation::NORTH,
            Action::FRONT => Orientation::EAST,
            Action::RIGHT => Orientation::SOUTH,
        },
        Orientation::SOUTH => match *action {
            Action::LEFT => Orientation::EAST,
            Action::FRONT => Orientation::SOUTH,
            Action::RIGHT => Orientation::WEST,
        },
        Orientation::WEST => match *action {
            Action::LEFT => Orientation::SOUTH,
            Action::FRONT => Orientation::WEST,
            Action::RIGHT => Orientation::NORTH,
        },
    }
}

pub fn next_coord_towards(from: &Coordinate, orientation: &Orientation) -> Coordinate {
    match orientation {
        Orientation::NORTH => Coordinate {
            x: from.x,
            y: from.y - 1,
        },
        Orientation::EAST => Coordinate {
            x: from.x + 1,
            y: from.y,
        },
        Orientation::SOUTH => Coordinate {
            x: from.x,
            y: from.y + 1,
        },
        Orientation::WEST => Coordinate {
            x: from.x - 1,
            y: from.y,
        },
    }
}

pub trait SnakeBot {
    fn get_next_action(&mut self, myself: &SnakeState, board: &GameBoard) -> Action;
}

pub struct SnakeState {
    /// Contains the positions of the snake body parts.
    /// The head is the front and the tail the back.
    pub positions: VecDeque<Position>,
    pub current_orientation: Orientation,
    pub alive: bool,
}

impl SnakeState {
    pub fn get_head_pos(&self) -> Position {
        *self.positions.front().unwrap()
    }

    pub fn get_head_coord(&self, board: &GameBoard) -> Coordinate {
        Coordinate::from_pos(self.get_head_pos(), board.width)
    }
}

struct Snake {
    id: SnakeId,
    state: SnakeState,
    bot: Box<SnakeBot>,

    /// The field gets decreased by one at each step. When it reaches zero,
    /// the snake grows and the field takes the value `GROWTH_RATE`.
    growth_state: i32,
}

impl Snake {
    pub const GROWTH_RATE: i32 = 3;
    pub const POSITIONS_VEC_INITIAL_CAPACITY: usize = 64;

    fn new(id: u32, bot: Box<SnakeBot>) -> Snake {
        Snake {
            id,
            state: SnakeState {
                positions: VecDeque::with_capacity(Self::POSITIONS_VEC_INITIAL_CAPACITY),
                current_orientation: Orientation::NORTH, // Random value
                alive: true,
            },
            bot,
            growth_state: Self::GROWTH_RATE,
        }
    }

    /// Returns whether the snake is dead or alive after this move.
    /// Heads-up: This doesn't say anything about collisions between snakes.
    fn make_a_step(&mut self, board: &mut GameBoard) {
        if !self.state.alive {
            eprintln!("make_a_step() called on a dead snake!");
            return;
        }

        // Ask the bot for its next action
        let action = self.bot.get_next_action(&self.state, board);

        let current_orientation: Orientation = self.state.current_orientation.clone();
        let next_orientation = next_orientation(&current_orientation, &action);
        let current_head_pos = self.state.positions.front()
            .expect("The game hasn't been initialized.")
            .clone();
        let current_head_coord = Coordinate::from_pos(current_head_pos, board.width);

        // Check if the move is in-bounds
        if (next_orientation == Orientation::WEST && current_head_coord.x == 0)
            || (next_orientation == Orientation::EAST && current_head_coord.x == board.width - 1)
            || (next_orientation == Orientation::NORTH && current_head_coord.y == 0)
            || (next_orientation == Orientation::SOUTH && current_head_coord.y == board.height - 1) {
            self.state.alive = false;
            self.remove_snake_from_board(board);
            return;
        }

        // Determine the next head position
        let next_head_coord = next_coord_towards(&current_head_coord, &next_orientation);
        let next_head_pos = next_head_coord.to_pos(board.width);

        // Remember if the next position is food
        let next_pos_type = (*board.get_tile_at_pos(next_head_pos)).clone();
        let food = next_pos_type == Cell::FOOD;

        // Check if the next position is free
        let free = match next_pos_type {
            Cell::EMPTY | Cell::FOOD => true,
            Cell::OBSTACLE | Cell::WALL | Cell::SNAKE(_, _) => false
        };
        if !free { // U dead, sry bro.
            self.state.alive = false;
            self.remove_snake_from_board(board);
            return;
        }

        // Check the growth rate
        assert!(self.growth_state > 0);
        self.growth_state -= 1;
        let growing = self.growth_state == 0;
        if growing {
            self.growth_state = Self::GROWTH_RATE;
        }

        // Update the snake
        self.state.positions.push_front(next_head_pos);
        self.state.current_orientation = next_orientation;

        // Update the head on the board
        board.set_tile_at_pos(
            current_head_pos, Cell::SNAKE(self.id, SnakeBodyType::BODY));
        board.set_tile_at_pos(
            next_head_pos, Cell::SNAKE(self.id, SnakeBodyType::HEAD));

        // Shrink the tail if didn't eat food
        if !food && !growing {
            if let Some(tail_pos) = self.state.positions.pop_back() {
                board.set_tile_at_pos(tail_pos, Cell::EMPTY);
            }
        }
    }

    fn remove_snake_from_board(&self, board: &mut GameBoard) {
        for position in &self.state.positions {
            board.set_tile_at_pos(*position, Cell::EMPTY);
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameResultWinner {
    WINNER(SnakeId),
    DRAW,
}

#[derive(Debug, Clone)]
pub struct GameResults {
    /// `winner` is None if there is only one snake and the notion of winner
    /// doesn't make sense.
    pub winner: Option<GameResultWinner>,
    pub steps: u32,
}

pub struct Game {
    board: GameBoard,
    snakes: Vec<Snake>,
    before_each_step: Vec<Box<Fn(&GameBoard)>>,
    after_each_step: Vec<Box<Fn(&GameBoard)>>,

    initialized: bool,
    step: u32,
    results: Option<GameResults>,

    /// If this field is `false` *and* there are more than one snake, then
    /// the simulation is stopped as soon as there is a winner. In other
    /// words, we don't continue the simulation with the remaining snake.
    lazy_simulation: bool,
}

impl Game {
    const NB_OBSTACLES: u32 = 5;
    const MAX_SIZE_OBSTACLE: u32 = 2;

    pub fn new(width: i32, height: i32) -> Game {
        let mut game = Game {
            board: GameBoard::new(width, height),
            snakes: vec![],
            before_each_step: vec![],
            after_each_step: vec![],
            initialized: false,
            step: 0,
            results: None,
            lazy_simulation: true,
        };
        game.board.add_random_obstacles(
            Self::NB_OBSTACLES,
            Self::MAX_SIZE_OBSTACLE,
        );
        game
    }

    pub fn add_snake(&mut self, id: SnakeId, snake_bot: Box<SnakeBot>) -> &mut Self {
        if self.snakes.iter().filter(|snake| snake.id == id).count() > 0 {
            panic!(format!("The ID {} is already used!", id));
        }
        self.snakes.push(Snake::new(id, snake_bot));
        self
    }

    pub fn before_each_step<F>(&mut self, func: F) -> &mut Self
        where F: Fn(&GameBoard) + 'static
    {
        self.before_each_step.push(Box::new(func));
        self
    }

    pub fn after_each_step<F>(&mut self, func: F) -> &mut Self
        where F: Fn(&GameBoard) + 'static
    {
        self.after_each_step.push(Box::new(func));
        self
    }

    pub fn continue_simulation_if_known_winner(&mut self, _continue: bool) -> &mut Self {
        self.lazy_simulation = !_continue;
        self
    }

    pub fn initialize(&mut self) -> &mut Self {
        let mut rng = thread_rng();
        let nb_cells = self.board.width * self.board.height;

        // Place the snakes on the board
        let head_positions = vec![];
        for snake in &mut self.snakes {
            let mut pos = None;
            for _ in 0..10_000 {
                let p = rng.gen_range(0, nb_cells);

                // Don't want two snakes at the same position
                if head_positions.contains(&p) {
                    continue;
                }

                // Check that the cell is free
                match *self.board.get_tile_at_pos(p) {
                    Cell::EMPTY | Cell::FOOD => {
                        pos = Some(p);
                        break;
                    }
                    _ => {} // Retry
                }
            }
            let pos = pos.expect("Not able to find an initial position for the snake.");

            // Update the snake
            snake.state.positions.push_front(pos);
            snake.state.current_orientation = Orientation::NORTH;

            // Update the board
            self.board.set_tile_at_pos(
                pos, Cell::SNAKE(snake.id, SnakeBodyType::HEAD));
        }
        self.initialized = true;
        self
    }

    pub fn print(&mut self) -> &mut Self {
        self.board.print();
        self
    }

    pub fn step(&mut self) -> &mut Self {
        assert!(self.initialized);
//        println!("Running step {}...", self.step);

        // Before-step callbacks
        for before_each_step in &self.before_each_step {
            before_each_step(&self.board);
        }

        // Remember which snakes are still alive
        let prev_alive_ids: Vec<SnakeId> = self.snakes.iter()
            .filter_map(|snake| if snake.state.alive { Some(snake.id) } else { None })
            .collect();
        let prev_nb_alive = prev_alive_ids.len();

        // Move the snakes
        let mut nb_alive = 0;
        for ref mut snake in self.snakes.iter_mut().filter(|snake| snake.state.alive) {
            snake.make_a_step(&mut self.board);
            if snake.state.alive {
                nb_alive += 1;
            }
        }

        // Verify if win/loose/draw
        // * Draw: all die
        if prev_nb_alive > 0 && nb_alive == 0 {
            self.results = Some(GameResults {
                winner: match self.snakes.len() > 1 {
                    true => Some(GameResultWinner::DRAW),
                    false => None // solo, no winner
                },
                steps: self.step + 1,
            });
        }
        // * Winner: last alive, >1 snake total
        if prev_nb_alive > 0 && nb_alive == 1 && self.snakes.len() > 1 {
            let winner_id: SnakeId = self.snakes.iter()
                .filter_map(|snake| if snake.state.alive { Some(snake.id) } else { None })
                .take(1)
                .next().unwrap();
            self.results = Some(GameResults {
                winner: Some(GameResultWinner::WINNER(winner_id)),
                steps: self.step + 1,
            });
        }

        // After-step callbacks
        for after_each_step in &self.after_each_step {
            after_each_step(&self.board);
        }

        self.step += 1;
        self
    }

    pub fn run_to_end(&mut self) -> GameResults {
        while self.results.is_none() || (!self.lazy_simulation
            && self.snakes.iter().filter(|snake| snake.state.alive).count() > 0) {
            self.step();
        }
        self.results.clone().unwrap()
    }

    pub fn is_game_over(&self) -> bool {
        self.results.is_some()
    }

    pub fn get_results(&self) -> Option<GameResults> {
        self.results.clone()
    }
}

/// Represents the game board.
///
/// `cells` is a 1D representation of the 2D board, where rows are "concatenated"
/// on one single row, so `(x, y)` is the `(x + y * width)`-th value.
pub struct GameBoard {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
}

impl GameBoard {
    fn new(width: i32, height: i32) -> GameBoard {
        GameBoard {
            width,
            height,
            cells: vec![Cell::EMPTY; (width * height) as usize],
        }
    }

    fn add_random_obstacles(&mut self, nb_obstacles: u32, max_size_obstacle: u32) {
        let mut rng = thread_rng();

        for _ in 0..nb_obstacles {
            let w: i32 = rng.gen_range(0, max_size_obstacle as i32) + 1;
            let x: i32 = rng.gen_range(0, self.width - w);
            let y: i32 = rng.gen_range(0, self.height - w);

            for i in 0..w {
                for j in 0..w {
                    let coord = Coordinate { x: x + i, y: y + j };
                    self.cells[coord.to_pos(self.width) as usize] = Cell::OBSTACLE;
                }
            }
        }
    }

    pub fn get_tile_at_coord(&self, coord: Coordinate) -> &Cell {
        self.get_tile_at_pos(coord.to_pos(self.width))
    }

    pub fn get_tile_at_pos(&self, pos: Position) -> &Cell {
        if pos >= 0 && pos < self.width * self.height {
            &self.cells[pos as usize]
        } else {
            &Cell::WALL
        }
    }

    pub fn set_tile_at_coord(&mut self, coord: Coordinate, cell: Cell) {
        let width = self.width;
        self.set_tile_at_pos(coord.to_pos(width), cell)
    }

    pub fn set_tile_at_pos(&mut self, pos: Position, cell: Cell) {
        if pos >= 0 && pos < self.width * self.height {
            self.cells[pos as usize] = cell;
        } else {
            panic!(format!("Position {} out-of-bounds: W={} H={} W*H={}",
                           pos, self.width, self.height, self.width * self.height));
        }
    }

    pub fn print(&self) {
        print!("+");
        for _ in 0..self.width {
            print!("-");
        }
        println!("+");

        let mut i = 0;
        for _ in 0..self.height {
            print!("|");
            for _ in 0..self.width {
                print!("{}", self.cells[i]);
                i += 1;
            }
            println!("|");
        }

        print!("+");
        for _ in 0..self.width {
            print!("-");
        }
        println!("+");
    }
}

fn rem(x: i32, y: i32) -> i32 {
    (x % y + y) % y
}
