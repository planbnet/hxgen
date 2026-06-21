# hxgen — Agent Skill Guide

This document is a skill guide for coding agents. It explains how to use the `hxgen` CLI to turn a tone description into a working Line 6 Helix Stomp preset file (`.hlx`).

## What hxgen does

`hxgen` converts a compact JSON spec describing a guitar signal chain into a `.hlx` preset file that can be loaded directly onto a Helix Stomp. It also ships a fully searchable catalog of every Helix model (amps, cabs, effects) with parameter ranges, and can decode existing `.hlx` files back to JSON.

It does **not** connect to the device over USB — it only produces files.

---

## Command Reference

| Command | What it does |
| :--- | :--- |
| `hxgen list` | List all models in the catalog |
| `hxgen list <category>` | Filter by category (amp, cab, drive, reverb, delay, mod, pitch, filter) |
| `hxgen list <category> <term>` | Fuzzy search within a category (e.g. `hxgen list amp marshall`) |
| `hxgen show <ModelID>` | Show full parameter details and ranges for one model |
| `hxgen schema` | Print the JSON Schema for the spec format (use to validate before generating) |
| `hxgen generate -i spec.json -o preset.hlx` | Compile a JSON spec into a `.hlx` preset |
| `hxgen decode -i preset.hlx` | Decode an existing `.hlx` back to compact JSON |
| `hxgen example` | Print an example spec to stdout |

---

## Spec Format

```json
{
  "device": "helix-stomp",
  "name": "Preset Name",
  "tempo": 120,
  "blocks": [
    {
      "model": "HD2_DistKlon",
      "params": {
        "Gain": 0.5,
        "Treble": 0.6,
        "Level": 0.7
      }
    },
    {
      "model": "HD2_AmpBrit2203",
      "params": {
        "Drive": 0.6,
        "Bass": 0.45,
        "Mid": 0.55,
        "Treble": 0.6,
        "Presence": 0.5,
        "Master": 0.7
      },
      "cab": {
        "model": "HD2_CabBrit4x12V30",
        "params": {}
      }
    },
    {
      "model": "HD2_DelayMultiHead",
      "params": {
        "Time": 0.4,
        "Feedback": 0.3,
        "Mix": 0.25
      }
    },
    {
      "model": "HD2_ReverbPlate",
      "params": {
        "Decay": 0.4,
        "Mix": 0.2
      }
    }
  ]
}
```

### Key rules

- **`model`** — the `symbolicID` from `hxgen list` output (e.g. `HD2_AmpBrit2203`)
- **`params`** — only specify parameters you want to override; all others use Helix defaults
- **`cab`** — place the cab inside the amp block when they share a slot (A+C mode, most common); or add it as a separate entry in `blocks`
- **Parameter values** are floats in the native Helix range shown by `hxgen show <model>` — do not normalize to 0–1 unless the range actually is 0–1
- **Block order** in the array maps to the signal chain left-to-right

---

## Agent Workflow: Tone Description → Preset

When a user describes a tone (from a video transcript, text, or conversation), follow these steps:

### 1. Identify the signal chain components

From the description, extract:
- **Amp** — the core amplifier character (e.g. "Marshall JCM800", "Fender Twin", "AC30")
- **Cab** — speaker cabinet (often implicit from the amp)
- **Drive/Boost** — overdrive or distortion pedals in front of the amp
- **Modulation** — chorus, flanger, phaser, tremolo
- **Delay** — echo/delay type and rough settings
- **Reverb** — room, plate, spring, hall, etc.

### 2. Find matching models

```bash
hxgen list amp marshall    # find Marshall-based amps
hxgen list amp fender
hxgen list drive
hxgen list reverb
```

Pick the best match by symbolic ID. When multiple options exist, prefer the one whose description best matches the source described.

### 3. Inspect parameters

```bash
hxgen show HD2_AmpBrit2203
```

This shows every parameter name, its valid range, and the default. Map the described settings (e.g. "gain around noon", "bright treble", "short reverb") to values within the shown range.

### 4. Write the spec

Create a `spec.json` with the blocks in signal chain order. Only include parameters that differ meaningfully from a neutral setting.

### 5. Validate (optional)

```bash
hxgen schema > schema.json
# validate spec.json against schema.json with any JSON Schema validator
```

### 6. Generate the preset

```bash
hxgen generate -i spec.json -o preset.hlx
```

### 7. Verify

```bash
hxgen decode -i preset.hlx
```

Check that the decoded output matches your intent. If a parameter is missing or wrong, adjust the spec and regenerate.

### 8. Deliver

Provide the `.hlx` file to the user. They load it via HX Edit or USB mass storage.

---

## Common Model IDs by Category

Run `hxgen list <category>` to see the full list. A few common starting points:

**Amps**
- `HD2_AmpBrit2203` — Marshall JCM800
- `HD2_AmpBritBlues` — Marshall Bluesbreaker
- `HD2_AmpUSALead` — Mesa Boogie Dual Rectifier
- `HD2_AmpFdrTwin` — Fender Twin Reverb
- `HD2_AmpVox` — Vox AC30
- `HD2_AmpPVH` — Peavey 5150

**Cabs** (pair with matching amp)
- `HD2_CabBrit4x12V30` — Marshall 4x12 with Celestion V30s
- `HD2_CabUSA4x12` — Mesa 4x12
- `HD2_CabFdr2x12` — Fender 2x12

**Drive**
- `HD2_DistKlon` — Klon Centaur
- `HD2_DistMinotaur` — Ibanez Tube Screamer
- `HD2_DistBigMuff` — EHX Big Muff

**Reverb**
- `HD2_ReverbPlate` — classic plate
- `HD2_ReverbHall` — large hall
- `HD2_ReverbSpring` — spring reverb

---

## Parameter Value Guidelines

When translating subjective descriptions to numbers:

| Description | Approximate value (relative to range) |
| :--- | :--- |
| "off" / "zero" | minimum of range |
| "subtle" / "light" | 20–30% of range |
| "noon" / "unity" / "neutral" | 50% of range |
| "pushed" / "moderate" | 60–70% of range |
| "heavy" / "dimed" | 85–100% of range |

Always check the actual min/max from `hxgen show` before computing absolute values.

---

## Amp + Cab Slot (A+C) Details

The Helix Stomp uses "A+C" slots where an amp and cab occupy a single DSP block position. This is the most common and efficient configuration.

In the spec, put the cab inside the amp block:

```json
{
  "model": "HD2_AmpBrit2203",
  "params": { ... },
  "cab": {
    "model": "HD2_CabBrit4x12V30"
  }
}
```

`hxgen` handles the internal mapping automatically. If you place the cab as a separate block in `blocks`, it will occupy its own slot (useful for stereo or dual-cab rigs).
