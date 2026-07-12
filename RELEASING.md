# Releasing

Muga is published to crates.io as `muga`.

## Choosing the Next Version

Follow semantic versioning (`MAJOR.MINOR.PATCH`):

Muga uses the following project-specific policy while it is still pre-v1:

| Project phase | Field to bump | Example |
|---|---|---|
| Any pre-v1 release, including features and breaking changes | PATCH | `0.6.0` → `0.6.1` |
| v1 release candidates, after the v1 entry criteria are met | prerelease | `1.0.0-rc.1` → `1.0.0-rc.2` |
| First mature, long-lived compatibility release | MAJOR | `1.0.0-rc.N` → `1.0.0` |
| Compatible maintenance after v1 | PATCH | `1.0.0` → `1.0.1` |

Before `1.0.0`, increment only the patch component for each release. A pre-v1
feature or breaking change therefore moves `0.6.0` to `0.6.1`, not to `0.7.0`
or `1.0.0`. The `0.x` version communicates that the language, standard
packages, tools, and artifact contracts may still change; every such change
must still be documented and tested.

Do not infer v1 readiness from the amount of implemented functionality, a
completed checklist, or a passing release gate alone. Reserve `1.0.0-rc.N`
and `1.0.0` until the maturity criteria in [ROADMAP.md](./ROADMAP.md) are met
and the project is ready to maintain the resulting language and ecosystem with
minimal redesign after v1.

The current version is in the `version` field of `Cargo.toml`.

## Release Flow

### 1. Run pre-release checks

When preparing any release, run the offline release-quality gate. For v1
release candidates and later, also confirm the scope against
[spec-v1.md](./spec-v1.md) and the v1 maturity criteria in
[ROADMAP.md](./ROADMAP.md).

```bash
scripts/v1-release-gate.sh
```

Fix any errors before proceeding.

### 2. Bump the version

Edit the `version` field in `Cargo.toml`:

```toml
version = "X.Y.Z"
```

Then update `Cargo.lock` to match:

```bash
cargo check
```

### 3. Commit

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to vX.Y.Z"
```

### 4. Run the publish dry run

After the version bump commit and before tagging, run the full release gate with the crates.io publish dry run:

```bash
scripts/v1-release-gate.sh --with-publish-dry-run
```

This may contact crates.io. Fix any errors before tagging.

### 5. Create an annotated tag

```bash
git tag -a vX.Y.Z -m "muga vX.Y.Z"
```

Verify the tag was created correctly:

```bash
git tag -n | tail -5
```

### 6. Push

```bash
git push origin main
git push origin vX.Y.Z
```

Pushing a `v*` tag triggers the `release.yml` GitHub Actions workflow, which
runs `scripts/v1-release-gate.sh --with-publish-dry-run`, publishes it to
crates.io, and creates a GitHub Release.

### 7. Verify the release

1. Check the Actions tab on GitHub and confirm the workflow succeeded.
2. Confirm the new version appears on [crates.io/crates/muga](https://crates.io/crates/muga).
3. Confirm a new release was created on the GitHub Releases page.

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
