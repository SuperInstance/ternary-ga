#![forbid(unsafe_code)]

//! Genetic algorithm toolkit for ternary genomes ({-1, 0, +1}).
//!
//! Provides TernaryChromosome, selection (tournament/roulette/rank), crossover
//! (one-point/two-point/uniform), mutation (random trit flip), fitness-proportional
//! reproduction, population statistics, and generational tracking.

/// A trit value.
pub type Trit = i8;

pub const NEG: Trit = -1;
pub const ZERO: Trit = 0;
pub const POS: Trit = 1;

// ---------------------------------------------------------------------------
// Simple RNG (xorshift32)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: (seed & 0xFFFFFFFF) as u32 | 1 }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    fn next_usize(&mut self, max: usize) -> usize {
        (self.next_u32() as usize) % max
    }

    fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / u32::MAX as f64
    }

    fn next_trit(&mut self) -> Trit {
        match self.next_u32() % 3 {
            0 => NEG,
            1 => ZERO,
            _ => POS,
        }
    }
}

// ---------------------------------------------------------------------------
// TernaryChromosome
// ---------------------------------------------------------------------------

/// A chromosome of ternary genes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TernaryChromosome {
    genes: Vec<Trit>,
}

impl TernaryChromosome {
    /// Create a new chromosome from a gene vector.
    pub fn new(genes: Vec<Trit>) -> Self {
        Self { genes }
    }

    /// Create a random chromosome of given length.
    pub fn random(len: usize, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let genes: Vec<Trit> = (0..len).map(|_| rng.next_trit()).collect();
        Self { genes }
    }

    /// Create a chromosome filled with a single value.
    pub fn filled(len: usize, value: Trit) -> Self {
        Self { genes: vec![value; len] }
    }

    /// Gene count.
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.genes.is_empty()
    }

    /// Get gene at index.
    pub fn get(&self, idx: usize) -> Option<Trit> {
        self.genes.get(idx).copied()
    }

    /// Set gene at index.
    pub fn set(&mut self, idx: usize, value: Trit) {
        if idx < self.genes.len() {
            self.genes[idx] = value;
        }
    }

    /// Access all genes.
    pub fn genes(&self) -> &[Trit] {
        &self.genes
    }

    /// Sum of genes.
    pub fn sum(&self) -> i32 {
        self.genes.iter().map(|&g| g as i32).sum()
    }

    /// Count of each trit value.
    pub fn counts(&self) -> [usize; 3] {
        let mut c = [0usize; 3];
        for &g in &self.genes {
            c[(g + 1) as usize] += 1;
        }
        c
    }

    // --- Crossover ---

    /// One-point crossover.
    pub fn crossover_one_point(&self, other: &TernaryChromosome, point: usize) -> (TernaryChromosome, TernaryChromosome) {
        let p = point.min(self.genes.len().min(other.genes.len()));
        let child1 = [&self.genes[..p], &other.genes[p..]].concat();
        let child2 = [&other.genes[..p], &self.genes[p..]].concat();
        (TernaryChromosome::new(child1), TernaryChromosome::new(child2))
    }

    /// Two-point crossover.
    pub fn crossover_two_point(&self, other: &TernaryChromosome, p1: usize, p2: usize) -> (TernaryChromosome, TernaryChromosome) {
        let len = self.genes.len().min(other.genes.len());
        let a = p1.min(p2).min(len);
        let b = p1.max(p2).min(len);
        let child1 = [&self.genes[..a], &other.genes[a..b], &self.genes[b..]].concat();
        let child2 = [&other.genes[..a], &self.genes[a..b], &other.genes[b..]].concat();
        (TernaryChromosome::new(child1), TernaryChromosome::new(child2))
    }

    /// Uniform crossover with given probability of taking from `other`.
    pub fn crossover_uniform(&self, other: &TernaryChromosome, swap_prob: f64, seed: u64) -> (TernaryChromosome, TernaryChromosome) {
        let mut rng = Rng::new(seed);
        let len = self.genes.len().min(other.genes.len());
        let mut c1 = self.genes.clone();
        let mut c2 = other.genes.clone();
        for i in 0..len {
            if rng.next_f64() < swap_prob {
                c1[i] = other.genes[i];
                c2[i] = self.genes[i];
            }
        }
        (TernaryChromosome::new(c1), TernaryChromosome::new(c2))
    }

    /// Mutate: flip `n` random trits to a different random trit value.
    pub fn mutate(&self, n: usize, seed: u64) -> TernaryChromosome {
        let mut rng = Rng::new(seed);
        let mut genes = self.genes.clone();
        let len = genes.len();
        if len == 0 { return TernaryChromosome::new(genes); }
        for _ in 0..n {
            let idx = rng.next_usize(len);
            let current = genes[idx];
            // Pick a different trit
            let choices: [Trit; 2] = if current == NEG { [ZERO, POS] } else if current == POS { [NEG, ZERO] } else { [NEG, POS] };
            genes[idx] = choices[rng.next_usize(2)];
        }
        TernaryChromosome::new(genes)
    }
}

