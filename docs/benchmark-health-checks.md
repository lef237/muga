# Benchmark Health Checks

Status: release-neutral local health checks for compiler and runtime paths.
These checks are not public performance claims, and they do not define v1
release thresholds.

Run them explicitly with:

```sh
scripts/benchmark-health-check.sh
```

The script runs ignored integration tests:

```sh
cargo test --locked --test benchmark_health -- --ignored --nocapture
```

Each check prints tab-separated lines in this shape:

```txt
benchmark-health	<label>	<elapsed-ms>ms
```

## Coverage

The current health check covers:

- compiler stages over `samples/string_helpers.muga`: lex, parse, check,
  typed HIR, MIR, and bytecode compilation
- package artifact reuse over the conformance package-artifact fixture copied
  under `~/tmp/`: the first build must write artifacts, the second build must
  reuse every artifact, and the built artifacts must run successfully
- representative String/List/Map runtime paths for String helpers,
  `std::list` helpers, and `std::map` helpers

## Policy

Keep these checks lightweight and diagnostic. They should answer whether a
local change obviously damaged representative paths, not whether Muga is faster
than another language or ready for public performance marketing.

Avoid strict timing thresholds until workloads and runtime/backend layers are
stable enough to interpret them. If thresholds become necessary later, keep
them local, generous, and documented as health checks rather than benchmark
claims.
