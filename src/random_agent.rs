use rand::prelude::*;

use game_engine::{SnakeBot, SnakeState, GameBoard, Action};

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
        let possible_actions = board.get_non_suicide_moves(
            &myself.get_head_coord(board), &myself.current_orientation);
        return if possible_actions.is_empty() {
            Action::FRONT // We're doomed, so don't care ^^'
        } else {
            let action_idx = self.rng.gen_range(0, possible_actions.len());
            let action = possible_actions[action_idx].clone();
            action
        }
    }
}
