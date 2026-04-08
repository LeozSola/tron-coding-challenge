# Tron Bot Implementation Agent Guide

This document is the working implementation plan for the new `CompetitiveBot` skeleton in this repo.

## Current repo integration

- Bot entry point: `src/players/competitive_bot.rs`
- Module export: `src/players/mod.rs`
- Local runner wiring: `src/main.rs`
- Engine surface used by the bot: `src/engine/*`

The current skeleton already provides:

- a compilable `CompetitiveBot` implementing `Bot`
- deterministic move selection and fallback behavior
- immediate move safety classification
- basic phase detection (`Contact`, `Split`, `Endgame`)
- reusable grid helpers for neighbors / BFS / reachable area / components
- an `OpponentProfile` state object
- hooks for future simultaneous search and ML feature extraction

## Recommended implementation order

### 1. Harden the safety baseline
- [x] Verify the move classifier handles every immediate loss case correctly
- [x] Improve head-to-head logic beyond simple contested-cell detection
- [x] Add explicit “all losing moves” ranking instead of raw direction fallback
- [x] Add regression tests for wall, tail, and simultaneous collision edge cases

**Exit criteria:** bot never panics, always returns a legal `Direction`, and reliably avoids obvious immediate losses.

### 2. Finish the board-analysis layer
- [x] Add `head_positions(...)` helpers for self/opponent lookup in analysis code
- [x] Build a generic BFS/flood-fill helper as the common traversal primitive
- [x] Refactor reachable-area logic to use the generic traversal helper
- [x] Add shortest-path / distance-map helpers for arbitrary valid starting cells
- [x] Add connected-region extraction, not just component counting
- [x] Add Voronoi territory counts from both heads, including contested cells
- [x] Add stronger corridor-width / local narrowness detection beyond raw neighbor count
- [x] Add articulation / choke-point detection scaffolding where lightweight enough
- [x] Add symmetry normalization helpers for future ML feature extraction
- [x] Add focused regression tests for BFS, distance maps, connected regions, Voronoi, and corridor detection

**Exit criteria:** the bot module has a stable internal analysis toolkit instead of ad hoc per-feature code.

### 3. Upgrade the heuristic evaluator
- [x] Replace placeholder scoring with explicit weighted features
- [x] Score reachable space after the move more accurately
- [x] Add self-trap penalties and cut-opponent bonuses
- [x] Tune tie-breaking to stay deterministic while stronger than the example bot
- [x] Revisit edge/wall penalties so wall-hugging is not over-penalized in stable positions
- [x] Add targeted benchmark scenarios and regression checks for wall-hugging exploit lines
- [x] Improve heuristic shape-awareness for edge races, partition quality, and semi-split geometry
- [x] Re-tune Phase 3 weights using the multi-opening benchmark before Phase 4 search/phase work

**Exit criteria:** `HeuristicBot v1` consistently beats trivial and noisy baselines.

### 4. Strengthen phase detection
- [ ] Tighten `Contact` vs `Split` using shared reachable-space analysis
- [ ] Improve `Endgame` detection with empty-cell and corridor metrics
- [ ] Apply phase-specific weight sets in the evaluator

**Exit criteria:** the scoring behavior changes meaningfully by regime.

### 5. Add shallow simultaneous search
- [ ] Generate legal self moves and legal opponent moves
- [ ] Simulate paired moves with `Grid::next_grid(...)`
- [ ] Respect O/X argument ordering when this bot is player X
- [ ] Start with 1-ply search, then grow to 2-ply if runtime allows
- [ ] Add hard safety caps on expansions / runtime

**Exit criteria:** search improves tactical choices without introducing instability or time risk.

### 6. Expand opponent modeling from history
- [ ] Infer opponent move sequence from `grid_history()`
- [ ] Track wall hugging, aggression, corridor tolerance, and symmetry preferences
- [ ] Feed the profile into heuristic scoring and enemy-reply weighting

**Exit criteria:** `OpponentProfile` meaningfully changes how the bot evaluates likely enemy replies.

### 7. Build a stable feature extractor
- [ ] Define a compact scalar feature vector per candidate move
- [ ] Include phase indicators, legal masks, reachable counts, Voronoi counts, and local geometry
- [ ] Keep feature ordering stable for offline training/export

**Exit criteria:** a single `extract_features(...)` path can support both heuristics and offline ML.

### 8. Add offline tooling outside the submitted bot
- [ ] Create self-play data generation scripts / binaries
- [ ] Serialize replays and labeled move scores
- [ ] Build train/validation/test splits

**Exit criteria:** you can generate teacher-labeled training data without bloating the runtime bot.

### 9. Train and distill a compact model
- [ ] Start with linear / tiny MLP models on handcrafted features
- [ ] Compare by tournament win rate, not just training accuracy
- [ ] Export compact weights for embedding back into Rust

**Exit criteria:** the chosen model improves move ranking while staying tiny and deterministic.

### 10. Integrate the hybrid final policy
- [ ] Combine safety filtering, heuristics, compact model priors, and shallow search
- [ ] Keep a deterministic fallback path at every stage
- [ ] Add confidence-based fallback to heuristic-only behavior

**Exit criteria:** final tournament bot is robust, fast, and dependency-light.

### 11. Benchmark and harden
- [ ] Run self-play across many seeds
- [ ] Compare against `ExampleBot` and simple greedy/trap baselines
- [ ] Stress cramped endgames and symmetric openings
- [ ] Verify no panics / no invalid moves under long batch runs

**Exit criteria:** stable tournament-ready bot with reproducible results.

### Delegate to smaller local model

- Phase 1
- Phase 2
- Phase 7
- Phase 8
- Phase 10
- much of the raw harness work in Phase 12

### sota

- Phase 3
- Phase 4
- Phase 5
- Phase 6
- Phase 9
- Phase 11
- analysis/decision-making parts of Phase 12


## Suggested next implementation checkpoints

1. Make the safety layer exact.
2. Implement Voronoi / contested-territory scoring.
3. Replace the search placeholder with 1-ply simultaneous search.
4. Add batch evaluation tooling for bot-vs-bot runs.

## Local validation commands

```bash
cargo check
cargo run
```
