#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: scripts/trash-generated-muga-locks.sh [--dry-run] [--quiet]" >&2
}

dry_run=0
quiet=0

for arg in "$@"; do
  case "$arg" in
    --dry-run)
      dry_run=1
      ;;
    --quiet)
      quiet=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      usage
      exit 2
      ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

generated_locks=(
  "samples/projects/cli_tool/muga.lock"
  "samples/projects/resource_export/muga.lock"
)

for relative_path in "${generated_locks[@]}"; do
  case "$relative_path" in
    samples/projects/*/muga.lock)
      ;;
    *)
      echo "refusing unexpected generated lock path: $relative_path" >&2
      exit 1
      ;;
  esac

  lock_path="$repo_root/$relative_path"
  if [[ ! -e "$lock_path" ]]; then
    if [[ "$quiet" != "1" ]]; then
      echo "clean $relative_path"
    fi
    continue
  fi

  if [[ ! -f "$lock_path" ]]; then
    echo "refusing to trash non-file path: $relative_path" >&2
    exit 1
  fi

  if [[ "$(basename "$lock_path")" != "muga.lock" ]]; then
    echo "refusing to trash non-muga.lock path: $relative_path" >&2
    exit 1
  fi

  if [[ "$dry_run" == "1" ]]; then
    echo "would trash $relative_path"
    continue
  fi

  if ! command -v trash >/dev/null 2>&1; then
    echo "trash command is required to remove generated muga.lock files" >&2
    exit 1
  fi

  trash "$lock_path"
  echo "trashed $relative_path"
done
