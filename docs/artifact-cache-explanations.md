# Artifact Cache Explanations

Status: read-only artifact/cache explanation command is implemented with
compact human text output and JSON output, including local archive-cache
metadata. Broader editor/agent integration is still deferred.

Muga already reports artifact paths, hashes, and regeneration commands in
focused diagnostics. The next tooling layer should make the same facts
queryable before editor, CI, or agent workflows depend on rebuild reasoning.
The implemented command is:

```text
muga why-rebuild [--format text|json] [--artifact-root <dir>|--built] <entry>
```

`muga why-artifact` remains a possible later spelling for inspecting one
artifact path directly, but the first useful surface should explain the
entry-reachable package graph because `.mgi`, `.mgc`, and `.mgb` artifacts are
validated together.

## Goals

- explain why an entry package or reachable dependency artifact is `missing`,
  `fresh`, `stale`, `hashMismatch`, `invalid`, or `unknown`
- expose the concrete `artifactRoot`, `lockfile`, `archiveCache`,
  `artifactFile`, `artifactHash`, and `regenerationCommand` facts that current
  JSON diagnostics already use
- keep the command non-mutating: it must not build, rewrite, delete, refresh,
  or materialize artifacts
- keep artifact-backed reasoning honest: it does not read dependency implementation source
  to make a stale or missing artifact look usable
- give humans concise rebuild guidance while giving tools a stable
  `--format json` envelope

## Input Rules

`--artifact-root <dir>` inspects an explicit artifact directory. `--built`
selects the same default `.muga/build` directory used by `muga build`,
`check --built`, and `run --built`. If neither flag is passed, the command uses
that same default build directory.

The `<entry>` argument identifies the package graph. The command may parse
manifest and package metadata needed to identify the graph, but artifact-backed
status must be based on current on-disk artifacts and their stored metadata.
For dependency packages, it does not read dependency implementation source as a
fallback for `.mgb` validity. Source hashes may be recomputed for local packages
only to explain whether an artifact's recorded source input still matches the
current source tree.

The command should fail with normal diagnostics for malformed manifests,
unsupported dependency forms, unreadable roots, or ambiguous entry packages.
Malformed existing lockfiles are reported inside `lockfile.state` as
`"invalid"` when the manifest still provides enough data to compute the
expected metadata. Command-level diagnostics use the same JSON diagnostic
envelope as `check --format json`.

## JSON Shape

The JSON command writes one object to stdout and leaves stderr empty for
compiler/artifact diagnostics, matching the existing JSON commands:

```json
{
  "schemaVersion": 1,
  "command": "why-rebuild",
  "entry": {
    "path": "samples/packages/app/artifact_facade/main.muga",
    "uri": "file:///workspace/muga/samples/packages/app/artifact_facade/main.muga"
  },
  "status": "ok",
  "diagnostics": [],
  "artifactRoot": {
    "path": "samples/packages/app/artifact_facade/.muga/build",
    "uri": "file:///workspace/muga/samples/packages/app/artifact_facade/.muga/build",
    "selection": "built"
  },
  "lockfile": {
    "kind": "lockfile",
    "path": "samples/projects/local_path_app/muga.lock",
    "uri": "file:///workspace/muga/samples/projects/local_path_app/muga.lock",
    "state": "fresh",
    "reason": "package lockfile metadata matches current dependencies",
    "dependencies": [
      {
        "packagePath": "shared",
        "sourceKind": "archive",
        "source": "../archives/shared.mgp",
        "hashKind": "archive",
        "hash": "sha256:<hex>",
        "dependencies": []
      }
    ],
    "metadataHash": [
      {
        "kind": "artifactHash",
        "role": "actual",
        "hashKind": "lockfile",
        "value": "sha256:<hex>"
      }
    ],
    "regenerationCommand": []
  },
  "archiveCache": [
    {
      "kind": "archiveCache",
      "packagePath": "shared",
      "path": "samples/projects/local_archive_app/.muga/packages/shared-sha256-<hex>",
      "uri": "file:///workspace/muga/samples/projects/local_archive_app/.muga/packages/shared-sha256-<hex>",
      "source": "../archives/shared.mgp",
      "sourceUri": "file:///workspace/muga/samples/projects/archives/shared.mgp",
      "state": "fresh",
      "reason": "package archive dependency cache matches declared archive hash",
      "metadataHash": [
        {
          "kind": "artifactHash",
          "role": "actual",
          "hashKind": "archiveCache",
          "packagePath": "shared",
          "value": "sha256:<hex>"
        }
      ],
      "regenerationCommand": []
    }
  ],
  "packages": [
    {
      "path": "app::artifact_facade",
      "role": "entry"
    }
  ],
  "artifacts": [
    {
      "artifactKind": "interface",
      "packagePath": "app::artifact_facade",
      "path": "samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi",
      "uri": "file:///workspace/muga/samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi",
      "state": "fresh",
      "reason": "artifact metadata matches current package interface",
      "artifactFile": {
        "kind": "artifactFile",
        "artifactKind": "interface",
        "path": "samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi",
        "uri": "file:///workspace/muga/samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi"
      },
      "artifactHash": [
        {
          "kind": "artifactHash",
          "role": "actual",
          "hashKind": "interface",
          "value": "sha256:<hex>"
        }
      ],
      "regenerationCommand": []
    }
  ]
}
```

