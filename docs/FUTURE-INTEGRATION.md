# Future Integration: ternary-ga

## Current State
Implements genetic algorithms on ternary genomes: `TernaryChromosome` with gene-level access, tournament/roulette/rank selection, one-point/two-point/uniform crossover, random trit-flip mutation, population statistics (diversity, fitness distribution), and generational tracking.

## Integration Opportunities

### With ternary-cell / room-as-codespace
Evolve room configurations. Each `TernaryChromosome` encodes a room's state vector. The fitness function rewards configurations that satisfy room constraints (comfort, energy efficiency, safety). Run the GA across a population of candidate configurations, select the fittest, and deploy the winner to the room. `population_diversity()` monitors whether the search has converged.

### With ternary-rl
GA + RL hybrid: use the GA to discover high-level strategies (room configurations) and RL to refine execution within those strategies. The `TernaryChromosome` defines the RL agent's hyperparameters (exploration rate, learning rate discretized to ternary), and the RL agent's performance is the fitness. Meta-learning via co-evolution.

### With ternary-gradient
`TernaryPoint::neighbors()` from `ternary-gradient` provides the mutation operator: flip a single trit. `coordinate_descent()` provides Lamarckian refinement — after crossover and mutation, apply local search to each offspring. This combines GA's global exploration with gradient's local exploitation.

## Potential in Mature Systems
In PLATO, the fleet runs a continuous GA at Layer 2. New room configurations are generated via `crossover()` of successful parents and `mutate()` for exploration. `tournament_selection()` picks parents proportionally to fitness. The `TernaryChromosome` format enables efficient serialization via `ternary-protocol` for fleet-wide breeding. At Layer 0, the GA compiles to a fixed lookup table — the result of evolution, not evolution itself.

## Cross-Pollination Ideas
**Music × GA:** Evolve ternary melodies. Chromosome = sequence of ternary pitch intervals. Fitness = voice-leading smoothness + harmonic variety + rhythmic interest. Crossover combines motifs from two parent melodies. Mutation explores new intervals. Over generations, the population converges to "beautiful" ternary music. Connects to `ternary-music` and `agent-rhythm-rs`.

**Evolution-ternary × GA:** `evolution-ternary` models biological evolution; `ternary-ga` is computational evolution. The biological models (selection pressure, genetic drift, speciation) could enrich the GA's selection operators. Conversely, the GA's diversity metrics inform biological model calibration.

## Dependencies for Next Steps
- Multi-objective fitness (Pareto optimization with `ternary-pareto`)
- Island model for distributed evolution across construct fleet
- Constraint-preserving crossover (offspring must satisfy room constraints)
