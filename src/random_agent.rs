#![allow(dead_code)]

use rand::prelude::*;

use game_engine::{SnakeBot, SnakeState, GameBoard, Action, Coordinate, Orientation, Cell};
use game_engine::next_coord_towards;
use game_engine::next_orientation;

pub struct RandomAgent {
    rng: ThreadRng
}

impl RandomAgent {
    pub fn new() -> RandomAgent {
        RandomAgent {
            rng: thread_rng()
        }
    }
}

impl SnakeBot for RandomAgent {
    fn get_next_action(&mut self, myself: &SnakeState, board: &GameBoard) -> Action {
        let possible_actions = get_non_suicide_moves(
            &myself.get_head_coord(board), &myself.current_orientation, board);
        return if possible_actions.is_empty() {
            Action::FRONT // We're doomed, so don't care ^^'
        } else {
            let action_idx = self.rng.gen_range(0, possible_actions.len());
            let action = possible_actions[action_idx].clone();
            action
        }
    }
}

fn get_non_suicide_moves(from: &Coordinate,
                         orientation: &Orientation,
                         board: &GameBoard,
) -> Vec<Action> {
    [Action::LEFT, Action::FRONT, Action::RIGHT].iter()
        .filter_map(|action| {
            let action = action.clone();

            let next_orientation = next_orientation(&orientation, &action);
            let next_coord = next_coord_towards(&from, &next_orientation);

            if next_coord.is_out_of_bounds(board) {
                return None;
            }

            match *board.get_tile_at_coord(next_coord) {
                Cell::EMPTY | Cell::FOOD => Some(action),
                _ => None
            }
        })
        .collect()
}
