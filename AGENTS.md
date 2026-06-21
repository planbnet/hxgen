# Agent Developer Guide

This document is for agents working **on** the `hxgen` codebase — building, testing, and modifying it.

For using `hxgen` as a tool to generate Helix presets, see [`skills/hx-preset-generator/SKILL.md`](skills/hx-preset-generator/SKILL.md).

---

## Build

```bash
cargo build --release
# binary: target/release/hx-preset-generator
```

Install to PATH:
```bash
cp target/release/hx-preset-generator /usr/local/bin/hxgen
```

## Test / Roundtrip Validation

```bash
node roundtrip_test.js
```

This decodes a known `.hlx`, modifies one parameter, re-generates, re-decodes, and asserts the output matches.

## Source Layout

| File | Responsibility |
| :--- | :--- |
| `src/main.rs` | CLI entry point, subcommand dispatch |
| `src/types.rs` | Data structures (`HXPresetSpec`, `HXBlockSpec`, etc.) |
| `src/catalog.rs` | Model catalog loading, search, and parameter resolution |
| `src/generator.rs` | `build_preset()` (JSON → `.hlx`) and `decode_preset()` (`.hlx` → JSON) |

## Data Files (embedded at compile time)

| File | Description |
| :--- | :--- |
| `data/models.json` | Full Helix model catalog (~500 models, 1.6 MB) |
| `data/template.json` | Base `.hlx` template that `build_preset()` populates |

Changes to these files require a `cargo build` to take effect.

## Updating the Catalog

The catalog is derived from the official Line 6 HX Edit application.

1. Locate the HX Edit resources directory:
   `/Applications/Line6/HX Edit.app/Contents/Resources`
2. Run `node extract_hxedit_catalog.js` — compiles all `.models` files into `data/models.json`
3. Optionally run `node enhance_models.js` — enriches entries with descriptions from `amplib/`
4. Run `cargo build --release`

## Release

Releases are built automatically by `.github/workflows/release.yml` when a version tag is pushed:

```bash
git tag v1.0.0
git push origin v1.0.0
```

This produces binaries for Linux x86_64/aarch64, macOS x86_64/aarch64, and Windows x86_64.
