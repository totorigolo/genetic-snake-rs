use std::collections::VecDeque;
use std::cmp::Ordering;
use std::cmp::min;

use game_engine::*;
use random_bot::get_non_suicide_random_action;

/// The number of weights needed by the `HeuristicBot`.
pub const NB_WEIGHTS: usize = 5 * 3;

/// The heuristic weights
pub type Weights = [f64; NB_WEIGHTS];

/// The maximum depth for the BFS => sight distance.
/// Attention: It's used to normalize `Stats::accessible_area`.
pub const MAX_DEPTH: i32 = 30;

/// Human-tuned good weights
lazy_static!(
    pub static ref GOOD_WEIGHTS: Weights = {
        [
            1., 0.2, 0.07, -0.1, -0.01,
            1., 0.2, 0.07, -0.1, -0.01,
            1., 0.2, 0.07, -0.1, -0.01
        ]
    };
);

pub struct HeuristicBot<'a> {
    /// This is a reference to prevent an unneeded copy during
    /// the genetic algorithm learning process.
    weights: &'a Weights,
}

impl<'a> HeuristicBot<'a> {
    pub fn new(weights: &'a Weights) -> Self {
        assert_eq!(weights.len(), NB_WEIGHTS,
                   "Got {} weights, but {} are needed.",
                   weights.len(), NB_WEIGHTS);
        HeuristicBot {
            weights,
        }
    }
}

impl<'a> Default for HeuristicBot<'a> {
    /// Creates a `HeuristicBot` with human-tuned weights.
    /// Note: The weights are `GOOD_WEIGHTS`.
    fn default() -> Self {
        Self::new(&GOOD_WEIGHTS)
    }
}

impl<'a> SnakeBot for HeuristicBot<'a> {
    fn get_next_action(&mut self,
                       myself: &SnakeState,
                       board: &GameBoard)
                       -> Action {
        let current_orientation = &myself.current_orientation;
        let head_pos = *myself.positions.front().unwrap();
        let head_coord = Coordinate::from_pos(head_pos);

        [Action::Left, Action::Front, Action::Right]
            .iter().enumerate()
            .map(|(i, action)| {
                let next_orientation = next_orientation(current_orientation, &action);
                let next_coord = next_coord_towards(&head_coord, &next_orientation);

                let stats = compute_stats_from(&myself.id, &next_coord, board);
                let offset = i * stats.len();
                let weight =
                    stats.accessible_area * self.weights[offset + 0]
                        + stats.num_accessible_food * self.weights[offset + 1]
                        + stats.sum_dist_enemy_heads * self.weights[offset + 2]
                        + stats.sum_dist_enemy_tails * self.weights[offset + 3]
                        + stats.min_dist_to_food * self.weights[offset + 4]
                ;

//                println!("{:?}:\n\
//                          \t-> {:?} => {:?}\n\
//                          \t-> {:?}\n\
//                          \t=> {}",
//                         action,
//                         head_coord, next_coord,
//                         stats,
//                         weight);

                (action, weight)
            })
            .max_by_key(|(_, weight)| NonNan::new(weight.clone()))
            .unwrap()
            .0.clone()
    }
}

#[derive(Debug)]
pub struct Stats {
    pub accessible_area: f64,
    pub num_accessible_food: f64,
    pub sum_dist_enemy_heads: f64,
    pub sum_dist_enemy_tails: f64,
    pub min_dist_to_food: f64,
}

impl Stats {
    fn new(accessible_area: f64,
           num_accessible_food: f64,
           sum_dist_enemy_heads: f64,
           sum_dist_enemy_tails: f64,
           min_dist_to_food: f64)
           -> Self {
        Stats {
            accessible_area,
            num_accessible_food,
            sum_dist_enemy_heads,
            sum_dist_enemy_tails,
            min_dist_to_food,
        }
    }

    fn len(&self) -> usize {
        5
    }
}