Field rules:

- `schemaVersion` starts at `1`.
- `command` is `"why-rebuild"` even when the CLI spelling is `muga
  why-rebuild`.
- `entry.path` preserves the user-provided entry path; `entry.uri` is a
  best-effort absolute `file://` URI.
- `status` is `"ok"` when all requested explanation data was produced and
  `"error"` when diagnostics prevent a complete explanation.
- `diagnostics` uses the same diagnostic object shape documented in
  [diagnostics-and-output.md](diagnostics-and-output.md).
- `artifactRoot.path`, `artifactRoot.uri`, and `artifactRoot.selection`
  identify the inspected root and whether it came from `--artifact-root` or
  `--built`.
- `lockfile` is `null` when no manifest project lockfile applies. For manifest
  projects it reports the expected `muga.lock` path, dependency source
  metadata, `metadataHash`, `state`, `reason`, and regeneration command data
  without rewriting the lockfile.
- `lockfile.dependencies[].sourceKind` is `"path"` for local path dependencies
  or `"archive"` for local `.mgp` archive dependencies. `hashKind` is
  `"source"` for local path `source_hash` metadata and `"archive"` for archive
  `hash` metadata.
- `archiveCache` lists local `.mgp` dependency cache entries under
  `.muga/packages`. Each entry reports `"kind":"archiveCache"`,
  `packagePath`, cache `path` / `uri`, source archive `source` / `sourceUri`,
  `metadataHash`, `state`, `reason`, and `regenerationCommand`.
- `archiveCache[].metadataHash[].hashKind` is `"archiveCache"` and compares
  the materialized cache contents against the declared archive hash.
- Current local archive dependency loading validates and materializes cache
  entries before the package graph can be explained, so the implemented JSON
  coverage records verified cache metadata. A future read-only manifest loader
  would be needed to classify missing archive caches without materialization.
- `packages` lists entry-reachable package paths and roles such as `"entry"` or
  `"dependency"`.
- `artifacts[].artifactKind` is `"interface"`, `"checkCache"`, or
  `"implementation"`.
- `artifacts[].state` is one of `"missing"`, `"fresh"`, `"stale"`,
  `"hashMismatch"`, `"invalid"`, or `"unknown"`.
- `artifacts[].reason` is short human-readable text. Tools should key behavior
  on `state`, artifact identity, and hash fields rather than this prose.
- `artifactFile`, `artifactHash`, and `regenerationCommand` reuse the current
  diagnostic context names so consumers can share parsing logic.
- For implementation artifacts, dependency-interface set changes include
  expected `dependencyInterface` hashes for newly required dependencies and
  actual `dependencyInterface` hashes for dependencies no longer required.
