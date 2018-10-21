use rand::prelude::*;

use std::fmt;
use std::collections::VecDeque;

use colored::Colorize;

pub type SnakeId = u32;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Cell {
    Empty,
    Food,
    Obstacle,
    Wall,
    SnakeHead(SnakeId),
    SnakeBody(SnakeId),
    SnakeTail(SnakeId),
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let colorize_snake = |id, s: String| match id {
            0 => s.green(),
            1 => s.red(),
            2 => s.blue(),
            3 => s.cyan(),
            _ => s.white(),
        };

        match self {
            Cell::Empty => write!(f, " "),
            Cell::Food => write!(f, "{}", "o".magenta().bold()),
            Cell::Obstacle => write!(f, "#"),
            Cell::Wall => unreachable!(),
            Cell::SnakeHead(id) => write!(f, "{}", colorize_snake(*id, "H".to_string())),
            Cell::SnakeTail(id) => write!(f, "{}", colorize_snake(*id, "T".to_string())),
            Cell::SnakeBody(id) => write!(f, "{}", colorize_snake(*id, format!("{}", id))),
        }
    }
}

pub type Position = i32;

#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    #[inline]
    pub fn to_pos(&self) -> i32 {
        // The position isn't checked because out-of-bounds means WALL.
        // assert!(...);

        self.x + self.y * BOARD_WIDTH
    }

    #[inline]
    pub fn from_pos(position: i32) -> Self {
        // The position isn't checked because out-of-bounds means WALL.
        // assert!(self >= 0 && self < BOARD_WIDTH * BOARD_HEIGHT);

        Coordinate {
            x: position % BOARD_WIDTH,
            y: position / BOARD_WIDTH,
        }
    }

    #[inline]
    pub fn is_out_of_bounds(&self) -> bool {
        return self.x < 0 || self.x >= BOARD_WIDTH || self.y < 0 || self.y >= BOARD_HEIGHT;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Left,
    Front,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

pub fn next_orientation(current_orientation: &Orientation, action: &Action) -> Orientation {
    match *current_orientation {
        Orientation::North => {
            match *action {
                Action::Left => Orientation::West,
                Action::Front => Orientation::North,
                Action::Right => Orientation::East,
            }
        }
        Orientation::East => {
            match *action {
                Action::Left => Orientation::North,
                Action::Front => Orientation::East,
                Action::Right => Orientation::South,
            }
        }
        Orientation::South => {
            match *action {
                Action::Left => Orientation::East,
                Action::Front => Orientation::South,
                Action::Right => Orientation::West,
            }
        }
        Orientation::West => {
            match *action {
                Action::Left => Orientation::South,
                Action::Front => Orientation::West,
                Action::Right => Orientation::North,
            }
        }
    }
}

/// Returns None if the move leads outside of the board
pub fn next_coord_towards(from: &Coordinate, orientation: &Orientation) -> Option<Coordinate> {

    // Check if the move is in-bounds
    if (*orientation == Orientation::West && from.x == 0) ||
        (*orientation == Orientation::East && from.x == BOARD_WIDTH - 1) ||
        (*orientation == Orientation::North && from.y == 0) ||
        (*orientation == Orientation::South && from.y == BOARD_HEIGHT - 1) {
        return None;
    }

    let next_coord = match orientation {
        Orientation::North => {
            Coordinate {
                x: from.x,
                y: from.y - 1,
            }
        }
        Orientation::East => {
            Coordinate {
                x: from.x + 1,
                y: from.y,
            }
        }
        Orientation::South => {
            Coordinate {
                x: from.x,
                y: from.y + 1,
            }
        }
        Orientation::West => {
            Coordinate {
                x: from.x - 1,
                y: from.y,
            }
        }
    };
    Some(next_coord)
}

pub trait SnakeBot {
    fn get_next_action(&mut self,
                       myself: &SnakeState,
                       board: &GameBoard)
                       -> Action;
}

pub struct SnakeState {
    pub id: SnakeId,
    /// Contains the positions of the snake body parts.
    /// The head is the front and the tail the back.
    pub positions: VecDeque<Position>,
    pub current_orientation: Orientation,
    pub alive: bool,
}

impl SnakeState {
    #[inline]
    pub fn get_head_pos(&self) -> Position {
        *self.positions.front().expect("get_head_pos() called before the game started.")
    }

    #[inline]
    pub fn get_head_coord(&self) -> Coordinate {
        Coordinate::from_pos(self.get_head_pos())
    }
}

pub struct Snake<'a> {
    pub state: SnakeState,
    bot: Box<SnakeBot + 'a>,
    just_died: bool,

    /// The field gets decreased by one at each step. When it reaches zero,
    /// the snake grows and the field takes the value `GROWTH_RATE`.
    growth_state: i32,
}

