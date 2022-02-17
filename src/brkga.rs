use rayon::prelude::*;
use rand::prelude::*;
use rand_pcg::{Pcg64, Lcg128Xsl64};
use std::time::Instant;
use crate::candlestick::{Candlestick, load_candlesticks};
use crate::backtest::{Backtest, RunMode};
use crate::backtest::strategy::SingleStrategy;

pub struct BRKGA {
    fraction_top: f32, // amount of individuals considered elite
    fraction_bottom: f32, // amount of mutants to be introduced in each generation
    population_size: usize, // size of the population
    max_iterations: usize, 
    elitism_rate: f32, // percentage chance from a gene to be selected from the elite parent
    rng: Lcg128Xsl64,
    cromossome_size: usize,
    population: Vec<Individual>,
    fitness_executor: FitnessExecutor,
}

trait FitnessFunction {
    fn fitness(&self, cromossome: Vec<f32>) -> f32;
}

pub type BrkgaConfig = (f32, f32, usize, usize, f32);

impl BRKGA {
    pub fn new(seed: u64, cromossome_size: usize, config: BrkgaConfig, fitness_executor: FitnessExecutor) -> Self {
         Self {
            fraction_top: config.0,
            fraction_bottom: config.1,
            population_size: config.2,
            max_iterations: config.3,
            elitism_rate: config.4,
            cromossome_size,
            fitness_executor,
            rng: Pcg64::seed_from_u64(seed),
            population: vec![],
        }
    }

    fn initial_population(&mut self) -> Vec<Individual>{
        (0..self.population_size).map(|_| self.random_individual()).collect()
    }

    pub fn random_individual(&mut self) -> Individual {
        Individual::new((0..self.cromossome_size).map(|_|self.rng.gen_range(0.0..1.0)).collect())
    }

    fn generate_mutants(&mut self) -> Vec<Individual> {
        let amount_mutants = (self.population_size as f32 * self.fraction_bottom) as usize;
        (0..amount_mutants).map(|_|self.random_individual()).collect()
    }

    fn get_elite_population(&self) -> Vec<Individual> {
        let amount_elite = (self.population_size as f32 * self.fraction_top) as usize;
        let elite_start = self.population_size - amount_elite;
        self.population[elite_start..].to_vec()
    }

    fn sort_population(&mut self) {
        self.population.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
    }

    fn calculate_population_fitness(&mut self) {
        self.population.par_iter_mut()
            .filter(|ind| ind.fitness.is_none())
            .for_each(|ind| {
                ind.fitness = Some(self.fitness_executor.calculate_fitness(ind.cromossome.as_slice()));
            });
    }

    fn crossover(&mut self, index_elite_parent: usize, index_non_elite_parent: usize) -> Individual {
        let mut child = Individual::new(Vec::with_capacity(self.cromossome_size));
        for i in 0..self.cromossome_size {
            if self.rng.gen_range(0.0..1.0) < self.elitism_rate {
                child.cromossome.push(self.population[index_elite_parent].cromossome[i]);
            } else {
                child.cromossome.push(self.population[index_non_elite_parent].cromossome[i]);
            }
        }
        child
    }

    fn evolve_population(&mut self) -> Vec<Individual>{
        let mut new_population: Vec<Individual> = Vec::with_capacity(self.population_size);
        new_population.extend(self.generate_mutants());
        new_population.extend(self.get_elite_population());
        
        let amount_to_reproduce = self.population_size - new_population.len();
        for _ in 0..amount_to_reproduce {
            let index_parent_1 = self.get_random_elite();
            let index_parent_2 = self.get_random_non_elite();
            let child = self.crossover(index_parent_1, index_parent_2);
            new_population.push(child);
        }

        new_population
    }

    // returns a random elite individual from the population
    fn get_random_elite(&mut self) -> usize {
        let amount_elite = (self.population_size as f32 * self.fraction_top) as usize;
        let elite_start = self.population_size - amount_elite;
        self.rng.gen_range(elite_start..self.population_size)
    }

