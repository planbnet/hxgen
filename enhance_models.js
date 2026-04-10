/**
 * enhance_models.js
 *
 * Parses every HTML file in the FluidSolo amplib dump and adds the following
 * fields to matching models in data/models.json:
 *   - summary:     full description text (replaces truncated previous value)
 *   - source:      real-world hardware name e.g. "Marshall JCM 800 2203"
 *   - url:         canonical FluidSolo page URL
 *
 * Matching: normalized model names (lowercase, remove special chars/spaces).
 * Run with: node enhance_models.js
 */

const fs = require('fs');
const cheerio = require('cheerio');
const path = require('path');

const MODELS_FILE = 'data/models.json';
const VIEW_MODEL_DIR = 'amplib/www.fluidsolo.com/patchexchange/view-model/';

const models = JSON.parse(fs.readFileSync(MODELS_FILE, 'utf8'));

// Build a normalized name -> model lookup for fast matching
function normalize(str) {
    return str.toLowerCase().replace(/[^a-z0-9]/g, '');
}

const modelByNormalizedName = new Map();
for (const m of models) {
    if (m.name) {
        modelByNormalizedName.set(normalize(m.name), m);
    }
}

const files = fs.readdirSync(VIEW_MODEL_DIR).filter(f => f.endsWith('.html'));

let updated = 0;
let noMatch = 0;

for (const file of files) {
    const html = fs.readFileSync(path.join(VIEW_MODEL_DIR, file), 'utf8');
    const $ = cheerio.load(html);

    // The main card always has: <div class="card-header"><h4>Name <i>Source</i></h4></div>
    const headerH4 = $('.card-header h4').first();
    if (headerH4.length === 0) continue;

    // Model name: raw text nodes of h4 (not the <i> child)
    const nameText = headerH4.contents()
        .filter(function() { return this.type === 'text'; })
        .text()
        .trim();

    // Real-world source: the <i> child of h4
    const sourceText = headerH4.find('i').text().trim()
        .replace(/^\(|\)$/g, '');

    // Full description: from the card body that belongs to the same card
    const cardBody = headerH4.closest('.card').find('.card-body').first().clone();
    // Remove noise elements
    cardBody.find('img, audio, source, a, br, small, em, p.text-muted').remove();
    const description = cardBody.text()
        .split('\n')
        .map(l => l.trim())
        .filter(l => l.length > 0)
        .join('\n\n')
        .trim();

    // Canonical URL: derive from filename slug
    const slug = file.replace('.html', '');
    const url = `https://www.fluidsolo.com/patchexchange/view-model/${slug}`;

    // Look up and update the model
    const model = modelByNormalizedName.get(normalize(nameText));
    if (model) {
        model.summary = description || model.summary || null;
        model.source  = sourceText  || model.source  || null;
        model.url     = url;
        updated++;
    } else {
        noMatch++;
    }
}

fs.writeFileSync(MODELS_FILE, JSON.stringify(models, null, 2));
console.log(`Done. Updated: ${updated}, Unmatched HTML files (not in our catalog): ${noMatch}`);