impl<'a> Snake<'a> {
    pub const GROWTH_RATE: i32 = 3;
    pub const POSITIONS_VEC_INITIAL_CAPACITY: usize = 64;

    fn new(id: u32, bot: Box<SnakeBot + 'a>) -> Self {
        Snake {
            state: SnakeState {
                id,
                positions: VecDeque::with_capacity(Self::POSITIONS_VEC_INITIAL_CAPACITY),
                current_orientation: Orientation::North, // Random value
                alive: true,
            },
            bot,
            just_died: false,
            growth_state: Self::GROWTH_RATE,
        }
    }

    /// Ask the `SnakeBot` its next action.
    /// Returns None if the snake is dead.
    fn get_next_action(&mut self, board: &GameBoard) -> Option<Action> {
        if !self.state.alive {
            eprintln!("get_next_action() called on a dead snake!");
            return None;
        }

        // Ask the bot for its next action
        Some(self.bot.get_next_action(&self.state, board))
    }

    /// Returns whether the snake is dead or alive after this move.
    /// Heads-up: This doesn't say anything about collisions between snakes.
    fn execute_action(&mut self, board: &mut GameBoard, action: &Action) {
        if !self.state.alive {
            eprintln!("execute_action() called on a dead snake!");
            return;
        }

        let current_orientation: Orientation = self.state.current_orientation.clone();
        let next_orientation = next_orientation(&current_orientation, &action);
        let current_head_pos = self.state
            .positions
            .front()
            .expect("The game hasn't been initialized.")
            .clone();
        let current_head_coord = Coordinate::from_pos(current_head_pos);

        // Determine the next head coordinate
        let next_head_coord = next_coord_towards(&current_head_coord, &next_orientation);

        // Check if the next position is out of the board => death & return
        if next_head_coord.is_none() {
            self.just_died = true;
            board.set_tile_at_pos(current_head_pos, Cell::SnakeBody(self.state.id));
            return;
        }
        let next_head_coord = next_head_coord.unwrap();

        // Check if the next position is free => death
        if !board.is_coord_free_or_food(&next_head_coord) {
            self.just_died = true;
        }

        // Convert the coordinate to a position
        let next_head_pos = next_head_coord.to_pos();

        // Remember if the next position is food
        let next_pos_type = board.get_tile_at_pos(&next_head_pos).clone();
        let food = next_pos_type == Cell::Food;

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

        // Change the current head to body
        board.set_tile_at_pos(current_head_pos, Cell::SnakeBody(self.state.id));

        // Shrink the tail if doesn't grow
        // FIXME: If >two heads go on the same cell, only the first snake eats the food.
        if !(food || growing) {
            if let Some(tail_pos) = self.state.positions.pop_back() {
                board.set_tile_at_pos(tail_pos, Cell::Empty);
            }
        }

        // Update the head and tail on the board
        board.set_tile_at_pos(*self.state.positions.back().expect("0-length Snake in execute_action()."),
                              Cell::SnakeTail(self.state.id));
        board.set_tile_at_pos(next_head_pos,
                              Cell::SnakeHead(self.state.id));
    }

