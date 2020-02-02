use std::{
    fs::OpenOptions,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use rand::prelude::*;

use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirmation, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};

use colored::Colorize;

use genevo::{
    genetic::FitnessFunction, operator::prelude::*, population::ValueEncodedGenomeBuilder,
    prelude::*, types::fmt::Display,
};

use rayon::prelude::*;

use chrono::prelude::*;

use crate::game_engine::{GameResultWinner::*, *};
use crate::heuristic_bot::*;
use crate::interactive_bot::InteractiveBot;
use crate::random_bot::RandomBot;
use crate::DIALOG_THEME;

/// The genotype is a vector of coefficients.
pub type GeneticBotGenome = Weights;

pub const GENOME_LENGTH: usize = NB_WEIGHTS;
pub const GENOME_MIN_VALUE: f64 = -1.;
pub const GENOME_MAX_VALUE: f64 = 1.;

#[derive(Debug)]
struct Parameters {
    population_size: usize,
    generation_limit: u64,
    num_individuals_per_parents: usize,
    selection_ratio: f64,
    num_crossover_points: usize,
    mutation_rate: f64,
    mutation_range: f64,
    mutation_precision: u8,
    reinsertion_ratio: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            population_size: 250,
            generation_limit: 10_000,
            num_individuals_per_parents: 2,
            selection_ratio: 0.7,
            num_crossover_points: 1,
            mutation_rate: 0.05,
            mutation_range: 0.1,
            mutation_precision: 2,
            reinsertion_ratio: 0.7,
        }
    }
}

/// The fitness function for `GeneticBotGenome`s.
#[derive(Clone, Debug)]
pub struct WinRatioFitnessCalc {
    target_fitness: usize,
}

impl WinRatioFitnessCalc {
    const NB_MATCHES: usize = 20;

    fn new(target_fitness: usize) -> Self {
        WinRatioFitnessCalc { target_fitness }
    }
}

impl FitnessFunction<GeneticBotGenome, usize> for WinRatioFitnessCalc {
    fn fitness_of(&self, genome: &GeneticBotGenome) -> usize {
        (0..Self::NB_MATCHES as usize)
            .into_par_iter()
            .map(|_| {
                let results = Game::new()
                    .continue_simulation_if_known_winner(false)
                    .add_snake(0, Box::from(HeuristicBot::new(genome)))
                    .add_snake(1, Box::from(HeuristicBot::default()))
                    .initialize()
                    .run_to_end();

                match results.winner {
                    Some(GameResultWinner::Winner(0)) => 2,
                    Some(GameResultWinner::Draw) => 1,
                    _ => 0,
                }
            })
            .sum()
    }

    fn average(&self, fitness_values: &[usize]) -> usize {
        fitness_values.iter().sum::<usize>() / fitness_values.len()
    }

    fn highest_possible_fitness(&self) -> usize {
        self.target_fitness
    }

    fn lowest_possible_fitness(&self) -> usize {
        0
    }
}

#[allow(dead_code)]
pub fn learning() {
    if let Some(learned_weights) = learn_weights() {
        // Ask the user if he/she wants the play against the found genome
        if Confirmation::with_theme(&*DIALOG_THEME)
            .with_text("Do you want to test the found genome?")
            .interact()
            .unwrap_or(false)
        {
            test_weights(learned_weights);
        }
    } else {
        println!("{}", "Learning failed.".red().bold());
    }
}

/// Add a Ctrl+C handler (if the feature is enabled)
///
/// Returns: (handler_enabled, ctrlc_interrupted, learning_stopped)
#[cfg(feature = "ctrlc")]
fn install_ctrlc_handler() -> (Arc<AtomicBool>, Arc<AtomicBool>, Arc<AtomicBool>) {
    let handler_enabled = Arc::new(AtomicBool::new(true));
    let handler_enabled_inner = handler_enabled.clone();
    let ctrlc_interrupted = Arc::new(AtomicBool::new(false));
    let ctrlc_interrupted_inner = ctrlc_interrupted.clone();
    let learning_stopped = Arc::new(AtomicBool::new(false));
    let learning_stopped_inner = learning_stopped.clone();
    ctrlc::set_handler(move || {
        if handler_enabled_inner.load(Ordering::SeqCst) {
            ctrlc_interrupted_inner.store(true, Ordering::SeqCst);

            // Ask the reason of the Ctrl+C
            let ctrlc_choice = Select::with_theme(&*DIALOG_THEME)
                .with_prompt(" Ctrl+C received, what do you want to do?")
                .default(0)
                .item("oops, nothing")
                .item("stop the learning")
                .item("quit")
                .interact()
                .unwrap_or(2);

            // Execute the action
            match ctrlc_choice {
                0 => {}
                1 => {
                    learning_stopped_inner.store(true, Ordering::SeqCst);
                }
                2 => {
                    ::std::process::exit(0);
                }
                _ => unreachable!(),
            }

            ctrlc_interrupted_inner.store(false, Ordering::SeqCst);
        }
    })
    .unwrap_or_else(|_| eprintln!("Error setting Ctrl-C handler."));

    (handler_enabled, ctrlc_interrupted, learning_stopped)
}

