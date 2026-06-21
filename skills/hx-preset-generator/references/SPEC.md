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
