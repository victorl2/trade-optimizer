use std::thread;

pub struct BRKGA {
    population: Vec<Individual>,
    current_generation: u32,
    max_generations: u32,
    elite_rate: f32, // percentage of the population that will be the elite [ 0 - 100% ]
    mutant_rate: f32, // percentage of the population that will be mutants ( completely random cromossomes ) [ 0 - 100% ]
    elite_selection_rate: f32, // percentage rate that a elite individual gene will be selected when reproducing ( from 0 to 1.0 )
    cromossome_size: u32,
}

struct Individual {
    fitness: f32,
    cromossome: Vec<f32>,
}

impl Individual {
    // Create a new individual with random values for cromossome ( each position has a value between 0 and 1 )
    pub fn new(cromossome_size: u32) -> Individual {
        let mut cromossome = Vec::new();
        for _ in 0..cromossome_size {
            cromossome.push(rand::random::<f32>());
        }
        Individual {
            fitness: 0.0,
            cromossome: cromossome,
        }
    }

    pub fn calculate_fitness(&mut self) {
        self.fitness = 0.0;
        for i in 0..self.cromossome.len() {
            self.fitness += self.cromossome[i];
        }
    }

    pub fn crossover(&self, elite_parent: &Individual) -> Individual {
        // choose elite_parent gene with rate elite_selection_rate
        
    }
}

impl BRKGA {
    pub fn new(population_size: u32, max_generations: u32, elite_rate: f32, mutant_rate: f32, elite_selection_rate: f32) -> BRKGA {
        let mut population: Vec<Individual> = Vec::new();
        for _ in 0..population_size {
            let mut cromossome: Vec<f32> = Vec::new();
            for _ in 0..10 {
                cromossome.push(rand::random::<f32>());
            }
            population.push(Individual {
                fitness: 0.0,
                cromossome: cromossome,
            });
        }
        BRKGA {
            population: population,
            current_generation: 0,
            max_generations: max_generations,
            elite_rate: elite_rate,
            mutant_rate: mutant_rate,
            elite_selection_rate: elite_selection_rate,
            cromossome_size: 10,
        }
    }

    // Substitute the bottom of the population with the new mutant individuals ( completely random cromossomes )
    fn generate_mutants(&mut self) {
        let upper_bound = (self.population.len() as f32 * self.mutant_rate) as u32;
        for i  in 0..upper_bound {
            self.population[i as usize] = Individual::new(self.cromossome_size);
        }
    }

    fn initial_population(&mut self) {
        for i in 0..self.population.len() {
            self.population.push(Individual::new(self.cromossome_size));
        }
    }

    // Calculate the fitness of each individual ( not already calculated ) from the population in parallel
    fn calculate_population_fitness(&mut self) {
        let upper_bound = if self.current_generation > 0 { (self.population.len() as f32 * self.mutant_rate) as usize } else { self.population.len() as usize };
        let mut fitness_threads = Vec::new();
        for i in 0..upper_bound  {
            let individual = self.population[i];
            let thread = thread::spawn(move || {
                individual.calculate_fitness();
            });
            fitness_threads.push(thread);
        }
        for thread in fitness_threads {
            thread.join().unwrap();
        }
    }

    // Sort the population by fitness ( from the worst to the best  )
    fn sort_population(&mut self) {
        self.population.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
    }

    fn children_new_population(&mut self) {
        let mut new_population = Vec::new();
        let upper_bound = self.population.len() - ((self.population.len() as f32 * self.elite_rate) as usize); // index from the first elite
        let lower_bound = (self.population.len() as f32 * self.mutant_rate) as usize; // index from the last mutant

        for i in lower_bound..upper_bound {
            // Select a elite individual randomly from the population
            let elite_index = rand::random::<u32>() % (self.population.len() as f32 * self.elite_rate) as u32;
        }
        self.population = new_population;
    }

    pub fn run(&mut self) {
        self.initial_population();
        
        for _ in 0..self.max_generations {
            self.calculate_population_fitness();
            self.sort_population();
        }
    }

}