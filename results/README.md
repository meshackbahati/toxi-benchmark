# Benchmarks

Results by area. Every figure is a divan microbenchmark (dispatch
only, no network) unless labeled **full HTTP**.

- `router.md` — dispatch cost by route shape and route count.
- `extractors.md` — typed extraction cost.
- `responses.md` — response construction and serialization.
- `middleware.md` — per-layer composition cost.
- `throughput.md` — in-process concurrent load.
- `comparison.md` — full HTTP against Rocket, Loco, Poem, Salvo, Warp.

Harness source: `toxi-core/benches/` (divan, CodSpeed-compatible).
CI runs the harness on every pull request; CodSpeed tracks regressions.