    // returns a the index of a random not elite individual from the population
    fn get_random_non_elite(&mut self) -> usize {
        let amount_elite = (self.population_size as f32 * self.fraction_top) as usize;
        let elite_start = self.population_size - amount_elite;
        self.rng.gen_range(0..elite_start)
    }

    pub fn run(&mut self) {
        println!("Starting BRKGA with a population of {}", self.population_size);
        let start = Instant::now();
        self.population = self.initial_population();

        for i in 0..self.max_iterations {
            self.calculate_population_fitness();
            self.sort_population();
            self.show_details(i);
            self.population = self.evolve_population();
        }

        let duration = start.elapsed();
        println!("Time elapsed is: {:?}", duration);

        self.fitness_executor.mode = RunMode::Validation;
        
    }

    fn show_details(&self, generation: usize) {
        let best = &self.population[self.population_size-1];
        let median = &self.population[self.population_size/2];
        let worst = &self.population[0];
        print!("Generation {}: ", generation);
        print!("best fitness: {} | ", best.fitness.unwrap());
        print!("median fitness: {} | ", median.fitness.unwrap());
        print!("worst fitness: {} | ", worst.fitness.unwrap());

        
        println!("validation fitness best of gen {} is {}", generation, 
            self.fitness_executor.validation_fitness(best.cromossome.as_slice()));

    }
}

pub struct Individual {
    pub fitness: Option<f32>,
    pub cromossome: Vec<f32>,
}

impl Individual {
    pub fn new(cromossome: Vec<f32>) -> Self {
        Self {
            fitness: None,
            cromossome,
        }
    }
}

impl Clone for Individual {
    fn clone(&self) -> Individual {
        Individual {
            fitness: self.fitness,
            cromossome: self.cromossome.clone(),
        }
    }
}

pub struct FitnessExecutor {
    backtester: Backtest,
    mode: RunMode,
}


impl FitnessExecutor {
    pub fn new(backtester: Backtest, mode: RunMode) -> Self {
        Self {
            backtester,
            mode,
        }
    }

    pub fn calculate_fitness(&self, cromossome: &[f32]) -> f32 {
        let mut trading_model = SingleStrategy::decode(cromossome);
        self.backtester.run(self.mode, &mut trading_model)
    }

    pub fn validation_fitness(&self, cromossome: &[f32]) -> f32 {
        let mut trading_model = SingleStrategy::decode(cromossome);
        self.backtester.run(RunMode::Validation, &mut trading_model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_brkga(){
        let config: BrkgaConfig = (0.1, 0.2, 15000, 1000, 0.6);
        let candles = load_candlesticks("test_files/ADAUSDT-30m.csv").unwrap();
        let backtest_engine = Backtest::new(candles, 12, 0.005, 0.02);
        let brkga = BRKGA::new(1223,36, config, FitnessExecutor::new(backtest_engine, RunMode::Training));
        
        assert_eq!(brkga.cromossome_size, 36);
        assert_eq!(brkga.fraction_top, 0.1);
        assert_eq!(brkga.fraction_bottom, 0.2);
        assert_eq!(brkga.population_size, 15000);
        assert_eq!(brkga.max_iterations, 1000);
        assert_eq!(brkga.elitism_rate, 0.6);
    }

    #[test]
    fn test_generate_random_invidiual() {
        let config: BrkgaConfig = (0.1, 0.2, 15000, 1000, 0.6);
        let candles = load_candlesticks("test_files/ADAUSDT-30m.csv").unwrap();
        let backtest_engine = Backtest::new(candles, 12, 0.005, 0.02);
        let mut brkga = BRKGA::new(59841, 36, config, FitnessExecutor::new(backtest_engine, RunMode::Training));
        let first_indivual = brkga.random_individual();

        assert_eq!(first_indivual.cromossome.len(), 36);

        for i in 0..first_indivual.cromossome.len() {
            assert!(first_indivual.cromossome[i] >= 0.0 && first_indivual.cromossome[i] <= 1.0);
        }       
        
        assert_eq!(0.66094065, first_indivual.cromossome[0]);
        assert_eq!(0.24245393, first_indivual.cromossome[35]);

        let second_indivual = brkga.random_individual();

        assert_ne!(first_indivual.cromossome, second_indivual.cromossome);
    }
}


