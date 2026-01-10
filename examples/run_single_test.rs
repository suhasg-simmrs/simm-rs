
use simm_rs::file_utils::{read_json_to_list, calculate_simm_by_measure};
use simm_rs::SIMM;
use simm_rs::EngineConfig;
use simm_rs::V2_5;
use std::path::PathBuf;

fn main() {
    // Create configuration
    let cfg = EngineConfig {
        weights_and_corr_version: "2_5".to_string(),
        calculation_currency: "USD".to_string(),
        exchange_rate: 1.0,
    };

    // Read C298 CRIF from JSON file (relative to project root)
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crif_path = project_root.join("tests_2_5").join("C298_crif.json");

    // Read the CRIF data from JSON
    let crif = read_json_to_list(crif_path).expect("Failed to read JSON CRIF file");

    // Create WNC instance
    let wnc = V2_5;

    // Clone CRIF for later use
    let crif_for_measure = crif.clone();

    // Calculate SIMM
    let simm = SIMM::from_crif(crif, &cfg, &wnc)
        .expect("Failed to create SIMM calculator");

    // Extract risk class breakdown
    let breakdown = &simm.simm_break_down;

    // Calculate totals for each measure using the proper aggregation method
    let delta_total = calculate_simm_by_measure(breakdown, &crif_for_measure, "Delta", &wnc);
    let vega_total = calculate_simm_by_measure(breakdown, &crif_for_measure, "Vega", &wnc);
    let curvature_total = calculate_simm_by_measure(breakdown, &crif_for_measure, "Curvature", &wnc);
    let basecorr_total = calculate_simm_by_measure(breakdown, &crif_for_measure, "BaseCorr", &wnc);

    // Get AddOn if it exists
    let mut addon_total = 0.0;
    if breakdown.len() > 1 {
        let header = &breakdown[0];
        if let Some(addon_idx) = header.iter().position(|s| s == "Add-On") {
            for row in breakdown.iter().skip(1) {
                if addon_idx < row.len() && !row[addon_idx].is_empty() {
                    if let Ok(val) = row[addon_idx].parse::<f64>() {
                        addon_total = val;
                        break;
                    }
                }
            }
        }
    }

    // Format values - use "-" for zero values, round non-zero values
    let format_value = |v: f64| -> String {
        if v == 0.0 {
            "-".to_string()
        } else {
            // Round using ROUND_HALF_UP behavior
            let rounded = (v + 0.5).floor() as i64;
            rounded.to_string()
        }
    };

    // Print CSV header
    println!("SIMM Delta,SIMM Vega,SIMM Curvature,SIMM Base Corr,SIMM AddOn,SIMM Benchmark");

    // Print CSV values
    println!("{},{},{},{},{},{}",
        format_value(delta_total),
        format_value(vega_total),
        format_value(curvature_total),
        format_value(basecorr_total),
        format_value(addon_total),
        format_value(simm.simm)
    );

    // Print detailed breakdown
    println!("\n=== Detailed Breakdown ===\n");

    // Print breakdown header
    if let Some(header) = breakdown.first() {
        println!("{}", header.join(" | "));
        println!("{}", "-".repeat(150));
    }

    // Print breakdown rows
    for row in breakdown.iter().skip(1) {
        println!("{}", row.join(" | "));
    }
}
