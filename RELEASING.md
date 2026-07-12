# Releasing

Muga is published to crates.io as `muga`.

## Choosing the Next Version

Muga version numbers use `X.Y.Z`. During active `0.x` development, `Z` is the
normal release counter:

| Release decision | Field to bump | Example |
|---|---|---|
| Normal small release, including a feature, fix, redesign, or removal | PATCH (`Z`) | `0.6.0` → `0.6.1` |
| Maintainer chooses to mark a broader development generation | MINOR (`Y`) | `0.6.23` → `0.7.0` |
| First mature, long-lived compatibility release | MAJOR (`X`) | `1.0.0-rc.N` → `1.0.0` |

Increment `Z` by default. There is no automatic change category, threshold, or
release count that increments `Y`; that decision belongs to the maintainer.
Changing `Y` within `0.x` does not imply readiness for `1.0.0`.

Numeric components are not limited to one or two digits, so the sequence does
not run out: `0.6.99`, `0.6.100`, and later values are valid. In practice the
available range is vastly larger than any plausible number of releases.

The `0.x` version communicates that the language, standard packages, tools,
and artifact contracts may still change. Every change must still be documented
and tested. Reserve `1.0.0-rc.N` and `1.0.0` for the point when continued small
releases show that foundational redesign is no longer needed and the readiness
criteria in [ROADMAP.md](./ROADMAP.md#10-readiness-criteria) are met. A completed
feature checklist or passing release gate alone does not establish readiness.

After `1.0.0`, use semantic-versioning meaning: PATCH for compatible fixes,
MINOR only for compatible additions the project intentionally chooses to make,
and MAJOR for a breaking compatibility change. The goal is to keep such
post-1.0 changes uncommon, not to forbid maintenance or compatible improvement.

The current version is in the `version` field of `Cargo.toml`.

## Release Flow

### 1. Run pre-release checks

When preparing any release, run the offline release-quality gate and confirm
that user-visible behavior agrees with [LANGUAGE.md](./LANGUAGE.md), the topic
specifications, and [ROADMAP.md](./ROADMAP.md).

```bash
scripts/release-gate.sh
```

Fix any errors before proceeding.

Before the first `1.0.0` release candidate, the release gate must also run on every
documented supported host, exercise crash-safe write recovery, and replay the
checked-in fuzz regression corpus. A green Linux-only run or a time-limited fuzz
session is useful evidence but is not sufficient by itself for the `1.0.0`
operational-quality criterion in [ROADMAP.md](./ROADMAP.md#10-readiness-criteria).

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
scripts/release-gate.sh --with-publish-dry-run
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
runs `scripts/release-gate.sh --with-publish-dry-run`, publishes it to
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
