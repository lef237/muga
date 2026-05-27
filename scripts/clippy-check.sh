#!/usr/bin/env bash
set -euo pipefail

cargo clippy --locked --all-targets --all-features -- -D warnings
