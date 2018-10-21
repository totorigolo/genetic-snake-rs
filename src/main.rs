#![allow(unused_imports)]

#[macro_use]
extern crate lazy_static;

use console::Style;
use dialoguer::theme::ColorfulTheme;

mod bench_tests;
mod game_engine;
mod heuristic_bot;
mod interactive_bot;
mod learning;
mod random_bot;

use crate::heuristic_bot::HeuristicBot;
use crate::interactive_bot::InteractiveBot;
use crate::learning::learning;
use crate::random_bot::RandomBot;

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
    // bench_tests::test_random_bot_simulation_speed(100_000, 1, true, false);
    // bench_tests::test_random_bot_simulation_speed(100_000, 2, false,false);
    // bench_tests::test_random_bot_simulation_speed(1, 2, false, true);
    // bench_tests::test_simulation_speed::<RandomBot>(1, 2, false, true);
    // bench_tests::test_simulation_speed::<HeuristicBot>(1, 2, false, true);
    // bench_tests::test_simulation_speed::<InteractiveBot>(1, 1, false, true);
    // bench_tests::test_simulation_speed::<HeuristicBot>(2000, 2, false, false);
    learning();
}
