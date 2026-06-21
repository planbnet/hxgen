# hxgen

A standalone CLI for generating Line 6 Helix Stomp `.hlx` preset files from a simple JSON spec.

You describe a signal chain — amps, cabs, pedals, effects — in a compact JSON format, and `hxgen` compiles it into a `.hlx` file ready to load onto your device. It also includes a full searchable catalog of every Helix model with parameter ranges, and can decode existing presets back to JSON for inspection or editing.

## Installation

Download the pre-built binary for your platform from the [latest release](https://github.com/planbnet/hxgen/releases/latest) and place it somewhere in your `PATH` (e.g. `/usr/local/bin/hxgen`).

Or build from source:

```bash
cargo build --release
# binary at: target/release/hx-preset-generator
```

## Quick Start

```bash
# Browse all available models
hxgen list

# Filter by category
hxgen list amp
hxgen list amp marshall

# Inspect a model's parameters and ranges
hxgen show HD2_AmpBrit2203

# Generate a preset from a JSON spec
hxgen generate --input my-preset.json --output my-preset.hlx

# Decode an existing .hlx back to JSON
hxgen decode --input my-preset.hlx

# Print an example spec to stdout
hxgen example

# Output the JSON Schema for the spec format
hxgen schema
```

## Spec Format

Create a JSON file describing your signal chain:

```json
{
  "device": "helix-stomp",
  "name": "My Preset",
  "tempo": 118,
  "blocks": [
    {
      "model": "HD2_DistMinotaur",
      "params": {
        "Gain": 0.42,
        "Tone": 0.48,
        "Level": 0.6
      }
    },
    {
      "model": "HD2_AmpBrit2203",
      "params": {
        "Drive": 0.58,
        "Bass": 0.46,
        "Mid": 0.57,
        "Treble": 0.62
      },
      "cab": {
        "model": "HD2_CabBrit4x12V30"
      }
    },
    {
      "model": "HD2_ReverbPlate",
      "params": {
        "Decay": 0.44,
        "Mix": 0.22
      }
    }
  ]
}
```

- **`model`** — the symbolic ID from `hxgen list` / `hxgen show`
- **`params`** — only include parameters you want to override; everything else uses the Helix default
- **`cab`** — attach a cabinet directly to an amp block (A+C slot); or add it as a standalone block in `blocks`
- **Parameter values** use the native Helix ranges shown by `hxgen show <model>`

## Catalog

The bundled catalog covers the full Helix model library — amps, cabs, distortions, modulations, delays, reverbs, pitch effects, filters, and utilities.

```bash
hxgen list              # all models
hxgen list reverb       # reverbs only
hxgen list amp fender   # Fender-based amps
hxgen show HD2_ReverbPlate  # full parameter details
```

## Transferring to Device

`hxgen` generates `.hlx` files only — it does not connect to the Helix over USB. Transfer the file using one of:

- **HX Edit** (Line 6's official editor) — drag the `.hlx` into a preset slot
- **USB mass storage** — copy the file directly when the Helix is mounted as a drive

## Updating the Catalog

The catalog is embedded in the binary at compile time from `data/models.json`. To regenerate it from a local HX Edit installation:

```bash
# Extract from HX Edit app bundle
node extract_hxedit_catalog.js

# Optionally enrich with descriptions from amplib
node enhance_models.js

# Rebuild the binary
cargo build --release
```

## For AI Agents

An [agentskills.io](https://agentskills.io)-compatible skill is available at [`skills/hx-preset-generator/`](skills/hx-preset-generator/). It guides coding agents through translating a tone description (e.g. from a YouTube transcript) into a working preset file.