/// No-op (if the feature is disabled)
///
/// Returns: (handler_enabled, ctrlc_interrupted, learning_stopped)
#[cfg(not(feature = "ctrlc"))]
fn install_ctrlc_handler() -> (Arc<AtomicBool>, Arc<AtomicBool>, Arc<AtomicBool>) {
    let handler_enabled = Arc::new(AtomicBool::new(false));
    let ctrlc_interrupted = Arc::new(AtomicBool::new(false));
    let learning_stopped = Arc::new(AtomicBool::new(false));

    (handler_enabled, ctrlc_interrupted, learning_stopped)
}

fn learn_weights() -> Option<Weights> {
    let params = Parameters::default();

    // Create the initial population
    let initial_population: Population<GeneticBotGenome> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(
            GENOME_LENGTH,
            GENOME_MIN_VALUE,
            GENOME_MAX_VALUE,
        ))
        .of_size(params.population_size)
        .uniform_at_random();

    // Ask the target fitness
    const DEFAULT_TARGET_FITNESS: usize = (WinRatioFitnessCalc::NB_MATCHES as f32 * 1.8) as usize;
    let target_fitness = Input::with_theme(&*DIALOG_THEME)
        .with_prompt("Target fitness")
        .default(DEFAULT_TARGET_FITNESS)
        .interact()
        .unwrap_or(DEFAULT_TARGET_FITNESS);
    let fitness_calc = WinRatioFitnessCalc::new(target_fitness);

    // Configure the simulation
    let mut snake_simulation = simulate(
        genetic_algorithm()
            .with_evaluation(fitness_calc.clone())
            .with_selection(MaximizeSelector::new(
                params.selection_ratio,
                params.num_individuals_per_parents,
            ))
            .with_crossover(MultiPointCrossBreeder::new(params.num_crossover_points))
            // .with_crossover(DiscreteCrossBreeder::new())
            // .with_mutation(RandomValueMutator::new(
            //     params.mutation_rate,
            //     GENOME_MIN_VALUE,
            //     GENOME_MAX_VALUE,
            // ))
            .with_mutation(BreederValueMutator::new(
                params.mutation_rate,
                params.mutation_range,
                params.mutation_precision,
                GENOME_MIN_VALUE * 10_f64,
                GENOME_MAX_VALUE * 10_f64,
            ))
            .with_reinsertion(ElitistReinserter::new(
                fitness_calc,
                true,
                params.reinsertion_ratio,
            ))
            .with_initial_population(initial_population)
            .build(),
    )
    .until(or(
        FitnessLimit::new(target_fitness),
        GenerationLimit::new(params.generation_limit),
    ))
    .build();

    // The progress bar, to entertain during the learning
    let max_fitness_bar = ProgressBar::new(target_fitness as u64);
    max_fitness_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:0.cyan/blue}] {pos}/{len}\n\n",
            )
            .progress_chars("#>-"),
    );

    // Add a Ctrl+C handler (if the feature is enabled)
    let (handler_enabled, ctrlc_interrupted, learning_stopped) = install_ctrlc_handler();

    // Open a file to dump the stats
    let dt = Local::now();
    let mut stats_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("stats_dump_{}.txt", dt.format("%Y-%m-%d_%H:%M:%S")));
    if let Ok(ref mut file) = stats_file {
        writeln!(file, "[")
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Save failed: {:?}", e));
    } else if let Err(ref e) = stats_file {
        println!("Unable to open a file to dump data: {}.", e);
    }

    // Run the learning
    let mut best_weights = None;
    while !learning_stopped.load(Ordering::SeqCst) {
        let result = snake_simulation.step();
        match result {
            Ok(SimResult::Intermediate(step)) => {
                if !ctrlc_interrupted.load(Ordering::SeqCst) {
                    let evaluated_population = step.result.evaluated_population;
                    let best_solution = step.result.best_solution;
                    println!(
                        "{}\n\
                         --> population_size: {}, average_fitness: {}, best fitness: {}\n\
                         --> duration: {}, processing_time: {}\n\
                         {}\n\n",
                        format!("[Generation {}]", step.iteration).yellow(),
                        evaluated_population.individuals().len(),
                        evaluated_population.average_fitness(),
                        best_solution.solution.fitness,
                        step.duration.fmt(),
                        step.processing_time.fmt(),
                        PrettyWeights(&best_solution.solution.genome)
                    );
                    max_fitness_bar.set_position(best_solution.solution.fitness as u64);

                    if let Ok(ref mut file) = stats_file {
                        let mut line = String::with_capacity(30 + 10 * params.population_size);

                        let g = step.iteration;
                        for f in evaluated_population.fitness_values().iter() {
                            line.push_str(&format!("({},{}),", g, f));
                        }
                        if let Err(e) = writeln!(file, "{}", line) {
                            eprintln!("Couldn't dump stats to file: {}", e);
                        }

                        file.sync_all().unwrap();
                    }

                    if learning_stopped.load(Ordering::SeqCst) {
                        best_weights = Some(best_solution.solution.genome.clone());
                    }
                }
            }
            Ok(SimResult::Final(step, processing_time, duration, stop_reason)) => {
                max_fitness_bar.finish();

                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                println!(
                    "{} Best solution: generation {}\n\
                     --> {}\n\
                     --> population_size: {}, average_fitness: {}, best fitness: {}\n\
                     --> duration: {}, processing_time: {}\n\
                     {}\n\n",
                    format!("[Generation {}]", step.iteration).yellow(),
                    format!("{}", best_solution.generation).yellow(),
                    stop_reason.green(),
                    evaluated_population.individuals().len(),
                    evaluated_population.average_fitness(),
                    best_solution.solution.fitness,
                    duration.fmt(),
                    processing_time.fmt(),
                    PrettyWeights(&best_solution.solution.genome)
                );

                best_weights = Some(best_solution.solution.genome);
                break;
            }
            Err(error) => {
                println!("{:?}", error);
                max_fitness_bar.finish_and_clear();
                break;
            }
        }
    }

    // Disable the Ctrl+C handler
    // TODO: Add a global handler
    handler_enabled.store(false, Ordering::SeqCst);

    // Add the closing bracket to the data (Python format)
    if let Ok(ref mut file) = stats_file {
        writeln!(file, "]")
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Save failed: {:?}", e));
    }

    best_weights
}