- `regenerationCommand` entries should be concrete, for example
  `"muga build <entry>"`, `"muga emit-artifacts --artifact-root <dir> <entry>"`,
  `"muga emit-interface --artifact-root <dir> <entry>"`, or
  `"muga emit-check-cache --artifact-root <dir> <entry>"`.

## Human Output

Human text output is the default. It is compact and tab-separated so users can
scan rebuild state without tools needing to depend on it:

```text
fresh<TAB>interface<TAB>app::artifact_facade<TAB>.muga/build/app__artifact_facade.mgi<TAB>artifact metadata matches current package interface
missing<TAB>checkCache<TAB>app::artifact_facade<TAB>.muga/build/app__artifact_facade.mgc<TAB>expected package check cache artifact is missing<TAB>run: muga build samples/packages/app/artifact_facade/main.muga
stale<TAB>implementation<TAB>util::numbers<TAB>.muga/build/util__numbers.mgb<TAB>source hash changed<TAB>run: muga build samples/packages/app/artifact_facade/main.muga
hashMismatch<TAB>checkCache<TAB>app::artifact_facade<TAB>.muga/build/app__artifact_facade.mgc<TAB>package check cache artifact hash mismatch<TAB>run: muga emit-check-cache --artifact-root artifacts samples/packages/app/artifact_facade/main.muga
invalid<TAB>implementation<TAB>util::numbers<TAB>.muga/build/util__numbers.mgb<TAB>invalid package implementation bytecode<TAB>run: muga emit-artifacts --artifact-root artifacts samples/packages/app/artifact_facade/main.muga
```

General row shape is `state<TAB>kind<TAB>package<TAB>path<TAB>reason`. The
package field is `-` for lockfiles, and rows may add optional `run:`
regeneration guidance. The exact human prose may improve. Machine consumers
should use `muga why-rebuild --format json`.

## State Meanings

- `missing`: the expected artifact path is absent.
- `fresh`: the artifact is present, structurally valid, and its recorded inputs
  match the currently known package/interface/dependency hashes.
- `stale`: the artifact is valid but was produced from old source,
  dependency-interface, or package metadata.
- `hashMismatch`: an artifact, source, interface, dependency-interface, archive,
  or cache hash differs from the expected value.
- `invalid`: the file exists but cannot be parsed or structurally validated as
  the expected artifact or lockfile kind.
- `unknown`: Muga cannot classify the artifact without a broader future design,
  such as non-local package metadata or a not-yet-implemented dependency form.

## Non-Goals

- no mutation, cleanup, `muga clean`, cache pruning, or rebuild execution
- no version solving, remote fetch, registry audit, package signing, or SBOM
  generation
- no daemon, watch mode, full incremental scheduler, or performance promise
- no artifact-root manifest configuration
- no broad editor protocol; JSON output remains the integration boundary
- no change to scalar-only v1 equality or other source-language semantics

## Implementation Order

1. land this design and release-readiness coverage
2. land initial read-only `muga why-rebuild --format json` for local
   package graphs and `.mgi` / `.mgc` / `.mgb` artifacts
3. add archive-cache metadata coverage for local `.mgp` dependencies
4. add compact text output after the JSON fields are covered
5. wire editor, CI, or agent workflows to the JSON output only after focused
   missing/fresh/stale/hashMismatch/invalid fixtures exist

The initial `cli_why_rebuild_json_reports_*` tests cover fresh reused
artifacts, missing `.mgi` / `.mgc` / `.mgb` artifacts, stale source hashes,
stale dependency-interface hashes, hashMismatch cases, invalid artifact
payloads, implementation dependency-interface set changes, local path/archive
lock metadata, local `.mgp` archive cache
metadata, explicit `--artifact-root`, and `--built`. The
`cli_why_rebuild_text_reports_*` tests cover human text output for fresh
artifacts, missing explicit artifacts, lockfile metadata, and archive-cache
metadata before broader agent or editor workflows depend on those cases.
