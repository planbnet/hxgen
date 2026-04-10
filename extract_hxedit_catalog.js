/**
 * extract_hxedit_catalog.js
 *
 * Scans the provided HX Edit Content/Resources path for .models files,
 * merges their contents, filters out strictly unnecessary fields (like "assign",
 * "load", "load_stereo" to keep the bloat down), and outputs a combined
 * `data/models.json` file.
 */

const fs = require('fs');
const path = require('path');

const HX_RESOURCES_DIR = '/Applications/Line6/HX Edit.app/Contents/Resources';
const OUTPUT_FILE = 'data/models.json';

if (!fs.existsSync(HX_RESOURCES_DIR)) {
    console.error(`Cannot find HX Edit resources directory at: ${HX_RESOURCES_DIR}`);
    process.exit(1);
}

const files = fs.readdirSync(HX_RESOURCES_DIR).filter(f => f.endsWith('.models'));

let allModels = [];

for (const file of files) {
    const fpath = path.join(HX_RESOURCES_DIR, file);
    try {
        const rawJson = fs.readFileSync(fpath, 'utf8');
        const modelsArr = JSON.parse(rawJson);
        
        for (let m of modelsArr) {
            // Skip internal models like @global_params or @dt
            if (m.symbolicID.startsWith('@')) continue;
            // Also skip if no name (just in case)
            if (!m.name) continue;

            // Build cleaned model object
            const cleanModel = {
                symbolicID: m.symbolicID,
                name: m.name,
                mono: m.mono || false,
                stereo: m.stereo || false,
                category: m.category,
                params: []
            };

            if (m.params) {
                for (let p of m.params) {
                    // Skip internal system string parameters like @topology0
                    if (typeof p.default === 'string' || typeof p.min === 'string' || typeof p.max === 'string') {
                        continue;
                    }
                    const cleanParam = {
                        symbolicID: p.symbolicID,
                        name: p.name,
                        valueType: p.valueType,
                        displayType: p.displayType,
                        min: p.min,
                        max: p.max,
                        default: p.default
                    };
                    cleanModel.params.push(cleanParam);
                }
            }

            allModels.push(cleanModel);
        }
    } catch (err) {
        console.error(`Error parsing ${file}: ${err.message}`);
    }
}

// Write the compiled catalog
fs.writeFileSync(OUTPUT_FILE, JSON.stringify(allModels, null, 2));

console.log(`Successfully extracted ${allModels.length} models to ${OUTPUT_FILE}`);
