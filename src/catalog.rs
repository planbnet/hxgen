use crate::types::{HXModelDef, HXParamDef};
use lazy_static::lazy_static;
use std::collections::HashMap;

const MODELS_JSON: &str = include_str!("../data/models.json");

lazy_static! {
    pub static ref HX_MODEL_CATALOG: Vec<HXModelDef> =
        serde_json::from_str(MODELS_JSON).expect("Failed to parse embedded models.json");
        
    pub static ref HX_MODEL_BY_ID: HashMap<String, HXModelDef> = {
        let mut map = HashMap::new();
        for model in HX_MODEL_CATALOG.iter() {
            map.insert(model.symbolic_id.clone(), model.clone());
        }
        map
    };

    static ref CATEGORY_ALIASES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("amp", "amp");
        m.insert("amps", "amp");
        m.insert("amplifier", "amp");
        m.insert("amplifiers", "amp");
        m.insert("preamp", "preamp");
        m.insert("preamps", "preamp");
        m.insert("cab", "cab");
        m.insert("cabs", "cab");
        m.insert("cabinet", "cab");
        m.insert("cabinets", "cab");
        m.insert("ir", "ir");
        m.insert("irs", "ir");
        m.insert("drive", "drive");
        m.insert("drives", "drive");
        m.insert("dist", "drive");
        m.insert("distortion", "drive");
        m.insert("distortions", "drive");
        m.insert("fuzz", "drive");
        m.insert("fuzzes", "drive");
        m.insert("boost", "drive");
        m.insert("boosts", "drive");
        m.insert("comp", "dynamics");
        m.insert("comps", "dynamics");
        m.insert("compressor", "dynamics");
        m.insert("compressors", "dynamics");
        m.insert("dynamics", "dynamics");
        m.insert("gate", "dynamics");
        m.insert("gates", "dynamics");
        m.insert("eq", "eq");
        m.insert("eqs", "eq");
        m.insert("equalizer", "eq");
        m.insert("equalizers", "eq");
        m.insert("modulation", "modulation");
        m.insert("modulations", "modulation");
        m.insert("mod", "modulation");
        m.insert("mods", "modulation");
        m.insert("chorus", "modulation");
        m.insert("flanger", "modulation");
        m.insert("phaser", "modulation");
        m.insert("tremolo", "modulation");
        m.insert("vibrato", "modulation");
        m.insert("rotary", "modulation");
        m.insert("delay", "delay");
        m.insert("delays", "delay");
        m.insert("reverb", "reverb");
        m.insert("reverbs", "reverb");
        m.insert("pitch", "pitch");
        m.insert("pitches", "pitch");
        m.insert("synth", "pitch");
        m.insert("synths", "pitch");
        m.insert("filter", "filter");
        m.insert("filters", "filter");
        m.insert("wah", "wah");
        m.insert("wahs", "wah");
        m.insert("looper", "looper");
        m.insert("loopers", "looper");
        m.insert("volume", "vol-pan");
        m.insert("pan", "vol-pan");
        m.insert("vol-pan", "vol-pan");
        m.insert("utility", "utility");
        m.insert("utilities", "utility");
        m.insert("send", "utility");
        m.insert("return", "utility");
        m
    };
}

fn normalize(value: &str) -> String {
    value.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

pub fn get_canonical_category(input: &str) -> Option<&'static str> {
    CATEGORY_ALIASES.get(normalize(input).as_str()).copied()
}

pub fn matches_category(model: &HXModelDef, category: &str) -> bool {
    let sym = &model.symbolic_id;
    match category {
        "amp" => sym.contains("_Amp") && !sym.contains("_AmpCab"),
        "preamp" => sym.contains("_Preamp"),
        "cab" => sym.contains("_Cab") && !sym.contains("_AmpCab"),
        "ir" => sym.contains("_IR"),
        "drive" => sym.contains("_Dist"),
        "dynamics" => sym.contains("_Compressor") || sym.contains("_Gate"),
        "eq" => sym.contains("_EQ"),
        "modulation" => {
            sym.contains("_Chorus")
                || sym.contains("_Flanger")
                || sym.contains("_Phaser")
                || sym.contains("_Tremolo")
                || sym.contains("_Vibrato")
                || sym.contains("_Rotary")
                || sym.contains("_MM4")
        }
        "delay" => sym.contains("_Delay"),
        "reverb" => sym.contains("_Reverb"),
        "pitch" => sym.contains("_Pitch") || sym.contains("_Synth"),
        "filter" => sym.contains("_Filter"),
        "wah" => sym.contains("_Wah"),
        "looper" => sym.contains("_Looper"),
        "vol-pan" => sym.contains("_Volume") || sym.contains("_Pan"),
        "utility" => {
            sym.contains("_SendReturn")
                || sym.contains("_Return")
                || sym.contains("_Send")
                || sym.contains("_Input")
                || sym.contains("_Output")
        }
        _ => false,
    }
}



pub fn resolve_model(input: &str) -> Result<HXModelDef, String> {
    if let Some(exact) = HX_MODEL_BY_ID.get(input) {
        return Ok(exact.clone());
    }

    let normalized_input = normalize(input);
    if let Some(exact) = HX_MODEL_CATALOG
        .iter()
        .find(|m| normalize(&m.name) == normalized_input)
    {
        return Ok(exact.clone());
    }

    let partial_matches: Vec<&HXModelDef> = HX_MODEL_CATALOG
        .iter()
        .filter(|m| {
            m.symbolic_id.to_lowercase().contains(&input.to_lowercase())
                || normalize(&m.name).contains(&normalized_input)
        })
        .collect();

    if partial_matches.len() == 1 {
        return Ok(partial_matches[0].clone());
    }

    if partial_matches.len() > 1 {
        let candidates = partial_matches
            .iter()
            .take(8)
            .map(|m| format!("{} ({})", m.symbolic_id, m.name))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("Model \"{}\" is ambiguous. Candidates: {}", input, candidates));
    }

    Err(format!(
        "Unknown Helix model \"{}\". Use \"hxgen list\" to search the catalog.",
        input
    ))
}

pub fn resolve_param(model: &HXModelDef, input: &str) -> Result<HXParamDef, String> {
    if let Some(exact) = model
        .params
        .iter()
        .find(|p| p.symbolic_id == input || p.name == input)
    {
        return Ok(exact.clone());
    }

    let normalized_input = normalize(input);
    if let Some(match_param) = model.params.iter().find(|p| {
        normalize(&p.symbolic_id) == normalized_input || normalize(&p.name) == normalized_input
    }) {
        return Ok(match_param.clone());
    }

    Err(format!("Unknown parameter \"{}\" for {}.", input, model.symbolic_id))
}

pub fn format_param_range(param: &HXParamDef) -> String {
    match (&param.min, &param.max, &param.default) {
        (crate::types::NumberOrBool::Bool(_), _, _) => "boolean".to_string(),
        (crate::types::NumberOrBool::Number(min), crate::types::NumberOrBool::Number(max), crate::types::NumberOrBool::Number(def)) => {
            format!("{}..{} (default {})", min, max, def)
        }
        _ => "unknown range".to_string()
    }
}