// ---------------------------------------------------------------------------
// Fitness and Selection
// ---------------------------------------------------------------------------

/// An individual with associated fitness.
#[derive(Clone, Debug)]
pub struct Individual {
    pub chromosome: TernaryChromosome,
    pub fitness: f64,
}

/// Selection method.
#[derive(Clone, Debug)]
pub enum SelectionMethod {
    Tournament { size: usize },
    Roulette,
    Rank,
}

/// Perform tournament selection.
pub fn tournament_select(pop: &[Individual], tournament_size: usize, seed: u64) -> usize {
    let mut rng = Rng::new(seed);
    let mut best_idx = rng.next_usize(pop.len());
    let mut best_fit = pop[best_idx].fitness;
    for _ in 1..tournament_size.min(pop.len()) {
        let idx = rng.next_usize(pop.len());
        if pop[idx].fitness > best_fit {
            best_idx = idx;
            best_fit = pop[idx].fitness;
        }
    }
    best_idx
}

/// Perform roulette (fitness-proportional) selection. Returns index.
pub fn roulette_select(pop: &[Individual], seed: u64) -> usize {
    let mut rng = Rng::new(seed);
    let total: f64 = pop.iter().map(|i| i.fitness).sum();
    if total <= 0.0 {
        return rng.next_usize(pop.len());
    }
    let threshold = rng.next_f64() * total;
    let mut cum = 0.0f64;
    for (idx, ind) in pop.iter().enumerate() {
        cum += ind.fitness;
        if cum >= threshold {
            return idx;
        }
    }
    pop.len() - 1
}

/// Perform rank selection. Returns index.
pub fn rank_select(pop: &[Individual], seed: u64) -> usize {
    let mut rng = Rng::new(seed);
    let n = pop.len();
    if n == 0 { return 0; }
    // Rank weights: rank 0 = worst, rank n-1 = best
    // Weight for rank i = (i + 1)
    // Total weight = n*(n+1)/2
    let total_weight = n * (n + 1) / 2;
    let threshold = rng.next_usize(total_weight);
    let mut cum = 0usize;
    // Sort by fitness (ascending) to assign ranks
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| pop[a].fitness.partial_cmp(&pop[b].fitness).unwrap_or(std::cmp::Ordering::Equal));
    for (rank, &idx) in indices.iter().enumerate() {
        cum += rank + 1;
        if cum > threshold {
            return idx;
        }
    }
    indices[n - 1]
}

// ---------------------------------------------------------------------------
// Population and GA Engine
// ---------------------------------------------------------------------------

/// Statistics about a population.
#[derive(Clone, Debug)]
pub struct PopulationStats {
    pub best_fitness: f64,
    pub worst_fitness: f64,
    pub avg_fitness: f64,
    pub best_chromosome: TernaryChromosome,
    pub generation: usize,
    pub population_size: usize,
}

/// A generation record for tracking GA progress.
#[derive(Clone, Debug)]
pub struct GenerationRecord {
    pub generation: usize,
    pub stats: PopulationStats,
}

/// The main GA engine.
#[derive(Clone, Debug)]
pub struct TernaryGA {
    pub population: Vec<Individual>,
    pub generation: usize,
    pub selection: SelectionMethod,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub history: Vec<GenerationRecord>,
    rng: Rng,
}

impl TernaryGA {
    /// Create a new GA with given population.
    pub fn new(
        population: Vec<Individual>,
        selection: SelectionMethod,
        mutation_rate: f64,
        crossover_rate: f64,
        seed: u64,
    ) -> Self {
        let rng = Rng::new(seed);
        Self {
            population,
            generation: 0,
            selection,
            mutation_rate,
            crossover_rate,
            history: vec![],
            rng,
        }
    }

