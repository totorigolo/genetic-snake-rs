use rand::prelude::*;

use game_engine::*;

pub struct RandomBot {
    rng: ThreadRng
}

impl RandomBot {
    pub fn new() -> Self {
        RandomBot {
            rng: thread_rng()
        }
    }
}

impl Default for RandomBot {
    fn default() -> Self {
        Self::new()
    }
}

impl SnakeBot for RandomBot {
    fn get_next_action(&mut self,
                       myself: &SnakeState,
                       board: &GameBoard)
                       -> Action {
        get_non_suicide_random_action(&mut self.rng, myself, board)
    }
}

pub fn get_non_suicide_random_action(rng: &mut impl Rng,
                                     myself: &SnakeState,
                                     board: &GameBoard) -> Action {
    let possible_actions = board.get_non_suicide_moves(
        &myself.get_head_coord(), &myself.current_orientation);

    return if possible_actions.is_empty() {
        Action::Front // We're doomed, so don't care ^^'
    } else {
        let action_idx = rng.gen_range(0, possible_actions.len());
        let action = possible_actions[action_idx].clone();
        action
    };
}