fn test_weights(weights: Weights) {
    let mut bot_choice = 0;
    loop {
        // Ask if who should be the player 2
        bot_choice = Select::with_theme(&*DIALOG_THEME)
            .with_prompt("With which bot do you want to test?")
            .default(bot_choice)
            .item("random AI")
            .item("heuristic AI")
            .item("random AI (slow)")
            .item("heuristic AI (slow)")
            .item("myself")
            .item("stop")
            .interact()
            .unwrap_or(5);

        // Create the game
        let mut game = Game::new();
        game.continue_simulation_if_known_winner(false)
            .add_snake(0, Box::from(HeuristicBot::new(&weights)));

        // Add the bot corresponding to the user's choice
        match bot_choice {
            0 | 2 => {
                game.add_snake(1, Box::from(RandomBot::new()));
            }
            1 | 3 => {
                game.add_snake(1, Box::from(HeuristicBot::default()));
            }
            4 => {
                println!(
                    "You play the {} and start toward the {}.",
                    "red snake".red(),
                    "NORTH".yellow()
                );
                game.add_snake(1, Box::from(InteractiveBot {}));
            }
            _ => {
                break;
            }
        }

        // Add sleeps if the user asked for "slow" games
        if bot_choice == 2 || bot_choice == 3 {
            game.after_each_step(|_| thread::sleep(Duration::from_millis(200)));
        }

        // Run the game until its end
        let results = game
            .initialize()
            .print()
            .after_each_step(|board| board.print())
            .run_to_end();

        // Show the results
        print!("\n  => ");
        if let Some(Winner(winner_id)) = results.winner {
            if winner_id == 1 {
                match bot_choice {
                    0 | 2 => println!(
                        "{}",
                        format!("RandomBot won in {} moves!", results.steps).red()
                    ),
                    1 | 3 => println!(
                        "{}",
                        format!("HeuristicBot won in {} moves!", results.steps).red()
                    ),
                    4 => println!("{}", format!("You won in {} moves!", results.steps).green()),
                    _ => unreachable!(),
                }
            } else if bot_choice == 4 {
                // Human player, different message
                println!(
                    "{}",
                    format!("The learned bot beat you in {} moves!", results.steps).red()
                );
            } else {
                println!(
                    "{}",
                    format!("The learned bot won in {} moves!", results.steps).green()
                );
            }
        } else {
            println!(
                "{}",
                format!("It's a draw! ({} moves)", results.steps).yellow()
            );
        }
        println!();

        // Reshow the weights, for convenience
        println!("You played against: {}\n", PrettyWeights(&weights));
    }
}
