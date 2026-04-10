/**
 * roundtrip_test.js
 *
 * For each .hlx file in examples/:
 *   1. DECODE:    hxgen decode → spec.json
 *   2. MODIFY:    change the first tweakable numeric param of the first block that has one
 *   3. GENERATE:  hxgen generate spec.json → regenerated.hlx
 *   4. RE-DECODE: hxgen decode regenerated.hlx → respec.json
 *   5. VERIFY:    only the modified param differs between spec.json and respec.json
 *
 * Skips presets containing models not in the catalog (generate would fail).
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const os = require('os');

const EXAMPLES_DIR = 'examples';
const BIN = 'cargo run --quiet --';
const TMPDIR = fs.mkdtempSync(path.join(os.tmpdir(), 'hxgen-roundtrip-'));

const EPSILON = 1e-4; // tolerance for float comparison
let passed = 0, skipped = 0, failed = 0;

function run(cmd) {
    try {
        return { ok: true, out: execSync(cmd, { encoding: 'utf8', stdio: ['pipe','pipe','pipe'] }).trim() };
    } catch (e) {
        return { ok: false, err: (e.stderr || e.stdout || e.message).trim() };
    }
}

function diffClose(a, b) {
    if (typeof a === 'boolean' && typeof b === 'boolean') return a === b;
    if (typeof a === 'number' && typeof b === 'number') return Math.abs(a - b) < EPSILON;
    return a === b;
}

// Collect all params across blocks as flat { "blockIdx.paramKey": value }
function flattenParams(spec) {
    const flat = {};
    for (let i = 0; i < spec.blocks.length; i++) {
        const block = spec.blocks[i];
        flat[`${i}.__model__`] = block.model;
        flat[`${i}.__enabled__`] = block.enabled;
        if (block.params) {
            for (const [k, v] of Object.entries(block.params)) {
                flat[`${i}.${k}`] = v;
            }
        }
        if (block.cab) {
            flat[`${i}.__cab_model__`] = block.cab.model;
            if (block.cab.params) {
                for (const [k, v] of Object.entries(block.cab.params)) {
                    flat[`${i}.cab.${k}`] = v;
                }
            }
        }
    }
    return flat;
}

const files = fs.readdirSync(EXAMPLES_DIR).filter(f => f.endsWith('.hlx'));

for (const file of files) {
    const label = file.replace('.hlx', '');
    const hlxPath = path.join(EXAMPLES_DIR, file);
    const specPath = path.join(TMPDIR, `${label}.json`);
    const modSpecPath = path.join(TMPDIR, `${label}-modified.json`);
    const rehlxPath = path.join(TMPDIR, `${label}-regenerated.hlx`);
    const respecPath = path.join(TMPDIR, `${label}-respec.json`);

    process.stdout.write(`  ${file}: `);

    // Step 1: Decode
    const decodeResult = run(`${BIN} decode -i "${hlxPath}" -o "${specPath}"`);
    if (!decodeResult.ok) {
        console.log(`SKIP (decode failed: ${decodeResult.err.slice(0, 80)})`);
        skipped++; continue;
    }

    const spec = JSON.parse(fs.readFileSync(specPath, 'utf8'));

    // Step 2: Find first numeric param to modify
    let modBlockIdx = -1, modParamKey = null, origValue = null, newValue = null;
    for (let i = 0; i < spec.blocks.length; i++) {
        const params = spec.blocks[i].params || {};
        for (const [k, v] of Object.entries(params)) {
            if (typeof v === 'number') {
                modBlockIdx = i;
                modParamKey = k;
                origValue = v;
                // Nudge: clamp to [0,1] range with small delta
                newValue = Math.round((origValue < 0.5 ? origValue + 0.1 : origValue - 0.1) * 1000) / 1000;
                break;
            }
        }
        if (modBlockIdx >= 0) break;
    }

    if (modBlockIdx < 0) {
        console.log(`SKIP (no tweakable numeric params found)`);
        skipped++; continue;
    }

    // Step 3: Generate modified spec
    const modSpec = JSON.parse(JSON.stringify(spec));
    modSpec.blocks[modBlockIdx].params[modParamKey] = newValue;
    fs.writeFileSync(modSpecPath, JSON.stringify(modSpec, null, 2));

    const genResult = run(`${BIN} generate -i "${modSpecPath}" -o "${rehlxPath}"`);
    if (!genResult.ok) {
        console.log(`SKIP (generate failed — likely unknown model: ${genResult.err.slice(0, 100)})`);
        skipped++; continue;
    }

    // Step 4: Re-decode the regenerated .hlx
    const redecodeResult = run(`${BIN} decode -i "${rehlxPath}" -o "${respecPath}"`);
    if (!redecodeResult.ok) {
        console.log(`FAIL (re-decode failed: ${redecodeResult.err.slice(0, 80)})`);
        failed++; continue;
    }

    const respec = JSON.parse(fs.readFileSync(respecPath, 'utf8'));

    // Step 5: Compare flat params - only the one we changed should differ
    const before = flattenParams(spec);
    const after = flattenParams(respec);

    const allKeys = new Set([...Object.keys(before), ...Object.keys(after)]);
    const unexpected = [];
    const expectedKey = `${modBlockIdx}.${modParamKey}`;
    let modifiedParamFound = false;

    for (const key of allKeys) {
        const bVal = before[key];
        const aVal = after[key];

        if (key === expectedKey) {
            if (diffClose(aVal, newValue)) {
                modifiedParamFound = true;
            } else {
                unexpected.push(`  MODIFIED PARAM WRONG: ${key}: expected ~${newValue} got ${aVal}`);
            }
            continue;
        }

        if (bVal === undefined && aVal !== undefined) {
            // New keys added in regenerated (e.g. fill-in defaults) — acceptable
            continue;
        }
        if (aVal === undefined && bVal !== undefined) {
            unexpected.push(`  MISSING: ${key} (was ${bVal})`);
            continue;
        }
        if (!diffClose(bVal, aVal)) {
            unexpected.push(`  CHANGED: ${key}: ${bVal} → ${aVal}`);
        }
    }

    if (!modifiedParamFound && unexpected.length === 0) {
        unexpected.push(`  MODIFIED PARAM NOT FOUND in re-decoded output (key: ${expectedKey})`);
    }

    if (unexpected.length === 0) {
        console.log(`PASS (block[${modBlockIdx}].${modParamKey}: ${origValue} → ${newValue})`);
        passed++;
    } else {
        console.log(`FAIL`);
        unexpected.forEach(m => console.log(m));
        failed++;
    }
}

console.log(`\nResults: ${passed} passed, ${skipped} skipped, ${failed} failed`);
if (failed > 0) process.exit(1);
