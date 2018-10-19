#![allow(dead_code, unused_imports)]
///! Simple speed tests. I should use `cargo bench` but I don't have time
///! to spend there now.

use game_engine::Game;
use random_agent::RandomAgent;

use std::time::{Instant, Duration};
use game_engine::GameBoard;


/// Test the performance with a random agents.
pub fn test_random_agent_simulation_speed(nb_agents: u32, continue_if_winner: bool) {
    let start_time = Instant::now();
    let mut steps: u128 = 0;
    const NB_SIMULATION: u32 = 100_000;
    for _ in 0..NB_SIMULATION {
        // Build the game
        let mut game = Game::new(30, 6);
        game.continue_simulation_if_known_winner(continue_if_winner);
        for id in 0..nb_agents {
            game.add_snake(id, Box::from(RandomAgent::new()));
        }

        // Execute the simulation and get results
        let results = game
            .initialize()
//            .print()
//            .after_each_step(|board: &GameBoard| board.print())
            .run_to_end();
//        println!("Results: {:?}", results);

        // Keep track of the total number of steps
        steps += results.steps as u128;
    };

    let duration = as_millis(start_time.elapsed());
    println!("Simulation with {} agents ended:\n\
              \t- {:12} simulations\n\
              \t- {:12} total steps\n\
              \t- {:12.3} total time ms\n\
              \t- {:12.3} steps/simulation\n\
              \t- {:12.3} simulations/sec\n\
              \t- {:12.3} steps/sec",
             nb_agents,
             NB_SIMULATION,
             steps,
             duration,
             steps as f64 / NB_SIMULATION as f64,
             NB_SIMULATION as f64 / (duration as f64 / 1000.),
             steps as f64 / (duration as f64 / 1000.)
    );
}

/// Returns a duration as milliseconds.
/// I don't want to use nightly features, otherwise there is a
/// `Duration::as_millis` method.
fn as_millis(duration: Duration) -> f64 {
    return duration.as_secs() as f64 * 1000.
        + duration.subsec_nanos() as f64 / 1_000_000.;
}