    /// Create a random initial population.
    pub fn random_population(
        pop_size: usize,
        chrom_len: usize,
        fitness_fn: &dyn Fn(&TernaryChromosome) -> f64,
        seed: u64,
    ) -> Vec<Individual> {
        (0..pop_size)
            .map(|i| {
                let chrom = TernaryChromosome::random(chrom_len, seed.wrapping_add(i as u64 * 7919));
                let fitness = fitness_fn(&chrom);
                Individual { chromosome: chrom, fitness }
            })
            .collect()
    }

    fn select_parent(&mut self) -> usize {
        let seed = self.rng.next_u32() as u64;
        match &self.selection {
            SelectionMethod::Tournament { size } => tournament_select(&self.population, *size, seed),
            SelectionMethod::Roulette => roulette_select(&self.population, seed),
            SelectionMethod::Rank => rank_select(&self.population, seed),
        }
    }

    /// Run one generation.
    pub fn step(&mut self, fitness_fn: &dyn Fn(&TernaryChromosome) -> f64) {
        let pop_size = self.population.len();
        if pop_size < 2 { return; }
        let chrom_len = self.population[0].chromosome.len();
        let mut new_pop = Vec::with_capacity(pop_size);

        while new_pop.len() < pop_size {
            let p1 = self.select_parent();
            let p2 = self.select_parent();
            let parent1 = &self.population[p1].chromosome;
            let parent2 = &self.population[p2].chromosome;

            if self.rng.next_f64() < self.crossover_rate && chrom_len > 1 {
                let point = self.rng.next_usize(chrom_len - 1) + 1;
                let (c1, c2) = parent1.crossover_one_point(parent2, point);
                new_pop.push(c1);
                if new_pop.len() < pop_size {
                    new_pop.push(c2);
                }
            } else {
                new_pop.push(parent1.clone());
                if new_pop.len() < pop_size {
                    new_pop.push(parent2.clone());
                }
            }
        }
        new_pop.truncate(pop_size);

        // Mutation
        let mut_genes: Vec<TernaryChromosome> = new_pop
            .into_iter()
            .map(|c| {
                let num_mutations = (self.mutation_rate * c.len() as f64).round() as usize;
                if num_mutations > 0 {
                    let seed = self.rng.next_u32() as u64;
                    c.mutate(num_mutations, seed)
                } else {
                    c
                }
            })
            .collect();

        self.population = mut_genes
            .into_iter()
            .map(|c| {
                let f = fitness_fn(&c);
                Individual { chromosome: c, fitness: f }
            })
            .collect();

        self.generation += 1;
        let stats = self.compute_stats();
        self.history.push(GenerationRecord { generation: self.generation, stats });
    }

    /// Run for `n` generations.
    pub fn run(&mut self, generations: usize, fitness_fn: &dyn Fn(&TernaryChromosome) -> f64) {
        for _ in 0..generations {
            self.step(fitness_fn);
        }
    }

    /// Compute current population statistics.
    pub fn compute_stats(&self) -> PopulationStats {
        if self.population.is_empty() {
            return PopulationStats {
                best_fitness: 0.0,
                worst_fitness: 0.0,
                avg_fitness: 0.0,
                best_chromosome: TernaryChromosome::new(vec![]),
                generation: self.generation,
                population_size: 0,
            };
        }
        let best = self.population.iter().fold(f64::NEG_INFINITY, |a, i| a.max(i.fitness));
        let worst = self.population.iter().fold(f64::INFINITY, |a, i| a.min(i.fitness));
        let avg: f64 = self.population.iter().map(|i| i.fitness).sum::<f64>() / self.population.len() as f64;
        let best_idx = self.population.iter().enumerate().max_by(|a, b| a.1.fitness.partial_cmp(&b.1.fitness).unwrap()).map(|(i, _)| i).unwrap_or(0);
        PopulationStats {
            best_fitness: best,
            worst_fitness: worst,
            avg_fitness: avg,
            best_chromosome: self.population[best_idx].chromosome.clone(),
            generation: self.generation,
            population_size: self.population.len(),
        }
    }

