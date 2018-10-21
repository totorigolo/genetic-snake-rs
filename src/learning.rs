use rand::prelude::*;

use indicatif::{ProgressBar, ProgressStyle};

use genevo::prelude::*;
use genevo::operator::prelude::*;
use genevo::types::fmt::Display;
use genevo::genetic::FitnessFunction;
use genevo::population::ValueEncodedGenomeBuilder;

use rayon::prelude::*;

use game_engine::*;
use random_bot::RandomBot;
use heuristic_bot::{HeuristicBot, Weights, NB_WEIGHTS};


#[derive(Debug)]
struct Parameters {
    population_size: usize,
    generation_limit: u64,
    num_individuals_per_parents: usize,
    selection_ratio: f64,
    num_crossover_points: usize,
    mutation_rate: f64,
    reinsertion_ratio: f64,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            population_size: 100,
            generation_limit: 2000,
            num_individuals_per_parents: 2,
            selection_ratio: 0.7,
            num_crossover_points: 1,
            mutation_rate: 0.05,
            reinsertion_ratio: 0.7,
        }
    }
}

#[allow(dead_code)]
pub fn learning() {
    let params = Parameters::default();

    let initial_population: Population<GeneticBotGenome> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(
            GENOME_LENGTH,
            GENOME_MIN_VALUE,
            GENOME_MAX_VALUE,
        ))
        .of_size(params.population_size)
        .uniform_at_random();

    let mut snake_simulation = simulate(
        genetic_algorithm()
            .with_evaluation(WinRatioFitnessCalc)
            .with_selection(MaximizeSelector::new(
                params.selection_ratio,
                params.num_individuals_per_parents,
            ))
            .with_crossover(
                MultiPointCrossBreeder::new(params.num_crossover_points))
            .with_mutation(RandomValueMutator::new(
                params.mutation_rate,
                GENOME_MIN_VALUE,
                GENOME_MAX_VALUE,
            ))
            .with_reinsertion(ElitistReinserter::new(
                WinRatioFitnessCalc,
                true,
                params.reinsertion_ratio,
            ))
            .with_initial_population(initial_population)
            .build(),
    ).until(or(
        FitnessLimit::new(WinRatioFitnessCalc.highest_possible_fitness()),
        GenerationLimit::new(params.generation_limit),
    ))
        .build();

    // The progress bar, to entertain during the learning
    let max_fitness_bar = ProgressBar::new(WinRatioFitnessCalc.highest_possible_fitness() as u64);
    max_fitness_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:0.cyan/blue}] {pos}/{len}")
        .progress_chars("#>-"));

    // Run the learning
    loop {
        let result = snake_simulation.step();
        match result {
            Ok(SimResult::Intermediate(step)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                println!(
                    "Step: generation: {}, population_size: {}, average_fitness: {}, \
                     best fitness: {}, duration: {}, processing_time: {}",
                    step.iteration,
                    evaluated_population.individuals().len(),
                    evaluated_population.average_fitness(),
                    best_solution.solution.fitness,
                    step.duration.fmt(),
                    step.processing_time.fmt()
                );

                println!("      {:?}", best_solution.solution.genome);
                //println!("| population: [{}]", evaluated_population.individuals().iter().map(|g| g.as_text())
                //    .collect::<Vec<String>>().join("], ["));
                println!();

                max_fitness_bar.set_position(best_solution.solution.fitness as u64);
            }
            Ok(SimResult::Final(step, processing_time, duration, stop_reason)) => {
                let best_solution = step.result.best_solution;
                println!("{}", stop_reason);
                println!(
                    "Final result after {}: generation: {}, \
                     best solution with fitness {} found in generation {}, processing_time: {}",
                    duration.fmt(),
                    step.iteration,
                    best_solution.solution.fitness,
                    best_solution.generation,
                    processing_time.fmt()
                );
                println!("      {:?}", best_solution.solution.genome);

                max_fitness_bar.finish_with_message("Ideal genome found!");
                break;
            }
            Err(error) => {
                println!("{}", error.display());
                max_fitness_bar.finish_and_clear();
                break;
            }
        }
    }
}

/// The genotype is a vector of coefficients.
pub type GeneticBotGenome = Weights;

pub const GENOME_LENGTH: usize = NB_WEIGHTS;
pub const GENOME_MIN_VALUE: f64 = -1.;
pub const GENOME_MAX_VALUE: f64 = 1.;

/// The fitness function for `GeneticBotGenome`s.
#[derive(Clone)]
pub struct WinRatioFitnessCalc;

impl WinRatioFitnessCalc {
    const NB_MATCHES: usize = 15;
}

impl FitnessFunction<GeneticBotGenome, usize> for WinRatioFitnessCalc {
    fn fitness_of(&self, genome: &GeneticBotGenome) -> usize {
        (0..Self::NB_MATCHES as usize).into_par_iter()
            .map(|_| {
                let results = Game::new()
                    .continue_simulation_if_known_winner(false)
                    .add_snake(0, Box::from(HeuristicBot::new(genome)))
                    .add_snake(1, Box::from(HeuristicBot::default()))
                    .initialize()
                    .run_to_end();
//            println!("{:?}", results);

                match results.winner {
                    Some(GameResultWinner::Winner(0)) => 2,
                    Some(GameResultWinner::Draw) => 1,
                    _ => 0
                }
            })
            .sum()
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
