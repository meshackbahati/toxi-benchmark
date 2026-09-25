# toxi-benchmark

Benchmarks for the Toxi web framework, separated from the framework
repository.

- `servers/` — HTTP comparison servers (Toxi, Rocket, Loco, Poem, Salvo,
  Warp) with identical routes, plus the Loco config.
- `bench.sh` — full matrix runner (build, serve, warmup, measure,
  CSV records).
- `benchmark_runner.py` — JSON shootout runner with chart output.
- `results/` — measured tables with graphs by area.

The `bench` branch holds the HTTP Arena entry (`frameworks/toxi/` with
Dockerfile, `meta.json`, and the arena server) for upstream submission.