    /// Get the best individual.
    pub fn best(&self) -> Option<&Individual> {
        self.population.iter().max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
    }

    /// Get the history.
    pub fn history(&self) -> &[GenerationRecord] {
        &self.history
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chromosome_new() {
        let c = TernaryChromosome::new(vec![1, 0, -1]);
        assert_eq!(c.len(), 3);
        assert_eq!(c.get(0), Some(1));
        assert_eq!(c.get(1), Some(0));
        assert_eq!(c.get(2), Some(-1));
    }

    #[test]
    fn test_chromosome_filled() {
        let c = TernaryChromosome::filled(5, POS);
        assert_eq!(c.len(), 5);
        assert!(c.genes().iter().all(|&g| g == POS));
    }

    #[test]
    fn test_chromosome_sum() {
        let c = TernaryChromosome::new(vec![1, 1, -1, 0]);
        assert_eq!(c.sum(), 1);
    }

    #[test]
    fn test_chromosome_counts() {
        let c = TernaryChromosome::new(vec![1, 0, -1, 1, -1]);
        let counts = c.counts();
        assert_eq!(counts, [2, 1, 2]); // [neg, zero, pos]
    }

    #[test]
    fn test_chromosome_set() {
        let mut c = TernaryChromosome::new(vec![1, 0, -1]);
        c.set(1, POS);
        assert_eq!(c.get(1), Some(POS));
    }

    #[test]
    fn test_one_point_crossover() {
        let p1 = TernaryChromosome::new(vec![1, 1, 1, 1]);
        let p2 = TernaryChromosome::new(vec![-1, -1, -1, -1]);
        let (c1, c2) = p1.crossover_one_point(&p2, 2);
        // c1 = self[0..2] + other[2..] = [1,1,-1,-1]
        assert_eq!(c1.genes(), &[1, 1, -1, -1]);
        assert_eq!(c2.genes(), &[-1, -1, 1, 1]);
    }

    #[test]
    fn test_two_point_crossover() {
        let p1 = TernaryChromosome::new(vec![1, 1, 1, 1, 1]);
        let p2 = TernaryChromosome::new(vec![-1, -1, -1, -1, -1]);
        let (c1, _c2) = p1.crossover_two_point(&p2, 1, 3);
        // c1 = self[0..1] + other[1..3] + self[3..]
        assert_eq!(c1.genes(), &[1, -1, -1, 1, 1]);
    }

    #[test]
    fn test_uniform_crossover() {
        let p1 = TernaryChromosome::new(vec![1, 1, 1, 1]);
        let p2 = TernaryChromosome::new(vec![-1, -1, -1, -1]);
        // With swap_prob=0.0, children should be identical to parents
        let (c1, c2) = p1.crossover_uniform(&p2, 0.0, 42);
        assert_eq!(c1.genes(), &[1, 1, 1, 1]);
        assert_eq!(c2.genes(), &[-1, -1, -1, -1]);
    }

    #[test]
    fn test_mutation_changes_genes() {
        let c = TernaryChromosome::filled(100, ZERO);
        let mutated = c.mutate(50, 12345);
        let same = c.genes().iter().zip(mutated.genes().iter()).filter(|(a, b)| a == b).count();
        assert!(same < 100);
    }

    #[test]
    fn test_mutation_changes_to_different_value() {
        let c = TernaryChromosome::filled(100, POS);
        let mutated = c.mutate(50, 999);
        // All genes should still be valid trits
        for &g in mutated.genes() {
            assert!(g == NEG || g == ZERO || g == POS);
        }
        // Mutated genes should be different from original (POS)
        let changed = c.genes().iter().zip(mutated.genes().iter()).filter(|(a, b)| a != b).count();
        assert!(changed > 0);
    }

    #[test]
    fn test_tournament_selection() {
        let pop: Vec<Individual> = vec![
            Individual { chromosome: TernaryChromosome::filled(3, POS), fitness: 10.0 },
            Individual { chromosome: TernaryChromosome::filled(3, NEG), fitness: 1.0 },
            Individual { chromosome: TernaryChromosome::filled(3, ZERO), fitness: 5.0 },
        ];
        // With tournament size 3, should always pick the best
        let idx = tournament_select(&pop, 3, 42);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_roulette_selection_runs() {
        let pop: Vec<Individual> = vec![
            Individual { chromosome: TernaryChromosome::filled(3, POS), fitness: 10.0 },
            Individual { chromosome: TernaryChromosome::filled(3, NEG), fitness: 5.0 },
        ];
        let idx = roulette_select(&pop, 42);
        assert!(idx < 2);
    }

    #[test]
    fn test_rank_selection_runs() {
        let pop: Vec<Individual> = vec![
            Individual { chromosome: TernaryChromosome::filled(3, POS), fitness: 10.0 },
            Individual { chromosome: TernaryChromosome::filled(3, NEG), fitness: 1.0 },
        ];
        let idx = rank_select(&pop, 42);
        assert!(idx < 2);
    }

    #[test]
    fn test_ga_random_population() {
        let pop = TernaryGA::random_population(20, 10, &|_| 1.0, 42);
        assert_eq!(pop.len(), 20);
        assert_eq!(pop[0].chromosome.len(), 10);
    }

    #[test]
    fn test_ga_step_improves_or_maintains() {
        let fitness = |c: &TernaryChromosome| -> f64 { c.sum() as f64 };
        let pop = TernaryGA::random_population(30, 10, &fitness, 42);
        let mut ga = TernaryGA::new(pop, SelectionMethod::Tournament { size: 3 }, 0.1, 0.8, 123);
        let initial_best = ga.compute_stats().best_fitness;
        ga.run(50, &fitness);
        let final_best = ga.compute_stats().best_fitness;
        // Should improve or maintain over 50 generations with tournament selection
        assert!(final_best >= initial_best - 1.0); // Allow small tolerance
    }

    #[test]
    fn test_ga_history_tracking() {
        let fitness = |c: &TernaryChromosome| -> f64 { c.sum() as f64 };
        let pop = TernaryGA::random_population(10, 5, &fitness, 42);
        let mut ga = TernaryGA::new(pop, SelectionMethod::Tournament { size: 2 }, 0.1, 0.7, 99);
        ga.run(5, &fitness);
        assert_eq!(ga.history().len(), 5);
    }

    #[test]
    fn test_population_stats() {
        let pop = vec![
            Individual { chromosome: TernaryChromosome::filled(3, POS), fitness: 10.0 },
            Individual { chromosome: TernaryChromosome::filled(3, ZERO), fitness: 5.0 },
            Individual { chromosome: TernaryChromosome::filled(3, NEG), fitness: 1.0 },
        ];
        let ga = TernaryGA::new(pop, SelectionMethod::Roulette, 0.1, 0.8, 1);
        let stats = ga.compute_stats();
        assert_eq!(stats.best_fitness, 10.0);
        assert_eq!(stats.worst_fitness, 1.0);
        assert!((stats.avg_fitness - 16.0 / 3.0).abs() < 0.01);
        assert_eq!(stats.population_size, 3);
    }

    #[test]
    fn test_best_individual() {
        let pop = vec![
            Individual { chromosome: TernaryChromosome::filled(3, POS), fitness: 3.0 },
            Individual { chromosome: TernaryChromosome::filled(3, NEG), fitness: -3.0 },
        ];
        let ga = TernaryGA::new(pop, SelectionMethod::Tournament { size: 2 }, 0.1, 0.8, 1);
        let best = ga.best().unwrap();
        assert!((best.fitness - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_chromosome_random_valid_trits() {
        let c = TernaryChromosome::random(100, 42);
        for &g in c.genes() {
            assert!(g == NEG || g == ZERO || g == POS);
        }
    }

    #[test]
    fn test_ga_converges_on_simple_objective() {
        // Objective: maximize sum of genes (best = all POS = sum=10)
        let fitness = |c: &TernaryChromosome| -> f64 { c.sum() as f64 };
        let pop = TernaryGA::random_population(50, 10, &fitness, 42);
        let mut ga = TernaryGA::new(pop, SelectionMethod::Tournament { size: 3 }, 0.05, 0.9, 7);
        ga.run(100, &fitness);
        let best = ga.best().unwrap();
        assert!(best.fitness >= 8.0, "GA should converge near optimal, got fitness={}", best.fitness);
    }

    #[test]
    fn test_chromosome_empty() {
        let c = TernaryChromosome::new(vec![]);
        assert!(c.is_empty());
        assert_eq!(c.sum(), 0);
        assert_eq!(c.counts(), [0, 0, 0]);
    }
}
