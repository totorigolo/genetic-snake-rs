extern crate rand;
extern crate genevo;
extern crate colored;

mod game_engine;
mod random_agent;
mod genetic_agent;
mod bench_tests;

use genevo::operator::prelude::*;
use genevo::population::ValueEncodedGenomeBuilder;
use genevo::prelude::*;
use genevo::types::fmt::Display;

use genetic_agent::{GeneticAgent, GeneticAgentGenome};
use genetic_agent::WinRatioFitnessCalc;

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

/// The phenotype
#[allow(dead_code)]
type Phenotype = GeneticAgent;

/// The genotype
type Genotype = GeneticAgentGenome;


fn main() {
//    bench_tests::test_random_agent_simulation_speed(100_000, 1, 30, 6, true, false);
//    bench_tests::test_random_agent_simulation_speed(100_000, 2, 30, 6, false,false);
//    bench_tests::test_random_agent_simulation_speed(1, 2, 30, 10, false, true);
    learning();
}

fn learning() {
    let params = Parameters::default();

    let initial_population: Population<Genotype> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(
            genetic_agent::GENETIC_AGENT_GENOME_LENGTH,
            genetic_agent::GENETIC_AGENT_GENOME_MIN_VALUE,
            genetic_agent::GENETIC_AGENT_GENOME_MAX_VALUE,
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
                genetic_agent::GENETIC_AGENT_GENOME_MIN_VALUE,
                genetic_agent::GENETIC_AGENT_GENOME_MAX_VALUE,
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
            },
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
                break;
            },
            Err(error) => {
                println!("{}", error.display());
                break;
            },
        }
    }
}
