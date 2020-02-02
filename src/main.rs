#![allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

use std::{
    time::{
        Duration, Instant,
    },
    thread,
};

use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirmation, Input, Select};

mod bench_tests;
mod game_engine;
mod heuristic_bot;
mod interactive_bot;
mod learning;
mod random_bot;

use crate::heuristic_bot::{HeuristicBot, Weights, NB_WEIGHTS};
use crate::interactive_bot::InteractiveBot;
use crate::learning::learning;
use crate::random_bot::RandomBot;
use crate::game_engine::{SnakeId, SnakeBot, Game, GameBoard, BOARD_HEIGHT};

lazy_static! {
    /// Global dialog theme
    pub static ref DIALOG_THEME: ColorfulTheme = {
        ColorfulTheme {
            values_style: Style::new().yellow().dim(),
            indicator_style: Style::new().yellow().bold(),
            yes_style: Style::new().yellow().dim(),
            no_style: Style::new().yellow().dim(),
            ..ColorfulTheme::default()
        }
    };
}

fn main() {
    loop {
        // Ask what to do
        let main_choice = Select::with_theme(&*DIALOG_THEME)
            .with_prompt("What do you want to do?")
            .default(0)
            .item("start the genetic algorithm")
            .item("play against the best bot")
            .item("see a match between bots")
            .item("speed test!")
            .item("quit")
            .interact()
            .unwrap_or(4);

        match main_choice {
            0 => {
                learning();
                break;
            },
            1 => human_vs_good_bot(),
            2 => start_match(prompt_and_create_bots()),
            3 => speed_test(),
            _ => break
        }
        println!();
    }
}

lazy_static! {
    /// Weights learned with the GA, which got 38/40 or 40/40
    pub static ref GA_WEIGHTS: Weights = {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        // 38/40
        // let weights: [f64; NB_WEIGHTS] = [
        //     0.97500, -0.64724, -0.24451, -0.30122, -0.25775,
        //     0.97500, -0.62002, -0.64823, -0.23038, 0.06820,
        //     1.00000, -0.64373, -0.08643, -0.33367, -0.38482,
        // ];
        // 38/40
        // let weights: [f64; NB_WEIGHTS] = [
        //     0.95000,   0.62497,  -0.04825,  -0.49258,  -0.17677,
        //     0.97500,   0.42442,  -0.63253,  -0.16685,  -0.05459,
        //     0.97500,  -0.57496,  -0.10656,  -0.34064,  -0.27314,
        // ];
        // 40/40
        let weights: [f64; NB_WEIGHTS] = [
            1.02867,  -0.62294,  -0.08552,  -0.36006,  -0.24858,
            1.07254,  -0.13452,  -0.45125,  -0.31519,   0.01470,
            1.03946,   0.38929,   0.01750,  -0.55665,  -0.18053,
        ];
        weights.iter().cloned().collect()
    };
}

fn human_vs_good_bot() {
    let results = Game::new()
        .continue_simulation_if_known_winner(false)
        .add_snake(0, Box::from(HeuristicBot::new(&GA_WEIGHTS)))
        .add_snake(1, Box::from(InteractiveBot::default()))
        .initialize()
        .print()
        .after_each_step(move |board: &GameBoard| {
            board.print()
        })
        .run_to_end();
    println!("{}", results);
}

enum Bot {
    Random,
    Heuristic,
    Best,
    Interactive,
}

fn prompt_and_create_bots() -> Vec<Box<dyn SnakeBot>> {
    let nb_players = loop {
        let nb_players = Input::with_theme(&*DIALOG_THEME)
            .with_prompt("How many players?")
            .default(2)
            .interact()
            .unwrap_or(2);

        if nb_players <= 5 {
            break nb_players;
        }
    };

    let mut bots: Vec<Box<dyn SnakeBot>> = vec![];
    for id in 1..=nb_players {
        let bot = prompt_which_bot(&format!("Which bot do you want for player {}?", id));
        match bot {
            Bot::Random => bots.push(Box::new(RandomBot::new())),
            Bot::Heuristic => bots.push(Box::new(HeuristicBot::default())),
            Bot::Best => bots.push(Box::new(HeuristicBot::new(&GA_WEIGHTS))),
            Bot::Interactive => bots.push(Box::new(InteractiveBot)),
        };
    }
    bots
}

fn prompt_which_bot(msg: &str) -> Bot {
    match Select::with_theme(&*DIALOG_THEME)
        .with_prompt(msg)
        .default(0)
        .item("random bot")
        .item("human-tuned heuristic bot")
        .item("best bot found with genetic algorithm")
        .item("human")
        .interact()
        .unwrap_or(0) {
        0 => Bot::Random,
        1 => Bot::Heuristic,
        2 => Bot::Best,
        3 => Bot::Interactive,
        _ => unreachable!(),
    }
}

/// TODO: Move all the simulation stuff in a separate module
fn start_match(mut bots: Vec<Box<dyn SnakeBot>>) {
    let mut game = Game::new();

    for id in (0..bots.len()).rev() {
        let idx = bots.len() - 1;
        game.add_snake(id as SnakeId, bots.swap_remove(idx));
    }

    let results = game
        .continue_simulation_if_known_winner(false)
        .initialize()
        .print()
        .after_each_step(move |board: &GameBoard| board.print())
        .after_each_step(|_| thread::sleep(Duration::from_millis(200)))
        .run_to_end();

    println!("{}", results);
}

fn speed_test() {
    let nb_simulations = Input::with_theme(&*DIALOG_THEME)
        .with_prompt("How many simulations?")
        .default(100_000)
        .interact()
        .unwrap_or(0);

    let nb_bots = Input::with_theme(&*DIALOG_THEME)
        .with_prompt("How many bots?")
        .default(2)
        .interact()
        .unwrap_or(2);

    let which_bot = match Select::with_theme(&*DIALOG_THEME)
        .with_prompt("Which bot?")
        .default(0)
        .item("random bot")
        .item("human-tuned heuristic bot")
        .interact()
        .unwrap_or(0) {
        0 => Bot::Random,
        1 => Bot::Heuristic,
        _ => unreachable!(),
    };

    let print = Input::with_theme(&*DIALOG_THEME)
        .with_prompt("Print?")
        .default(false)
        .interact()
        .unwrap_or(false);

    let continue_if_winner = false;

    use crate::bench_tests::test_simulation_speed;
    match which_bot {
        Bot::Random => {
            test_simulation_speed::<RandomBot>(nb_simulations, nb_bots, continue_if_winner, print);
        }
        Bot::Heuristic => {
            test_simulation_speed::<HeuristicBot>(nb_simulations, nb_bots, continue_if_winner, print);
        }
        _ => unreachable!()
    };
}
