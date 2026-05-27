#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

git config core.hooksPath .githooks

denylist_path="$(git rev-parse --git-path info/privacy-denylist)"
if [[ ! -f "$denylist_path" ]]; then
  mkdir -p "$(dirname "$denylist_path")"
  {
    printf '# One forbidden literal per line.\n'
    printf '# Keep this file local; do not commit sensitive values.\n'
  } > "$denylist_path"
fi

cat <<EOF
privacy hooks installed.

Configured:
  core.hooksPath = .githooks

Add forbidden literals to:
  $denylist_path
EOF