    fn remove_snake_from_board(&self, board: &mut GameBoard) {
        for position in &self.state.positions {
            board.set_tile_at_pos(*position, Cell::Empty);
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameResultWinner {
    Winner(SnakeId),
    Draw,
}

#[derive(Debug, Clone)]
pub struct GameResults {
    /// `winner` is None if there is only one snake and the notion of winner
    /// doesn't make sense.
    pub winner: Option<GameResultWinner>,
    pub steps: u32,
}

pub struct Game<'a> {
    board: GameBoard,
    snakes: Vec<Snake<'a>>,
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

impl<'a> Game<'a> {
    const NB_OBSTACLES: u32 = 5;
    const MAX_SIZE_OBSTACLE: u32 = 2;

    pub fn new() -> Self {
        let mut game = Game {
            board: GameBoard::new(),
            snakes: vec![],
            before_each_step: vec![],
            after_each_step: vec![],
            initialized: false,
            step: 0,
            results: None,
            lazy_simulation: true,
        };
        game.board.add_random_obstacles(Self::NB_OBSTACLES, Self::MAX_SIZE_OBSTACLE);
        game
    }

    pub fn add_snake(&mut self, id: SnakeId, snake_bot: Box<SnakeBot + 'a>) -> &mut Self {
        if self.snakes.iter().filter(|snake| snake.state.id == id).count() > 0 {
            panic!(format!("The ID {} is already used!", id));
        }
        self.snakes.push(Snake::new(id, snake_bot));
        self
    }

    #[allow(dead_code)]
    pub fn before_each_step<F>(&mut self, func: F) -> &mut Self
        where F: Fn(&GameBoard) + 'static
    {
        self.before_each_step.push(Box::new(func));
        self
    }

    #[allow(dead_code)]
    pub fn after_each_step<F>(&mut self, func: F) -> &mut Self
        where F: Fn(&GameBoard) + 'static
    {
        self.after_each_step.push(Box::new(func));
        self
    }

    #[allow(dead_code)]
    pub fn continue_simulation_if_known_winner(&mut self, _continue: bool) -> &mut Self {
        self.lazy_simulation = !_continue;
        self
    }

    pub fn initialize(&mut self) -> &mut Self {
        let mut rng = thread_rng();
        let nb_cells = BOARD_WIDTH * BOARD_HEIGHT;

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
                match self.board.get_tile_at_pos(&p) {
                    Cell::Empty | Cell::Food => {
                        pos = Some(p);
                        break;
                    }
                    _ => {} // Retry
                }
            }
            let pos = pos.expect("Not able to find an initial position for the snake.");

            // Update the snake
            snake.state.positions.push_front(pos);
            snake.state.current_orientation = Orientation::North;

            // Update the board
            self.board.set_tile_at_pos(pos, Cell::SnakeHead(snake.state.id));
        }
        self.initialized = true;
        self
    }

    #[allow(dead_code)]
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
        let prev_nb_alive = self.snakes.iter()
            .filter(|snake| snake.state.alive)
            .map(|snake| snake.state.id)
            .count();

        // Update the board before calling the bots
        self.board.nb_alive_snakes = prev_nb_alive;

        // Take the snakes' next actions
        let mut actions = vec![];
        for ref mut snake in self.snakes.iter_mut()
            .filter(|snake| snake.state.alive) {
            actions.push(snake.get_next_action(&self.board));
        }

        // Move the snakes
        for (ref mut snake, ref action) in self.snakes.iter_mut()
            .filter(|snake| snake.state.alive)
            .zip(actions) {
            if let Some(action) = action {
                snake.execute_action(&mut self.board, action);
            }
        }

        // Check head collisions
        for ref mut snake in self.snakes.iter_mut()
            .filter(|snake| snake.state.alive) {
            if let Some(head) = snake.state.positions.front() {
                if let Cell::SnakeHead(id) = self.board.get_tile_at_pos(&head) {
                    if id != snake.state.id {
                        snake.just_died = true;
                    }
                }
            }
        }

        // Remove the dead snakes from the board
        for ref mut snake in self.snakes.iter_mut().filter(|snake| snake.just_died) {
            snake.remove_snake_from_board(&mut self.board);
            snake.just_died = false;
            snake.state.alive = false;
        }

        // Count the live snakes
        let nb_alive = self.snakes.iter().filter(|snake| snake.state.alive).count();

        // Verify if win/loose/draw
        if self.results.is_none() {
            // * Draw/end: all die
            if prev_nb_alive > 0 && nb_alive == 0 {
                self.results = Some(GameResults {
                    winner: match self.snakes.len() > 1 {
                        true => Some(GameResultWinner::Draw),
                        false => None, // solo, no winner
                    },
                    steps: self.step + 1,
                });
            }
            // * Winner: last alive, >1 snake total
            if prev_nb_alive > 0 && nb_alive == 1 && self.snakes.len() > 1 {
                let winner_id: SnakeId = self.snakes
                    .iter()
                    .filter(|snake| snake.state.alive)
                    .map(|snake| snake.state.id)
                    .next()
                    .expect("Logic error: nb_alive == 1 but none found in self.snakes.");
                self.results = Some(GameResults {
                    winner: Some(GameResultWinner::Winner(winner_id)),
                    steps: self.step + 1,
                });
            }
        }

        // Update the board
        self.board.update();

        // After-step callbacks
        for after_each_step in &self.after_each_step {
            after_each_step(&self.board);
        }

        self.step += 1;
        self
    }

    pub fn run_to_end(&mut self) -> GameResults {
        while self.results.is_none() ||
            (!self.lazy_simulation &&
                self.snakes.iter().filter(|snake| snake.state.alive).count() > 0) {
            self.step();
        }
        self.results.clone().expect("Logic error, no result in run_to_end().")
    }

    #[allow(dead_code)]
    pub fn is_game_over(&self) -> bool {
        self.results.is_some()
    }

    #[allow(dead_code)]
    pub fn get_results(&self) -> Option<GameResults> {
        self.results.clone()
    }
}

pub const BOARD_WIDTH: i32 = 32;
pub const BOARD_HEIGHT: i32 = 16;

/// Represents the game board.
///
/// `cells` is a 1D representation of the 2D board, where rows are "concatenated"
/// on one single row, so `(x, y)` is the `(x + y * width)`-th value.
pub struct GameBoard {
    /// The number of non-OBSTACLE cells.
    pub nb_free_cells: i32,
    pub nb_alive_snakes: usize,
    cells: [Cell; (BOARD_WIDTH * BOARD_HEIGHT) as usize],

