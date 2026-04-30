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

## Recovering From a Bad Release

crates.io does not allow deleting a published version — once `X.Y.Z` is on the registry, it stays there forever. The recovery path is to yank the broken version and publish a new one.

1. Yank the bad version so new dependents stop resolving to it. Existing `Cargo.lock` files that already pinned it keep working.

```bash
cargo yank --version X.Y.Z
```

(If you later determine the version was actually fine, `cargo yank --version X.Y.Z --undo` reverses it.)

2. Delete the bad tag locally and on the remote so it does not linger as a published reference:

```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
```

3. If a GitHub Release was created by the workflow, delete it from the GitHub UI (or with `gh release delete vX.Y.Z`).

4. Fix the underlying issue, bump `version` in `Cargo.toml` to the next patch (e.g. `X.Y.Z+1`), and run the normal Release Flow above. Do not reuse the yanked version number.

Yanking is the only supported "undo" — there is no way to overwrite or republish the same version, so always cut a new one.
