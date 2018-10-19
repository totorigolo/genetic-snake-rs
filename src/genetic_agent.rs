use rand::prelude::*;

use genevo::genetic::FitnessFunction;

use game_engine::{SnakeBot, SnakeState, GameBoard, Action};
use game_engine::Game;
use random_agent::RandomAgent;
use game_engine::GameResultWinner;

/// The genotype is a vector of coefficients.
pub type GeneticAgentGenome = Vec<f32>;

pub const GENETIC_AGENT_GENOME_LENGTH: usize = 4;
pub const GENETIC_AGENT_GENOME_MIN_VALUE: f32 = -1.;
pub const GENETIC_AGENT_GENOME_MAX_VALUE: f32 = 1.;

/// The fitness function for `GeneticAgentGenome`s.
#[derive(Clone)]
pub struct WinRatioFitnessCalc;

impl WinRatioFitnessCalc {
    const NB_MATCHES: usize = 15;
}

impl FitnessFunction<GeneticAgentGenome, usize> for WinRatioFitnessCalc {
    fn fitness_of(&self, genome: &GeneticAgentGenome) -> usize {
        let mut nb_wins = 0;
        for _ in 0..Self::NB_MATCHES {
            let results = Game::new(30, 6)
                .continue_simulation_if_known_winner(false)
                .add_snake(0, Box::from(RandomAgent::new()))
                .add_snake(1, Box::from(GeneticAgent::new()))
                .initialize()
                .run_to_end();
//            println!("{:?}", results);
            match results.winner {
                Some(GameResultWinner::WINNER(id)) => {
                    if id == 1 {
                        nb_wins += 2;
                    };
                }
                Some(GameResultWinner::DRAW) => nb_wins += 1,
                _ => {}
            }
        }

        nb_wins
    }

    fn average(&self, fitness_values: &[usize]) -> usize {
        fitness_values.iter().sum::<usize>() / fitness_values.len()
    }

    fn highest_possible_fitness(&self) -> usize {
        Self::NB_MATCHES * 2
    }

    fn lowest_possible_fitness(&self) -> usize {
        0
    }
}

pub struct GeneticAgent {
    rng: ThreadRng,
    genome: GeneticAgentGenome,
}

impl GeneticAgent {
    pub fn new() -> GeneticAgent {
        GeneticAgent {
            rng: thread_rng(),
            genome: vec![],
        }
    }
}

impl SnakeBot for GeneticAgent {
    fn get_next_action(&mut self, myself: &SnakeState, board: &GameBoard) -> Action {
        let possible_actions = board.get_non_suicide_moves(
            &myself.get_head_coord(board), &myself.current_orientation);
        return if possible_actions.is_empty() {
            Action::FRONT // We're doomed, so don't care ^^'
        } else {
            let action_idx = self.rng.gen_range(0, possible_actions.len());
            let action = possible_actions[action_idx].clone();
            action
        };
    }
}