    rng: ThreadRng,
    food_add_probability: f32,
}

impl GameBoard {
    fn new() -> Self {
        GameBoard {
            nb_free_cells: BOARD_WIDTH * BOARD_HEIGHT,
            nb_alive_snakes: 0,
            cells: [Cell::Empty; (BOARD_WIDTH * BOARD_HEIGHT) as usize],

            rng: thread_rng(),
            food_add_probability: 0.1,
        }
    }

    fn add_random_obstacles(&mut self, nb_obstacles: u32, max_size_obstacle: u32) {
        let mut rng = thread_rng();

        for _ in 0..nb_obstacles {
            let w: i32 = rng.gen_range(0, max_size_obstacle as i32) + 1;
            let x: i32 = rng.gen_range(0, BOARD_WIDTH - w);
            let y: i32 = rng.gen_range(0, BOARD_HEIGHT - w);

            for i in 0..w {
                for j in 0..w {
                    let coord = Coordinate {
                        x: x + i,
                        y: y + j,
                    };
                    self.cells[coord.to_pos() as usize] = Cell::Obstacle;
                    self.nb_free_cells -= 1;
                }
            }
        }
    }

    fn update(&mut self) {
        self.nb_free_cells = 0;
        for i in 0..(BOARD_WIDTH * BOARD_HEIGHT) as usize {
            match self.cells[i] {
                Cell::Empty | Cell::Food => self.nb_free_cells += 1,
                _ => {},
            }
        }

        self.update_food();
    }

    fn update_food(&mut self) {
        let p = self.rng.gen_range(0., 1.);
        if p < self.food_add_probability {
            let x = self.rng.gen_range(0, BOARD_WIDTH);
            let y = self.rng.gen_range(0, BOARD_HEIGHT);
            let coord = Coordinate { x, y };
            let pos = coord.to_pos();
            if self.is_pos_free_or_food(&pos) {
                self.set_tile_at_pos(pos, Cell::Food);
            }
        }
    }

    pub fn get_tile_at_coord(&self, coord: &Coordinate) -> Cell {
        if coord.is_out_of_bounds() {
            return Cell::Wall;
        }
        self.get_tile_at_pos(&coord.to_pos())
    }

    pub fn get_tile_at_pos(&self, pos: &Position) -> Cell {
//        assert!(*pos >= 0 && *pos < BOARD_WIDTH * BOARD_HEIGHT);
        self.cells[*pos as usize]
    }

    #[allow(dead_code)]
    pub fn set_tile_at_coord(&mut self, coord: &Coordinate, cell: Cell) {
        self.set_tile_at_pos(coord.to_pos(), cell)
    }

    pub fn set_tile_at_pos(&mut self, pos: Position, cell: Cell) {
        if pos >= 0 && pos < BOARD_WIDTH * BOARD_HEIGHT {
            self.cells[pos as usize] = cell;
        } else {
            panic!(format!("Position {} out-of-bounds: W={} H={} W*H={}",
                           pos,
                           BOARD_WIDTH, BOARD_HEIGHT,
                           BOARD_WIDTH * BOARD_HEIGHT));
        }
    }

    pub fn is_suicide_moves(&self,
                            from: &Coordinate,
                            orientation: &Orientation,
                            action: &Action)
                            -> bool {
        let next_orientation = next_orientation(&orientation, action);
        let next_coord = next_coord_towards(&from, &next_orientation);
        if next_coord.is_none() {
            return true;
        }
        let next_coord = next_coord.unwrap();
        assert!(!next_coord.is_out_of_bounds()); // TODO: Remove -> useless

        !self.is_coord_free_or_food(&next_coord)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn is_pos_free_or_food(&self, pos: &Position) -> bool {
        match self.get_tile_at_pos(pos) {
            Cell::Empty | Cell::Food => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_coord_free_or_food(&self, coord: &Coordinate) -> bool {
        match self.get_tile_at_coord(coord) {
            Cell::Empty | Cell::Food => true,
            _ => false,
        }
    }

    pub fn get_non_suicide_moves(&self,
                                 from: &Coordinate,
                                 orientation: &Orientation)
                                 -> Vec<Action> {
        [Action::Left, Action::Front, Action::Right]
            .iter()
            .filter_map(|action| {
                match self.is_suicide_moves(from, orientation, action) {
                    false => Some(action.clone()),
                    true => None,
                }
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        print!("+");
        for _ in 0..BOARD_WIDTH {
            print!("-");
        }
        println!("+");

        let mut i = 0;
        for _ in 0..BOARD_HEIGHT {
            print!("|");
            for _ in 0..BOARD_WIDTH {
                print!("{}", self.cells[i]);
                i += 1;
            }
            println!("|");
        }

        print!("+");
        for _ in 0..BOARD_WIDTH {
            print!("-");
        }
        println!("+");
    }
}
