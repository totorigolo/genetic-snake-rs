#![allow(dead_code, unused_imports)]
///! Simple speed tests. I should use `cargo bench` but I don't have time
///! to spend there now.

use game_engine::Game;
use random_agent::RandomAgent;

use std::time::{Instant, Duration};
use game_engine::GameBoard;


/// Test the performance with a random agents.
pub fn test_random_agent_simulation_speed(nb_simulations: usize,
                                          nb_agents: u32,
                                          board_width: u16,
                                          board_height: u16,
                                          continue_if_winner: bool,
                                          print: bool) {
    let start_time = Instant::now();
    let mut steps: u128 = 0;
    for _ in 0..nb_simulations {
        // Build the game
        let mut game = Game::new(board_width, board_height);
        game.continue_simulation_if_known_winner(continue_if_winner);
        for id in 0..nb_agents {
            game.add_snake(id, Box::from(RandomAgent::new()));
        }
        if print {
            game.print()
                .after_each_step(|board: &GameBoard| board.print());
        }

        // Execute the simulation and get results
        let results = game
            .initialize()
            .run_to_end();

        if print {
            println!("Results: {:?}", results);
        }

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
             nb_simulations,
             steps,
             duration,
             steps as f64 / nb_simulations as f64,
             nb_simulations as f64 / (duration as f64 / 1000.),
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