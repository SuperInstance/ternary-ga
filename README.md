# ternary-ga

Genetic algorithm toolkit for ternary genomes — selection, crossover, mutation, and generational tracking optimized for `{-1, 0, +1}` search spaces.

## Why This Exists

Standard genetic algorithm libraries assume real-valued or binary genomes. Ternary search spaces (three values per gene) appear naturally in trinary logic design, quantized neural architecture search, and combinatorial optimization problems. This crate provides a complete GA engine with chromosome operations, multiple selection strategies, and population statistics — all built around the ternary alphabet.

## Core Concepts

- **TernaryChromosome** — A vector of ternary genes with crossover and mutation operations
- **Selection Methods** — Tournament, roulette (fitness-proportional), and rank-based selection
- **Crossover** — One-point, two-point, and uniform crossover with configurable rates
- **Mutation** — Random trit flipping that always changes to a *different* value
- **TernaryGA** — Full GA engine with generational tracking and population statistics

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-ga = "0.1"
```

```rust
use ternary_ga::*;

// Define a fitness function: maximize sum of genes (optimal = all +1)
let fitness = |c: &TernaryChromosome| -> f64 { c.sum() as f64 };

// Create a random initial population
let population = TernaryGA::random_population(50, 10, &fitness, 42);

// Set up the GA engine
let mut ga = TernaryGA::new(
    population,
    SelectionMethod::Tournament { size: 3 },
    0.05,  // mutation rate
    0.9,   // crossover rate
    123,   // RNG seed
);

// Run for 100 generations
ga.run(100, &fitness);

// Check results
let best = ga.best().unwrap();
println!("Best fitness: {:.1}", best.fitness);
println!("Best genes: {:?}", best.chromosome.genes());

// Population statistics
let stats = ga.compute_stats();
println!("Avg fitness: {:.2}", stats.avg_fitness);
println!("Best: {:.1}, Worst: {:.1}", stats.best_fitness, stats.worst_fitness);

// History tracking
for record in ga.history() {
    println!("Gen {}: best={:.1} avg={:.1}",
        record.generation,
        record.stats.best_fitness,
        record.stats.avg_fitness);
}
```

## API Overview

| Type / Function | Description |
|---|---|
| `TernaryChromosome` | Genome with `genes()`, `sum()`, `counts()`, `get()`, `set()` |
| `TernaryChromosome::random` | Random ternary genome |
| `crossover_one_point` / `crossover_two_point` / `crossover_uniform` | Recombination operators |
| `mutate` | Flip n random genes to a different value |
| `SelectionMethod` | `Tournament`, `Roulette`, `Rank` |
| `TernaryGA::new` | Create engine with population and parameters |
| `TernaryGA::run` | Evolve for n generations |
| `TernaryGA::best` / `compute_stats` | Query results |
| `PopulationStats` | Best/worst/avg fitness, best chromosome, generation |

## How It Works

The GA operates in generational cycles:

1. **Selection**: Parents are chosen based on the configured strategy. Tournament selection picks the best of a random subset. Roulette selects proportionally to fitness. Rank selection uses fitness ordering to reduce selection pressure.

2. **Crossover**: With probability `crossover_rate`, two parents produce two children via recombination. One-point crossover splits at a random locus. Two-point uses two random loci. Uniform swaps individual genes with configurable probability.

3. **Mutation**: Each offspring has `mutation_rate × chromosome_length` genes randomly flipped. Crucially, mutation always changes to a *different* trit value — never a no-op.

4. **Evaluation**: Each new individual's fitness is computed, and statistics are recorded.

## Use Cases

1. **Neural architecture search** — Encode architectural choices as ternary genes (expand/keep/shrink) and evolve high-performing networks
2. **Game AI strategy optimization** — Represent mixed strategies (attack/defend/neutral) as ternary genomes
3. **Combinatorial scheduling** — Encode task assignments (early/on-time/late) and optimize via GA
4. **Circuit design** — Search ternary logic gate configurations for desired truth tables

## Known Limitations

- **Xorshift32 RNG is not cryptographically secure**: The internal RNG uses xorshift32, which has a short period (2³² − 1) and fails modern statistical test suites (TestU01, PractRand). Population diversity degrades for runs exceeding ~100K generations. For research-quality GA work, provide your own RNG via the public API.

- **Roulette selection fails with negative fitness**: `SelectionMethod::Roulette` assumes non-negative fitness values. If the fitness function returns negative values, the proportion computation `fitness / total_fitness` produces nonsensical selection probabilities, potentially panicking on division by a zero or negative total.

- **Elitism is not configurable**: The GA engine does not preserve the best individual across generations by default. A promising solution can be lost during crossover/mutation if no copy survives into the next generation. There is no `elitism_count` parameter.

- **Mutation count is probabilistic, not guaranteed**: The number of mutated genes is drawn from a binomial distribution (`mutation_rate × len` expected). For short chromosomes (length < 20), the actual number of mutations often deviates significantly from the expected rate — sometimes zero mutations occur.

- **No niching or speciation**: All individuals compete in a single panmictic population. Without fitness sharing or speciation, the population tends to converge prematurely on a single peak in multimodal fitness landscapes.

## Ecosystem

Part of the **SuperInstance** ternary computing crate family:

- `ternary-compression-v2` — Multi-algorithm ternary compression
- `ternary-hash` — Hashing and fingerprinting for ternary data
- `ternary-pca` — Principal component analysis on ternary values
- `ternary-matrix` — Compact ternary matrix operations
- `ternary-reservoir` — Echo state networks with ternary nodes
- `ternary-evolution-advanced` — Advanced evolutionary optimization
- `ternary-geometry` — Geometric algorithms in ternary space
- `ternary-causality` — Causal inference for ternary systems
- `ternary-consensus` — Distributed consensus for ternary agents

## License

MIT
