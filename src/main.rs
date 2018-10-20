#![allow(unused_imports)]

extern crate rand;
extern crate genevo;
extern crate colored;
#[macro_use]
extern crate lazy_static;
extern crate rayon;

mod game_engine;
mod bench_tests;
mod random_bot;
mod heuristic_bot;
mod learning;

use random_bot::RandomBot;
use heuristic_bot::HeuristicBot;
use learning::learning;


fn main() {
//    bench_tests::test_random_bot_simulation_speed(100_000, 1, true, false);
//    bench_tests::test_random_bot_simulation_speed(100_000, 2, false,false);
//    bench_tests::test_random_bot_simulation_speed(1, 2, false, true);
//    bench_tests::test_simulation_speed::<RandomBot>(1, 2, false, true);
//    bench_tests::test_simulation_speed::<HeuristicBot>(1, 2, false, true);
//    bench_tests::test_simulation_speed::<HeuristicBot>(2000, 2, false, false);
    learning();
}
