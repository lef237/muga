# Release Gate Alignment

Status: completed v1 hardening audit. This document records how the local
release gate and GitHub Actions stay aligned when local gate changes happen.

## Canonical Gate

`scripts/v1-release-gate.sh` is the canonical offline v1 gate. Keep the command
list in that script first, then let GitHub Actions call the script instead of
copying each command into workflow YAML.

The offline gate runs formatting, clippy, locked tests, a locked build, CLI
smoke checks for source-compatible and artifact-backed package execution, and
offline package/API-diff/archive verification:

```bash
scripts/v1-release-gate.sh
```

Current offline command list:

```bash
cargo fmt --check
scripts/clippy-check.sh
cargo test --locked
cargo build --locked
mkdir -p "$gate_tmp"
target/debug/muga check samples/println_sum.muga
target/debug/muga samples/println_sum.muga
target/debug/muga build samples/packages/app/artifact_facade/main.muga
target/debug/muga check --built samples/packages/app/artifact_facade/main.muga
target/debug/muga run --built samples/packages/app/artifact_facade/main.muga
target/debug/muga api-diff --old-artifact-root samples/packages/app/artifact_facade/.muga/build --new-artifact-root samples/packages/app/artifact_facade/.muga/build --package app::artifact_facade --fail-on breaking
target/debug/muga emit-package-archive --archive-root "$gate_tmp/package-archives" samples/projects/local_path_shared/src/logging/main.muga
target/debug/muga verify-package-archive "$package_archive_path"
target/debug/muga verify-package-archive --expected-hash "$package_archive_hash" "$package_archive_renamed"
target/debug/muga unpack-package-archive --expected-hash "$package_archive_hash" --output-dir "$gate_tmp/renamed-unpacked-package" "$package_archive_renamed"
target/debug/muga check "$gate_tmp/renamed-unpacked-package/src/logging/main.muga"
cp -R samples/projects/my_service "$gate_tmp/my_service"
target/debug/muga emit-app-bundle --source-free --output-dir "$gate_tmp/app-bundle" --program release-gate "$gate_tmp/my_service/src/main/main.muga"
target/debug/muga emit-app-archive --archive-root "$gate_tmp/app-archives" --program release-gate "$gate_tmp/app-bundle"
target/debug/muga verify-app-archive "$app_archive_path"
target/debug/muga verify-app-archive --expected-hash "$app_archive_hash" "$app_archive_renamed"
target/debug/muga unpack-app-archive --expected-hash "$app_archive_hash" --output-dir "$gate_tmp/renamed-unpacked-app" "$app_archive_renamed"
target/debug/muga unpack-app-archive --output-dir "$gate_tmp/unpacked-app" "$app_archive_path"
target/debug/muga run-app-bundle "$gate_tmp/unpacked-app"
target/debug/muga install-app --output-dir "$gate_tmp/installed-bin" --program release-gate "$gate_tmp/unpacked-app"
target/debug/muga list-installed-apps --output-dir "$gate_tmp/installed-bin"
MUGA_BIN="$PWD/target/debug/muga" "$gate_tmp/installed-bin/release-gate"
target/debug/muga uninstall-app --output-dir "$gate_tmp/installed-bin" --program release-gate
cp -R samples/projects/resource_export "$gate_tmp/resource_export"
target/debug/muga emit-app-bundle --source-free --output-dir "$gate_tmp/resource-export-bundle" --program resource-export "$gate_tmp/resource_export/src/main/main.muga"
target/debug/muga run-app-bundle "$gate_tmp/resource-export-bundle" -- "$gate_tmp/resource-export-payload.bin"
cargo package --locked --allow-dirty --offline --list
cargo package --locked --allow-dirty --offline
```

The Clippy sub-gate is `scripts/clippy-check.sh`, which runs
`cargo clippy --locked --all-targets --all-features -- -D warnings`. Keep that
script aligned with `Cargo.toml` lint policy and `clippy.toml` MSRV whenever the
Rust toolchain policy changes.

The publish-time gate adds the crates.io dry run:

```bash
scripts/v1-release-gate.sh --with-publish-dry-run
```

The `--with-publish-dry-run` form may contact crates.io and is intended for
tag-time or release workflow use, not for ordinary offline CI.

## GitHub Actions Contract

- `.github/workflows/ci.yml` installs the pinned Rust toolchain and runs
  `scripts/v1-release-gate.sh`.
- `.github/workflows/release.yml` installs the same Rust toolchain and runs
  `scripts/v1-release-gate.sh --with-publish-dry-run` before authenticating and
  running `cargo publish --locked`.
- `tests/release_readiness.rs` checks that the script still contains the gate
  command list, CI calls the offline script, and the release workflow calls the
  publish-dry-run form before publishing.

## Maintenance Rule

When adding, removing, or reordering local gate commands:

1. Update `scripts/v1-release-gate.sh`.
2. Run `scripts/v1-release-gate.sh` locally and record the result in
   [implementation-resume-plan.md](implementation-resume-plan.md).
3. Update [v1-release-checklist.md](v1-release-checklist.md) and this document
   if the gate contract changes.
4. Keep GitHub Actions invoking the script rather than duplicating the command
   list.

Do not add publish, tag, registry, or network steps to the default offline gate.
Keep those behind `--with-publish-dry-run` or the explicit release workflow.