/// `coord` is an Option because we don't forbid suicide.
pub fn compute_stats_from(snake_id: &SnakeId, coord: &Option<Coordinate>, board: &GameBoard) -> Stats {
    assert!(MAX_DEPTH > 0);
    assert!(BOARD_WIDTH > 0);
    assert!(BOARD_HEIGHT > 0);

    let board_diag_size = ((BOARD_WIDTH.pow(2) + BOARD_HEIGHT.pow(2)) as f64)
        .sqrt().ceil();
    const NB_CELLS: usize = (BOARD_WIDTH * BOARD_HEIGHT) as usize;

    // The stats
    let mut accessible_area = 0.;
    let mut num_accessible_food = 0;
    let mut sum_dist_enemy_heads = 0.;
    let mut sum_dist_enemy_tails = 0.;
    let mut min_dist_to_food = board_diag_size as i32;

    // Added set and fringe queue
    let mut added = [false; NB_CELLS];
    let mut queue = [(0, 0); NB_CELLS];
    let mut queue_front: usize = 0;
    let mut queue_back: usize = 0;

    // Only add the start coordinate if it's in a free cell
    // => don't perform the BFS if not free
    if let Some(coord) = coord {
        if board.is_coord_free_or_food(&coord) {
            let pos = coord.to_pos();
            queue[queue_back] = (pos, 0_i32);
            queue_back += 1;
            added[pos as usize] = true;
        }
    }

    // BFS
    while queue_front < queue_back {
        // Pop the next position
        let (pos, dist) = queue[queue_front];
        queue_front += 1;

        // Check the max depth
        if dist > MAX_DEPTH {
            break;
        }

        // Update the stats depending on the current free-tile type
        match board.get_tile_at_pos(&pos) {
            Cell::Empty => accessible_area += 1.,// / (dist + 1) as f64,
            Cell::Food => {
                accessible_area += 1.;// / (dist + 1) as f64;
                num_accessible_food += 1;
                min_dist_to_food = min(dist, min_dist_to_food);
            }
            _ => {}
        }

        // Add the neighbors to the fringe
        let Coordinate { x, y } = Coordinate::from_pos(pos);
        [
            Coordinate { x: x - 1, y },
            Coordinate { x: x + 1, y },
            Coordinate { x, y: y - 1 },
            Coordinate { x, y: y + 1 },
        ].iter()
            .for_each(|coord| {
                let pos = coord.to_pos();
                if !coord.is_out_of_bounds() && !added[pos as usize] {
                    // Update the stats depending on the neighbor non-free-tile type
                    match board.get_tile_at_pos(&pos) {
                        Cell::SnakeHead(id) => {
                            if id != *snake_id {
                                sum_dist_enemy_heads += dist as f64;
                            }
                        }
                        Cell::SnakeTail(id) => {
                            if id != *snake_id {
                                sum_dist_enemy_tails += dist as f64;
                            }
                        }
                        Cell::Empty | Cell::Food => {
                            // Add the neighbor the the fringe
                            queue[queue_back] = (pos, dist + 1);
                            queue_back += 1;
                        }
                        _ => {}
                    }
                    added[pos as usize] = true;
                }
            });
    }

    // dists are "inf"=1 by default
    let max_sum_dist_enemy = (board.nb_alive_snakes - 1) as f64 * board_diag_size;
    if sum_dist_enemy_heads == 0. {
        sum_dist_enemy_heads = max_sum_dist_enemy;
    }
    if sum_dist_enemy_tails == 0. {
        sum_dist_enemy_tails = max_sum_dist_enemy;
    }

    // Return normalized stats
    let nb_free_cells = board.nb_free_cells;
    return Stats::new(
        // TODO: Normalize accessible_area with other directions?
        accessible_area as f64 / nb_free_cells as f64,
//        accessible_area as f64 / MAX_DEPTH as f64,
        num_accessible_food as f64 / nb_free_cells as f64, // TODO: Normalize with num_food?
        sum_dist_enemy_heads as f64 / max_sum_dist_enemy,
        sum_dist_enemy_tails as f64 / max_sum_dist_enemy,
        min_dist_to_food as f64 / board_diag_size,
    )
}

#[derive(PartialEq, PartialOrd)]
struct NonNan(f64);

impl NonNan {
    #[inline]
    fn new(val: f64) -> Option<NonNan> {
        if val.is_nan() {
            panic!("NonNan::new() called with NaN value.");
        }
        Some(NonNan(val))
    }
}

impl Eq for NonNan {}

impl Ord for NonNan {
    #[inline]
    fn cmp(&self, other: &NonNan) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
