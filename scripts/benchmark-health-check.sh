#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 0 ]]; then
  echo "usage: scripts/benchmark-health-check.sh" >&2
  exit 2
fi

echo "benchmark health checks are release-neutral local measurements, not public performance claims"
cargo test --locked --test benchmark_health -- --ignored --nocapture
