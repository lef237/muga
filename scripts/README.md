# Scripts

Repository helper scripts live here. Prefer these wrappers over ad hoc shell
commands when they exist, because they encode the repository's expected safety
checks.

## Release And Checks

- `scripts/release-gate.sh`: release-neutral local quality gate. It runs formatting,
  clippy, tests, build, CLI smoke checks, API-diff gating, and offline
  package/app archive verification, app archive unpack/run/install smoke, and
  packaging verification.
- `scripts/benchmark-health-check.sh`: release-neutral local benchmark health
  checks. These are currently one-shot sanity measurements, not statistical
  benchmarks or public performance claims. The roadmap calls for
  representative warm/cold workloads, repeated latency and allocation/memory
  measurements, and machine-readable cross-release comparison output.
- `scripts/clippy-check.sh`: clippy policy wrapper used by the release gate.

## Privacy Guard

- `scripts/privacy-guard.sh`: scans staged content before commit and outgoing
  commits before push for locally configured forbidden literal strings. It also
  checks commit messages and outgoing commit metadata.
- `scripts/install-privacy-hooks.sh`: configures this clone to use the tracked
  `.githooks/` directory.

Install the hooks for this clone:

```sh
scripts/install-privacy-hooks.sh
```

Add one forbidden literal per line to the local-only denylist printed by the
installer. Prefer `.git/info/privacy-denylist` so sensitive values never appear
in the worktree. A local `.privacy-denylist` file is also supported and ignored
by Git.

## Generated Lockfile Cleanup

- `scripts/trash-generated-muga-locks.sh`: safely trashes known generated
  sample-project `muga.lock` files.

Use this script instead of directly running `trash` on generated sample locks
such as `samples/projects/cli_tool/muga.lock` or
`samples/projects/resource_export/muga.lock`. Those paths are also ignored by
Git so release-gate packaging does not pick up generated locks.

```sh
scripts/trash-generated-muga-locks.sh --dry-run
scripts/trash-generated-muga-locks.sh
```

The script refuses unexpected paths, non-files, and files whose basename is not
`muga.lock`.
