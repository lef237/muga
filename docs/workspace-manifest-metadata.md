# Workspace Manifest Metadata

Status: `muga workspace --format json` now reports manifest roots, source roots,
resource roots, and dependency source/resource metadata for entry-reachable
manifest projects.

This slice is the resource-discovery foundation after generated config apps
gained `MUGA_CONFIG_PATH`. It does not make the runtime search the filesystem
implicitly. Instead, editor adapters, CI wrappers, installers, and service
launchers can ask the compiler where the project and dependency source/resource
roots are, then decide how to set paths such as `MUGA_CONFIG_PATH`.

## Goals

Short-Term Goal: expose project root, manifest path, source root, optional
resource root, package path, direct dependency names, and dependency source
facts in the existing workspace JSON command.

Medium-Term Goal: let tooling derive project-relative config/resource paths
without scraping `muga.toml`, changing process current directories, or relying
on private package-loader behavior.

Long-Term Goal: keep package resource lookup and installed app layouts layered
on the same manifest metadata contract used by archive/resource inclusion.

Final Goal: make Muga practical for real tools by giving surrounding tooling a
small, deterministic machine-readable project map.

## JSON Shape

`muga workspace --format json <entry>` now includes:

```json
{
  "project": {
    "manifest": { "path": "muga.toml", "uri": "file:///workspace/app/muga.toml" },
    "root": { "path": ".", "uri": "file:///workspace/app" },
    "sourceRoot": { "path": "src", "uri": "file:///workspace/app/src" },
    "resourceRoot": { "path": "resources", "uri": "file:///workspace/app/resources" },
    "packagePath": "app",
    "directDependencies": ["shared"],
    "dependencies": [
      {
        "packagePath": "shared",
        "root": { "path": "../shared", "uri": "file:///workspace/shared" },
        "sourceRoot": { "path": "../shared/src", "uri": "file:///workspace/shared/src" },
        "resourceRoot": null,
        "sourceKind": "path",
        "source": "../shared",
        "hash": null,
        "dependencies": []
      }
    ]
  }
}
```

The `project` field is `null` for package-mode source trees that do not use
`muga.toml`.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Extend `workspace --format json` with manifest metadata | Reuses an existing machine-readable command; supports editors, CI, installers, and wrappers; no runtime magic. | Adds a stable JSON surface that must stay compatible. | Select |
| Implement TOML config parsing | Familiar config syntax. | Larger format and decoder policy; does not solve path/resource discovery. | Defer |
| Add runtime package resource lookup | Direct app-level API for declared resources. | Needed stable source/archive/cache resource roots from this metadata first. | Done in [runtime-package-resource-lookup.md](runtime-package-resource-lookup.md) |
| Make `muga run` change current directory to the project root | Makes relative config paths appear to work. | Breaks host process expectations and hides path policy. | Reject |
| Teach generated apps to parse `muga.toml` themselves | Keeps logic in source. | Duplicates manifest parsing in user code and exposes unstable manifest details. | Reject |

## Non-Goals

This slice does not add:

- TOML config decoding;
- package resource runtime lookup or installed layout;
- automatic working-directory changes;
- runtime-owned config/resource precedence;
- remote registry metadata.

## Implementation Plan

1. Done: add `project_manifest_metadata_from_entry()` as a read-only package API.
2. Done: include `project` metadata in `workspace --format json`.
3. Done: document the JSON shape and cover it in examples and release-readiness
   tests.
4. Done: add `resourceRoot` to the project and dependency JSON shape.
5. Done: evaluate package resource inclusion as the next archive/hash slice in
   [package-resource-archives.md](package-resource-archives.md).
