use crate::catalog::{resolve_model, resolve_param};
use crate::types::{CabSpec, HXBlockSpec, HXModelDef, HXParamDef, HXPresetSpec, NumberOrBool};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

const TEMPLATE_JSON: &str = include_str!("../data/template.json");

fn sanitize_preset_name(name: Option<&str>) -> String {
    let name_str = name.unwrap_or("HX Preset");
    let regex = regex::Regex::new(r#"[<>:"/\\|?\*\x00-\x1F]"#).unwrap();
    let sanitized = regex.replace_all(name_str, "");
    sanitized.chars().take(32).collect()
}

fn assert_numeric_range(value: f64, param: &HXParamDef) -> Result<(), String> {
    if let (NumberOrBool::Number(min), NumberOrBool::Number(max)) = (&param.min, &param.max) {
        if value < *min || value > *max {
            return Err(format!(
                "Parameter {} must be between {} and {}; received {}.",
                param.symbolic_id, min, max, value
            ));
        }
    }
    Ok(())
}

fn coerce_param_value(param: &HXParamDef, value: Option<&NumberOrBool>) -> Result<serde_json::Value, String> {
    if value.is_none() {
        return match &param.default {
            NumberOrBool::Bool(b) => Ok(json!(*b)),
            NumberOrBool::Number(n) => Ok(json!(*n)),
        };
    }

    let val = value.unwrap();

    match &param.default {
        NumberOrBool::Bool(_) => {
            if let Some(b) = val.as_bool() {
                Ok(json!(b))
            } else {
                Err(format!("Parameter {} expects a boolean value.", param.symbolic_id))
            }
        }
        NumberOrBool::Number(_) => {
            if let Some(n) = val.as_f64() {
                if n.is_nan() {
                    return Err(format!("Parameter {} expects a numeric value.", param.symbolic_id));
                }
                assert_numeric_range(n, param)?;
                if param.value_type == Some(0) {
                    Ok(json!(n.round()))
                } else {
                    Ok(json!(n))
                }
            } else {
                Err(format!("Parameter {} expects a numeric value.", param.symbolic_id))
            }
        }
    }
}

fn build_block(model: &HXModelDef, block_spec: &HXBlockSpec, position: u32) -> Result<serde_json::Map<String, Value>, String> {
    let mut block = serde_json::Map::new();
    
    let is_enabled = block_spec.enabled.unwrap_or(true);
    let path = block_spec.path.unwrap_or(0);

    block.insert("@model".to_string(), json!(model.symbolic_id));
    block.insert("@enabled".to_string(), json!(is_enabled));
    block.insert("@path".to_string(), json!(path));
    block.insert("@position".to_string(), json!(position));
    block.insert("@no_snapshot_bypass".to_string(), json!(false));

    for param in &model.params {
        let def_val = match &param.default {
            NumberOrBool::Bool(b) => json!(*b),
            NumberOrBool::Number(n) => json!(*n),
        };
        block.insert(param.symbolic_id.clone(), def_val);
    }

    if let Some(params_map) = &block_spec.params {
        for (key, raw_value) in params_map {
            let param = resolve_param(model, key)?;
            let coerced = coerce_param_value(&param, Some(raw_value))?;
            block.insert(param.symbolic_id.clone(), coerced);
        }
    }

    // @type: 1 = amp-only, 3 = amp+cab (A+C slot), 2 = standalone cab, 0 = effect
    // We don't set @type here since it depends on whether a cab is linked; caller sets it.
    block.insert("@enabled".to_string(), json!(is_enabled));
    Ok(block)
}

fn sort_and_assign_positions(blocks: Vec<HXBlockSpec>) -> Result<Vec<(HXBlockSpec, u32)>, String> {
    let mut explicit = Vec::new();
    let mut implicit = Vec::new();

    for block in blocks {
        if block.position.is_some() {
            explicit.push(block);
        } else {
            implicit.push(block);
        }
    }

    explicit.sort_by_key(|b| b.position.unwrap_or(0));

    let mut result = Vec::new();
    let mut taken = HashSet::new();

    for block in explicit {
        let pos = block.position.unwrap();
        if taken.contains(&pos) {
            return Err(format!("Duplicate block position {}.", pos));
        }
        taken.insert(pos);
        result.push((block, pos));
    }

    let mut next_pos = 0;
    for block in implicit {
        while taken.contains(&next_pos) {
            next_pos += 1;
        }
        taken.insert(next_pos);
        result.push((block, next_pos));
        next_pos += 1;
    }

    result.sort_by_key(|(_, pos)| *pos);
    Ok(result)
}

pub fn build_preset(spec: HXPresetSpec) -> Result<serde_json::Value, String> {
    if let Some(ref device) = spec.device {
        if device != "helix-stomp" {
            return Err(format!("Unsupported device \"{}\". Only \"helix-stomp\" is available right now.", device));
        }
    }

    let blocks_with_pos = sort_and_assign_positions(spec.blocks)?;

    if blocks_with_pos.is_empty() {
        return Err("Preset spec must contain at least one block.".to_string());
    }

    let max_blocks = 8;
    if blocks_with_pos.len() > max_blocks {
        return Err(format!("Helix Stomp supports at most {} user blocks in this generator.", max_blocks));
    }

    let mut preset: Value = serde_json::from_str(TEMPLATE_JSON).map_err(|e| e.to_string())?;
    
    let meta_name = sanitize_preset_name(spec.name.as_deref());
    let tempo = spec.tempo.unwrap_or(120.0);
    let now_seconds = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    preset["data"]["meta"]["name"] = json!(meta_name);
    preset["data"]["meta"]["modifieddate"] = json!(now_seconds);
    preset["data"]["tone"]["global"]["@tempo"] = json!(tempo);
    preset["data"]["tone"]["global"]["@cursor_group"] = json!("block0");
    preset["data"]["tone"]["global"]["@cursor_position"] = json!(0);

    for snapshot in &["snapshot0", "snapshot1", "snapshot2"] {
        preset["data"]["tone"][snapshot]["@tempo"] = json!(tempo);
        preset["data"]["tone"][snapshot]["@valid"] = json!(true);
        preset["data"]["tone"][snapshot]["blocks"] = json!({ "dsp0": {} });
    }

    for i in 0..max_blocks {
        let block_key = format!("block{}", i);
        if let Some(dsp0) = preset["data"]["tone"]["dsp0"].as_object_mut() {
            dsp0.remove(&block_key);
        }
    }

    let mut highest_position = 0;
    let mut cab_index = 0usize;

    for (index, (block_spec, pos)) in blocks_with_pos.iter().enumerate() {
        let model = resolve_model(&block_spec.model)?;
        let block_key = format!("block{}", index);
        let mut block_obj = build_block(&model, block_spec, *pos)?;
        
        highest_position = std::cmp::max(highest_position, *pos);
        
        // Handle A+C slot: link cab via @cab pointer and write sibling cab key
        if let Some(cab_spec) = &block_spec.cab {
            let cab_key = format!("cab{}", cab_index);
            block_obj.insert("@cab".to_string(), json!(cab_key));
            block_obj.insert("@type".to_string(), json!(3)); // 3 = amp+cab
            block_obj.insert("@bypassvolume".to_string(), json!(1));

            // Build the sibling cab object
            let mut cab_obj = serde_json::Map::new();
            cab_obj.insert("@model".to_string(), json!(cab_spec.model));
            cab_obj.insert("@enabled".to_string(), json!(true));
            // Write any explicit cab params
            if let Some(cab_params) = &cab_spec.params {
                for (k, v) in cab_params {
                    match v {
                        NumberOrBool::Number(n) => { cab_obj.insert(k.clone(), json!(n)); }
                        NumberOrBool::Bool(b) => { cab_obj.insert(k.clone(), json!(b)); }
                    }
                }
            }
            preset["data"]["tone"]["dsp0"][&cab_key] = Value::Object(cab_obj);
            cab_index += 1;
        } else if model.symbolic_id.contains("_Amp") {
            block_obj.insert("@type".to_string(), json!(1)); // amp-only
        }

        let is_enabled = block_obj.get("@enabled").unwrap_or(&json!(true)).clone();

        preset["data"]["tone"]["dsp0"][&block_key] = Value::Object(block_obj);

        for snapshot in &["snapshot0", "snapshot1", "snapshot2"] {
            preset["data"]["tone"][snapshot]["blocks"]["dsp0"][&block_key] = json!(is_enabled);
        }
    }

    preset["data"]["tone"]["dsp0"]["join"]["@position"] = json!(highest_position + 1);

    Ok(preset)
}

pub fn decode_preset(raw_preset: serde_json::Value) -> Result<HXPresetSpec, String> {
    let mut spec = HXPresetSpec {
        device: None,
        name: None,
        tempo: None,
        blocks: Vec::new(),
    };

    if let Some(device_id) = raw_preset["data"]["device"].as_i64() {
        if device_id == 2162694 {
            spec.device = Some("helix-stomp".to_string());
        } else {
            spec.device = Some(device_id.to_string());
        }
    }

    if let Some(name) = raw_preset["data"]["meta"]["name"].as_str() {
        spec.name = Some(name.to_string());
    }

    if let Some(tempo) = raw_preset["data"]["tone"]["global"]["@tempo"].as_f64() {
        spec.tempo = Some(tempo);
    }

    let dsp0 = &raw_preset["data"]["tone"]["dsp0"];
    if let Some(obj) = dsp0.as_object() {
        let mut block_entries: Vec<(&String, &Value)> = obj
            .iter()
            .filter(|(k, _)| k.starts_with("block"))
            .collect();
            
        block_entries.sort_by_key(|(_, v)| v.get("@position").and_then(|p| p.as_i64()).unwrap_or(0));

        for (_, block_val) in block_entries {
            if let Some(model_str) = block_val["@model"].as_str() {
                let cab_key_opt: Option<String> = block_val.get("@cab")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let mut block = HXBlockSpec {
                    model: model_str.to_string(),
                    enabled: block_val.get("@enabled").and_then(|v| v.as_bool()),
                    path: block_val.get("@path").and_then(|v| v.as_i64()).map(|v| v as u8),
                    position: block_val.get("@position").and_then(|v| v.as_i64()).map(|v| v as u32),
                    params: Some(std::collections::HashMap::new()),
                    cab: None,
                };
                
                let model_def_opt = resolve_model(model_str).ok();
                
                if let Some(block_obj) = block_val.as_object() {
                    for (k, v) in block_obj {
                        if !k.starts_with('@') {
                            let mut include = true;
                            
                            if let Some(model_def) = &model_def_opt {
                                if let Some(param_def) = model_def.params.iter().find(|p| p.symbolic_id == *k) {
                                    match &param_def.default {
                                        NumberOrBool::Number(dn) => {
                                            if let Some(vn) = v.as_f64() {
                                                if (vn - dn).abs() < 1e-6 {
                                                    include = false;
                                                }
                                            }
                                        }
                                        NumberOrBool::Bool(db) => {
                                            if let Some(vb) = v.as_bool() {
                                                if *db == vb {
                                                    include = false;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            if include {
                                if let Some(f) = v.as_f64() {
                                    block.params.as_mut().unwrap().insert(k.clone(), NumberOrBool::Number(f));
                                } else if let Some(b) = v.as_bool() {
                                    block.params.as_mut().unwrap().insert(k.clone(), NumberOrBool::Bool(b));
                                }
                            }
                        }
                    }
                }
                
                if block.params.as_ref().map(|p| p.is_empty()).unwrap_or(false) {
                    block.params = None;
                }

                // Handle linked A+C cab slot
                if let Some(cab_key) = cab_key_opt {
                    if let Some(cab_val) = obj.get(&cab_key) {
                        if let Some(cab_model) = cab_val["@model"].as_str() {
                            let mut cab_params: std::collections::HashMap<String, NumberOrBool> = std::collections::HashMap::new();
                            if let Some(cab_obj) = cab_val.as_object() {
                                for (k, v) in cab_obj {
                                    if !k.starts_with('@') {
                                        if let Some(f) = v.as_f64() {
                                            cab_params.insert(k.clone(), NumberOrBool::Number(f));
                                        } else if let Some(b) = v.as_bool() {
                                            cab_params.insert(k.clone(), NumberOrBool::Bool(b));
                                        }
                                    }
                                }
                            }
                            block.cab = Some(CabSpec {
                                model: cab_model.to_string(),
                                params: if cab_params.is_empty() { None } else { Some(cab_params) },
                            });
                        }
                    }
                }
                
                spec.blocks.push(block);
            }
        }
    }

    Ok(spec)
}
