extern crate rand;

mod game_engine;
mod random_agent;
mod bench_tests;

use game_engine::Game;
use game_engine::GameBoard;
use random_agent::RandomAgent;

use std::time::{Instant, Duration};

fn main() {
    let results = Game::new(30, 6)
        .continue_simulation_if_known_winner(true)
        .add_snake(0, Box::from(RandomAgent::new()))
        .initialize()
        .print()
        .after_each_step(|board: &GameBoard| board.print())
        .run_to_end();
    println!("Results: {:?}", results);

    bench_tests::test_random_agent_simulation_speed(1, true);
    bench_tests::test_random_agent_simulation_speed(2, false);
}
