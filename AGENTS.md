# Agent Implementation & Usage Guide

This document provides technical context for AI agents working with or using `hxgen`.

## Core Philosophy
`hxgen` is a bridge between a human-readable (and agent-readable) JSON specification and the complex, binary-ish JSON format used by Line 6 Helix devices (`.hlx`). 

Agents should avoid manipulating `.hlx` files directly. Instead, they should work with the **Compact Spec format** and use `hxgen` to compile or decompile.

## 🛠 Command Reference for Agents

| Command | Purpose | Agent Use Case |
| :--- | :--- | :--- |
| `hxgen list` | Search the catalog | Find the correct `symbolicID` for an amp/effect. |
| `hxgen show` | Inspect a model | Understand parameter names, ranges, and defaults. |
| `hxgen schema` | Output JSON Schema | Validate your generated JSON spec before compiling. |
| `hxgen generate` | JSON → `.hlx` | Create a binary preset for the user to load. |
| `hxgen decode` | `.hlx` → JSON | Read an existing preset to modify it. |

## 🏗 Data Structures

### Compact Specification (`HXPresetSpec`)
The top-level structure.
- `blocks`: An array of `HXBlockSpec`.
- `device`: Always `"helix-stomp"` currently.

### Block Structure (`HXBlockSpec`)
- `model`: The `symbolicID` (e.g., `HD2_AmpBrit2203`).
- `params`: Key-value pairs. Only include parameters you want to change from defaults to keep the spec readable.
- `cab`: **Critical for Amps.** If an amp and cab share a slot (A+C), use this field. If they are separate blocks, the cab should be its own `HXBlockSpec` in the `blocks` array.

### Amp + Cab Slots (A+C)
In the Helix, an Amp can "contain" a Cab.
- In JSON Spec: `HXBlockSpec` has a `cab` field of type `CabSpec`.
- In `.hlx`: The Block has a `@cab: "cab0"` pointer and there is a sibling `cab0` object in the DSP flow.
- `hxgen` handles this mapping automatically if the `cab` field is populated in the spec.

## 🧪 Validation: The Roundtrip Test
To verify the integrity of the encoder/decoder, use `roundtrip_test.js`.

**Method:**
1. **Decode** a known good `.hlx`.
2. **Modify** exactly one numeric parameter in the resulting JSON.
3. **Generate** a new `.hlx` from that modified JSON.
4. **Re-Decode** the new `.hlx`.
5. **Assert** that the final JSON is identical to the first JSON, except for the one specifically modified parameter.

Run it with:
```bash
node roundtrip_test.js
```

## 📖 Catalog Management
The catalog is stored in `data/models.json` and embedded in the Rust binary at compile time.

### Updating the Catalog
The catalog is derived from the official Line 6 `HX Edit` application.

1. Locate the resources directory (`/Applications/Line6/HX Edit.app/Contents/Resources`).
2. Run `node extract_hxedit_catalog.js`. This will compile all ~680 official `.models` files into `data/models.json` and sanitize them.
3. Run `node enhance_models.js` to enrich the raw hardware parameter definitions with multi-paragraph text summaries and physical origins from `amplib/`.
4. Re-run `cargo build` so the binary embeds the new JSON.

### Enhancing Descriptions
If you have an `amplib` dump from FluidSolo, run:
```bash
node enhance_models.js
```
This parses the HTML files and updates `models.json` with full multi-paragraph summaries and real-world source information.

## 🤖 Recommended Agent Workflow
If a user asks for a preset (e.g., "Give me a Dookie tone"):
1. **Research:** `hxgen list marshall` to find the JMP or Plexi models.
2. **Schema Check:** `hxgen schema` to ensure your block structure is correct.
3. **Draft Spec:** Create a `spec.json` with the Amp, Cab, and Distortion blocks.
4. **Compile:** `hxgen generate -i spec.json -o dookie.hlx`.
5. **Verify:** Use `hxgen decode -i dookie.hlx` to see if the resulting spec matches your intent.
6. **Delivery:** Provide the `.hlx` file to the user.
