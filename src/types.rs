use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum NumberOrBool {
    Number(f64),
    Bool(bool),
}

impl NumberOrBool {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            NumberOrBool::Number(n) => Some(*n),
            NumberOrBool::Bool(_) => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            NumberOrBool::Bool(b) => Some(*b),
            NumberOrBool::Number(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HXParamDef {
    #[serde(rename = "symbolicID")]
    pub symbolic_id: String,
    pub name: String,
    pub min: NumberOrBool,
    pub max: NumberOrBool,
    pub default: NumberOrBool,
    #[serde(rename = "valueType")]
    pub value_type: Option<u32>,
    #[serde(rename = "displayType")]
    pub display_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HXModelDef {
    #[serde(rename = "symbolicID")]
    pub symbolic_id: String,
    pub name: String,
    pub mono: bool,
    pub stereo: bool,
    pub category: Option<u32>,
    pub params: Vec<HXParamDef>,
    pub source: Option<String>,
    pub summary: Option<String>,
}

/// An optional cab linked directly to an amp block (A+C slot mode).
/// This corresponds to the `@cab: "cab0"` pointer and sibling `cab0` key in the raw .hlx file.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CabSpec {
    pub model: String,
    pub params: Option<HashMap<String, NumberOrBool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HXBlockSpec {
    pub model: String,
    pub enabled: Option<bool>,
    pub path: Option<u8>,
    pub position: Option<u32>,
    pub params: Option<HashMap<String, NumberOrBool>>,
    /// When set, the amp and cab occupy a single A+C slot (linked via @cab in the .hlx file).
    pub cab: Option<CabSpec>,
    /// When set, creates a Dual Cab block (@type 4): this block is cab A, cabB is the second cab.
    /// Both models must be WithPan variants (HD2_CabMicIr_*WithPan). Cannot be combined with `cab`.
    #[serde(rename = "cabB")]
    pub cab_b: Option<CabSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HXPresetSpec {
    pub device: Option<String>,
    pub name: Option<String>,
    pub tempo: Option<f64>,
    pub blocks: Vec<HXBlockSpec>,
}


