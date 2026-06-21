# Spec Format Reference

## Full schema

Get the live JSON Schema from `hxgen schema`. Below is a prose reference.

## Top-level fields

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `device` | string | Yes | Always `"helix-stomp"` |
| `name` | string | No | Preset display name (max ~32 chars) |
| `tempo` | number | No | BPM for tempo-synced effects |
| `blocks` | array | Yes | Ordered list of effect blocks |

## Block fields (`HXBlockSpec`)

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `model` | string | Yes | Symbolic ID from `hxgen list` |
| `params` | object | No | Parameter overrides (name → value) |
| `cab` | object | No | Linked cab for A+C slot (amp blocks only) |

## Cab fields (`CabSpec`)

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `model` | string | Yes | Cab symbolic ID |
| `params` | object | No | Cab parameter overrides |

## A+C Slot (Amp + Cab linked)

The Helix Stomp uses "A+C" DSP slots — an amp and its cab occupy one position.

**Use the `cab` field inside the amp block:**

```json
{
  "model": "HD2_AmpBrit2203",
  "params": { "Drive": 0.6 },
  "cab": {
    "model": "HD2_CabBrit4x12V30"
  }
}
```

`hxgen` handles the internal `@cab` pointer mapping automatically.

**Separate blocks** (each uses its own DSP slot — useful for dual-cab or stereo rigs):

```json
{ "model": "HD2_AmpBrit2203", "params": { "Drive": 0.6 } },
{ "model": "HD2_CabBrit4x12V30" }
```

Use A+C (linked) unless you have a specific reason for separation — it's more DSP-efficient.

## Parameter values

- Use the native Helix ranges shown by `hxgen show <model>`
- Do **not** assume values are normalized to 0–1 unless `hxgen show` says the range is 0–1
- Out-of-range values are rejected at generation time with a clear error

## Block ordering

Blocks are placed in signal chain order left-to-right. Typical order:

```
[Tuner] → [Drive/Boost] → [Amp+Cab] → [Modulation] → [Delay] → [Reverb]
```

## Dual Cab block

A Dual Cab block blends two cabs (with independent mic and position settings) in a single DSP slot. This is common for recording-style tones — same cabinet, different mics panned left and right.

Use `cabB` on a standalone cab block. Both models must be `WithPan` variants (`HD2_CabMicIr_*WithPan`).

```json
{
  "model": "HD2_CabMicIr_4x121960AT75WithPan",
  "params": { "Mic": 1, "Distance": 1.5, "Pan": 0.3 },
  "cabB": {
    "model": "HD2_CabMicIr_4x121960AT75WithPan",
    "params": { "Mic": 5, "Distance": 1.0, "Pan": 0.7 }
  }
}
```

`cabB` cannot be combined with `cab`. Use `cab` only on amp blocks (A+C slot).

## Mic index reference

`hxgen show <model>` prints the mic index mapping inline for any cab parameter. The two mic sets used across the catalog:

**Standard cabs** (`@mic` param, range 0–15):

| Index | Mic |
| :--- | :--- |
| 0 | 57 Dynamic (Shure SM57) |
| 1 | 409 Dynamic (Sennheiser e409) |
| 2 | 421 Dynamic (Sennheiser MD421) |
| 3 | 30 Dynamic |
| 4 | 20 Dynamic |
| 5 | 121 Ribbon (Royer R-121) |
| 6 | 160 Ribbon (Beyerdynamic M160) |
| 7 | 4038 Ribbon (Coles 4038) |
| 8 | 414 Cond (AKG C414) |
| 9 | 84 Cond (Neumann KM84) |
| 10 | 67 Cond (Neumann U67) |
| 11 | 87 Cond (Neumann U87) |
| 12 | 47 Cond (Neumann U47) |
| 13 | 112 Dynamic (AKG D112) |
| 14 | 12 Dynamic |
| 15 | 7 Dynamic (Shure SM7) |

**IR/WithPan cabs** (`Mic` param, range 0–11):

| Index | Mic |
| :--- | :--- |
| 0 | 57 Dynamic (Shure SM57) |
| 1 | 421 Dynamic (Sennheiser MD421) |
| 2 | 7 Dynamic (Shure SM7) |
| 3 | 906 Dynamic (Sennheiser e906) |
| 4 | 30 Dynamic |
| 5 | 121 Ribbon (Royer R-121) |
| 6 | 160 Ribbon (Beyerdynamic M160) |
| 7 | 4038 Ribbon (Coles 4038) |
| 8 | 84 Ribbon |
| 9 | 414 Cond (AKG C414) |
| 10 | 47 Cond FET |
| 11 | 67 Cond (Neumann U67) |

## Minimal working example

```json
{
  "device": "helix-stomp",
  "name": "Clean Fender",
  "blocks": [
    {
      "model": "HD2_AmpFdrTwin",
      "cab": { "model": "HD2_CabFdr2x12" }
    }
  ]
}
```
