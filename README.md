# hxgen

Standalone CLI for generating Line 6 Helix Stomp `.hlx` preset files.

## Commands

```bash
hxgen list
hxgen list amp marshall
hxgen show HD2_AmpBrit2203
hxgen generate --input ./my-preset.json --output ./my-preset.hlx
hxgen example

# Using cargo directly:
cargo run -- list amp
```

## Browsing the Catalog

You can easily search through the bundled model database using the `list` and `show` commands.

*   **View Full Catalog**: Run `hxgen list` to output all available models.
*   **Filter by Category**: Run `hxgen list <category>` (e.g., `hxgen list amp`, `hxgen list cab`, `hxgen list reverb`).
*   **Fuzzy Search**: Add a search term to find models by name, ID, or real-world source. For example, `hxgen list amp marshall` will return all amps based on Marshall hardware.
*   **View Model Details**: Run `hxgen show <Model_ID>` to view the full descriptions, real-world source, and all adjustable parameters with their valid numerical ranges.

## Building & Installation

This tool has been ported to Rust and compiles into a single, standalone binary. 

To build the optimized release executable:
```bash
cargo build --release
```
The compiled binary will be located at `target/release/hx-preset-generator` (or `hxgen`). You can move this executable anywhere in your system `PATH` (like `/usr/local/bin`) to use it globally.

## Modifying the Device Database

The model catalog, descriptions, and preset templates are baked directly into the binary at compile time. 

If you want to update descriptions or add new models:
1. Edit `data/models.json` or `data/template.json`.
2. Run `cargo build --release` again.
3. The binary will automatically include your changes.

## Spec Format

```json
{
  "device": "helix-stomp",
  "name": "HXGen Example",
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

## Notes

- Helix Stomp is the only exposed target for now, but the code is structured around device profiles so more Helix-family devices can be added later.
- Model lookup accepts either the symbolic ID or the exact display name.
- Parameter values use the native Helix ranges shown by `hxgen show <model>`, and out-of-range values are rejected.
