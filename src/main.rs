mod catalog;
mod generator;
mod types;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use types::NumberOrBool;

#[derive(Parser)]
#[command(name = "hxgen")]
#[command(about = "Standalone CLI and Code Agent tool for generating Line 6 Helix Stomp preset files")]
#[command(long_about = "hxgen is a CLI designed for musicians and autonomous coding agents to discover Helix hardware models,
inspect parameters, and compile compact JSON specifications into complete .hlx Line 6 presets.

RECOMMENDED AI / AGENT WORKFLOW:
1. FIND GEAR: Use `hxgen list <search_query>` to find models by physical hardware names (e.g. `hxgen list marshall`). Or output JSON with `hxgen list --json`.
2. INSPECT PARAMETERS: Use `hxgen show <Model_ID>` to read the tonal summary and note the precise parameter ranges. Use `--json` for structured data.
3. UNDERSTAND FORMAT: Use `hxgen example` to see an example `.json` specification file, or `hxgen schema` to get the strict JSON Schema.
4. GENERATE: Write your JSON spec and compile it using `hxgen generate --input my-preset.json --output preset.hlx`.
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a preset from a spec
    Generate {
        /// Input spec.json file
        #[arg(short, long)]
        input: PathBuf,

        /// Output preset.hlx file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// List models in the catalog
    List {
        /// Optional category (e.g. amp, cab, drive)
        category: Option<String>,
        /// Optional query string
        query: Option<String>,
        /// Output the list as a raw JSON array instead of human-readable text
        #[arg(long)]
        json: bool,
    },
    /// Show parameters and details of a specific model
    Show {
        /// Model identifier (symbolic ID or name)
        model: String,
        /// Output the model details as raw JSON instead of human-readable text
        #[arg(long)]
        json: bool,
    },
    /// Generate an example preset spec targeting Helix Stomp
    Example {
        /// Output spec.json file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Output the strict JSON Schema for the preset specification format
    Schema,
    /// Decode a raw .hlx preset file back into a simplified JSON spec
    Decode {
        /// Input preset.hlx file
        #[arg(short, long)]
        input: PathBuf,
        /// Output spec.json file (optional, defaults to deriving from input name)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn write_example_spec(output_path: Option<PathBuf>) {
    use std::collections::HashMap;

    let mut minotaur_params: HashMap<String, NumberOrBool> = HashMap::new();
    minotaur_params.insert("Gain".to_string(), NumberOrBool::Number(0.42));
    minotaur_params.insert("Tone".to_string(), NumberOrBool::Number(0.48));
    minotaur_params.insert("Level".to_string(), NumberOrBool::Number(0.6));

    let mut brit_params: HashMap<String, NumberOrBool> = HashMap::new();
    brit_params.insert("Drive".to_string(), NumberOrBool::Number(0.58));
    brit_params.insert("Bass".to_string(), NumberOrBool::Number(0.46));
    brit_params.insert("Mid".to_string(), NumberOrBool::Number(0.57));
    brit_params.insert("Treble".to_string(), NumberOrBool::Number(0.62));
    brit_params.insert("Presence".to_string(), NumberOrBool::Number(0.56));
    brit_params.insert("ChVol".to_string(), NumberOrBool::Number(0.82));
    brit_params.insert("Master".to_string(), NumberOrBool::Number(0.74));

    let mut reverb_params: HashMap<String, NumberOrBool> = HashMap::new();
    reverb_params.insert("Decay".to_string(), NumberOrBool::Number(0.44));
    reverb_params.insert("Mix".to_string(), NumberOrBool::Number(0.22));
    reverb_params.insert("Level".to_string(), NumberOrBool::Number(0.0));

    let example = types::HXPresetSpec {
        device: Some("helix-stomp".to_string()),
        name: Some("HXGen Example".to_string()),
        tempo: Some(118.0),
        blocks: vec![
            types::HXBlockSpec {
                model: "HD2_DistMinotaur".to_string(),
                enabled: None,
                path: None,
                position: None,
                params: Some(minotaur_params),
                cab: None,
                cab_b: None,
            },
            types::HXBlockSpec {
                model: "HD2_AmpBrit2203".to_string(),
                enabled: None,
                path: None,
                position: None,
                params: Some(brit_params),
                cab: None,
                cab_b: None,
            },
            types::HXBlockSpec {
                model: "HD2_ReverbPlate".to_string(),
                enabled: None,
                path: None,
                position: None,
                params: Some(reverb_params),
                cab: None,
                cab_b: None,
            },
        ],
    };

    let target_path = output_path.unwrap_or_else(|| PathBuf::from("hxgen-example.json"));
    let mut file_content = serde_json::to_string_pretty(&example).unwrap();
    file_content.push('\n');

    fs::write(&target_path, file_content).expect("Failed to write example file");
    println!("{}", target_path.display());
}

fn list_models(category: Option<&String>, query: Option<&String>, as_json: bool) {
    let req_cat = category.and_then(|c| catalog::get_canonical_category(c));

    let full_query = match (category, query) {
        (Some(_), Some(q)) if req_cat.is_some() => q.clone(),
        (Some(c), Some(q)) if req_cat.is_none() => format!("{} {}", c, q),
        (Some(c), None) if req_cat.is_none() => c.clone(),
        _ => String::new(),
    };
    
    let needle = full_query.to_lowercase();

    let mut models: Vec<_> = catalog::HX_MODEL_CATALOG
        .iter()
        .filter(|m| {
            if let Some(cat) = req_cat {
                if !catalog::matches_category(m, cat) {
                    return false;
                }
            }
            if needle.is_empty() {
                return true;
            }
            
            let source_str = m.source.clone()
                .unwrap_or_default()
                .to_lowercase();
            
            m.symbolic_id.to_lowercase().contains(&needle)
                || m.name.to_lowercase().contains(&needle)
                || source_str.contains(&needle)
        })
        .collect();

    models.sort_by(|a, b| a.name.cmp(&b.name));

    if as_json {
        let out = serde_json::to_string_pretty(&models).unwrap();
        println!("{}", out);
        return;
    }

    for m in models {
        let source = m.source.as_ref().map(|s| format!(" | {}", s)).unwrap_or_default();
        println!("{} | {}{}", m.symbolic_id, m.name, source);
    }
}

fn show_model(input: &str, as_json: bool) {
    match catalog::resolve_model(input) {
        Ok(model) => {
            if as_json {
                let out = serde_json::to_string_pretty(&model).unwrap();
                println!("{}", out);
                return;
            }
            
            println!("{} ({})", model.name, model.symbolic_id);
            if let Some(s) = &model.source {
                println!("Source: {}", s);
            }
            if let Some(s) = &model.summary {
                println!("Summary: {}", s);
            }
            println!(
                "Mono: {} | Stereo: {}",
                if model.mono { "yes" } else { "no" },
                if model.stereo { "yes" } else { "no" }
            );
            println!();

            for param in &model.params {
                println!(
                    "- {} ({}): {}",
                    param.symbolic_id,
                    param.name,
                    catalog::format_param_range(param)
                );
                if let Some(dt) = &param.display_type {
                    if dt == "mic" {
                        println!("    Mic indices: 0=57 Dynamic, 1=409 Dynamic, 2=421 Dynamic, 3=30 Dynamic, 4=20 Dynamic, 5=121 Ribbon, 6=160 Ribbon, 7=4038 Ribbon, 8=414 Cond, 9=84 Cond, 10=67 Cond, 11=87 Cond, 12=47 Cond, 13=112 Dynamic, 14=12 Dynamic, 15=7 Dynamic");
                    } else if dt == "cabMICir" {
                        println!("    Mic indices: 0=57 Dynamic, 1=421 Dynamic, 2=7 Dynamic, 3=906 Dynamic, 4=30 Dynamic, 5=121 Ribbon, 6=160 Ribbon, 7=4038 Ribbon, 8=84 Ribbon, 9=414 Cond, 10=47 Cond FET, 11=67 Cond");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("hxgen: {}", e);
            std::process::exit(1);
        }
    }
}

fn generate_preset(input: &Path, output: Option<PathBuf>) {
    let content = match fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("hxgen: Failed to read {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };

    let spec: types::HXPresetSpec = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("hxgen: Invalid spec file {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };

    match generator::build_preset(spec.clone()) {
        Ok(preset) => {
            let mut name_safe = spec.name.unwrap_or_else(|| "HX Preset".to_string());
            let regex = regex::Regex::new(r#"[<>:"/\\|?\*\x00-\x1F]"#).unwrap();
            name_safe = regex.replace_all(&name_safe, "").to_string();
            name_safe.truncate(32);

            let out_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.hlx", name_safe)));
            
            let mut hl_json = serde_json::to_string_pretty(&preset).unwrap();
            hl_json.push('\n');

            if let Err(e) = fs::write(&out_path, hl_json) {
                eprintln!("hxgen: Failed to write {}: {}", out_path.display(), e);
                std::process::exit(1);
            }
            
            let abs_path = out_path.canonicalize().unwrap_or(out_path);
            println!("{}", abs_path.display());
        }
        Err(e) => {
            eprintln!("hxgen: {}", e);
            std::process::exit(1);
        }
    }
}

fn decode_preset(input: &Path, output: Option<PathBuf>) {
    let content = match fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("hxgen: Failed to read {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };

    let raw_preset: serde_json::Value = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("hxgen: Invalid .hlx JSON in {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };

    match generator::decode_preset(raw_preset) {
        Ok(spec) => {
            let out_path = output.unwrap_or_else(|| {
                let name = input.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "spec".to_string());
                PathBuf::from(format!("{}.json", name))
            });

            let mut hl_json = serde_json::to_string_pretty(&spec).unwrap();
            hl_json.push('\n');

            if let Err(e) = fs::write(&out_path, hl_json) {
                eprintln!("hxgen: Failed to write {}: {}", out_path.display(), e);
                std::process::exit(1);
            }

            let abs_path = out_path.canonicalize().unwrap_or(out_path);
            println!("{}", abs_path.display());
        }
        Err(e) => {
            eprintln!("hxgen: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { input, output } => {
            generate_preset(input, output.clone());
        }
        Commands::List { category, query, json } => {
            list_models(category.as_ref(), query.as_ref(), *json);
        }
        Commands::Show { model, json } => {
            show_model(model, *json);
        }
        Commands::Example { output } => {
            write_example_spec(output.clone());
        }
        Commands::Schema => {
            let schema = schemars::schema_for!(types::HXPresetSpec);
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        }
        Commands::Decode { input, output } => {
            decode_preset(input, output.clone());
        }
    }
}
