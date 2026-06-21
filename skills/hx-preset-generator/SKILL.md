---
name: hx-preset-generator
description: Generate Line 6 Helix Stomp guitar effect preset files (.hlx) from a tone description or spec. Use when a user wants to create, modify, or export a Helix preset — for example after describing a guitar tone, referencing a specific amp or pedal setup, or providing a YouTube video or transcript about a guitar sound. Uses the hxgen CLI to search the Helix model catalog and compile a preset file.
compatibility: Requires hxgen CLI installed and available in PATH. Build from source with cargo or download from https://github.com/planbnet/hxgen/releases.
metadata:
  author: planbnet
  version: "1.0"
allowed-tools: Bash(hxgen:*) Read Write
---

## Overview

`hxgen` converts a compact JSON spec into a `.hlx` preset file that loads directly onto a Helix Stomp. Your job is to translate a tone description into that spec and compile it.

It does **not** connect to the device — you deliver a `.hlx` file the user loads via HX Edit or USB.

## Workflow

### 1. Identify the signal chain

From the user's description (transcript, text, video), extract:
- **Drive/boost pedals** — before the amp
- **Amp** — core character (brand, model, era)
- **Cab** — speaker cabinet (often implied by the amp)
- **Modulation** — chorus, flanger, phaser, tremolo
- **Delay** — type and rough settings
- **Reverb** — type and size

### 2. Find matching models

```bash
hxgen list amp marshall
hxgen list amp fender
hxgen list drive
hxgen list reverb
hxgen list delay
hxgen list mod
```

Pick the best symbolic ID match. When unsure, `hxgen list amp <keyword>` with the real-world amp name usually narrows it down.

### 3. Check parameters

```bash
hxgen show HD2_AmpBrit2203
```

Shows every parameter name, its valid range, and default. Map described settings to values in range.

### 4. Write `spec.json`

```json
{
  "device": "helix-stomp",
  "name": "My Preset",
  "tempo": 120,
  "blocks": [
    {
      "model": "HD2_DistKlon",
      "params": { "Gain": 0.5, "Treble": 0.6, "Level": 0.7 }
    },
    {
      "model": "HD2_AmpBrit2203",
      "params": { "Drive": 0.6, "Bass": 0.45, "Mid": 0.55, "Treble": 0.6 },
      "cab": { "model": "HD2_CabBrit4x12V30" }
    },
    {
      "model": "HD2_ReverbPlate",
      "params": { "Decay": 0.4, "Mix": 0.2 }
    }
  ]
}
```

Rules:
- `model` — symbolic ID from `hxgen list` (e.g. `HD2_AmpBrit2203`)
- `params` — only parameters you want to override; rest use Helix defaults
- `cab` — put the cab inside the amp block for A+C slot mode (most common); or add it as its own entry in `blocks`
- Block order = signal chain order (left to right)
- Parameter values use the native Helix ranges from `hxgen show`, not normalized 0–1

### 5. Generate and verify

```bash
hxgen generate -i spec.json -o preset.hlx
hxgen decode -i preset.hlx   # confirm it round-trips correctly
```

### 6. Deliver

Provide the `.hlx` file. The user loads it via HX Edit or USB mass storage.

## Parameter value guidelines

| Description | Value (relative to range) |
| :--- | :--- |
| off / zero | minimum |
| subtle / light | ~20–30% |
| noon / neutral | ~50% |
| pushed / moderate | ~60–70% |
| heavy / dimed | ~85–100% |

Always compute absolute values using the actual min/max from `hxgen show`.

## Detailed reference

- [Common models by category](references/MODELS.md)
- [Spec format and A+C slot details](references/SPEC.md)
