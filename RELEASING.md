# Releasing

Muga is published to crates.io as `muga`.

## Current Manual Checks

Run these before cutting a release:

```bash
cargo fmt --check
cargo test --locked
cargo publish --dry-run --locked
```

## Release Flow

1. Update `version` in `Cargo.toml`.
2. Commit the release changes.
3. Create and push an annotated tag:

```bash
git tag -a vX.Y.Z -m "muga vX.Y.Z"
git push origin main
git push origin vX.Y.Z
```

The release workflow runs when a `v*` tag is pushed. It tests the crate, verifies the package, publishes it to crates.io, and creates a GitHub Release.

## Trusted Publishing Setup

The workflow uses crates.io Trusted Publishing, so it does not need a long-lived crates.io token in GitHub Secrets.

Configure this once on crates.io for the `muga` crate:

- repository: `lef237/muga`
- workflow: `release.yml`
- environment: leave blank

The first release must be published manually before Trusted Publishing can be configured. That has already been done for `v0.1.0`.
