use simm_rs::file_utils::read_json_to_list;
use simm_rs::SIMM;
use simm_rs::EngineConfig;
use simm_rs::V2_5;
use serde_json::json;
use std::collections::HashMap;
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

    println!("Processing CRIF file: {}", crif_path.display());

    // Read the CRIF data from JSON
    let crif = read_json_to_list(crif_path).expect("Failed to read JSON CRIF file");
    println!("Read {} rows of CRIF data from JSON", crif.len() - 1);

    // Create WNC instance
    let wnc = V2_5;

    // Calculate SIMM
    let simm = SIMM::from_crif(crif, &cfg, &wnc)
        .expect("Failed to create SIMM calculator");

    println!("\n=== SIMM Calculation Results ===\n");

    // Extract risk class breakdown
    let breakdown: &Vec<Vec<String>> = &simm.simm_break_down;

    // Find the summary row (row with RiskType = "SIMM")
    let mut summary_row: HashMap<String, String> = HashMap::new();
    for row in breakdown.iter().skip(1) {
        if row.len() > 1 && row[1] == "SIMM" {
            // Map the values from the row
            for (i, header) in breakdown[0].iter().enumerate() {
                if i < row.len() {
                    summary_row.insert(header.to_string(), row[i].clone());
                }
            }
            break;
        }
    }

    // Create JSON output
    let output = json!({
        "SIMM Delta": summary_row.get("Delta").unwrap_or(&"-".to_string()),
        "SIMM Vega": summary_row.get("Vega").unwrap_or(&"-".to_string()),
        "SIMM Curvature": summary_row.get("Curvature").unwrap_or(&"-".to_string()),
        "SIMM Base Corr": summary_row.get("BaseCorr").unwrap_or(&"-".to_string()),
        "SIMM AddOn": summary_row.get("AddOn").unwrap_or(&"-".to_string()),
        "SIMM Benchmark": format!("{:.0}", simm.simm)
    });

    // Print JSON output
    println!("{}", serde_json::to_string_pretty(&output).unwrap());

    println!("\n=== Detailed Breakdown by Risk Class ===\n");

    // Print detailed breakdown
    for row in breakdown.iter() {
        if row.len() > 1 && !row[0].is_empty() && row[0] != "ProductClass" {
            let delta = if row.len() > 2 { &row[2] } else { "-" };
            let vega = if row.len() > 3 { &row[3] } else { "-" };
            let curvature = if row.len() > 4 { &row[4] } else { "-" };

            println!("{:15} | {:20} | Delta: {:>15} | Vega: {:>15} | Curvature: {:>15}",
                &row[0],
                &row[1],
                delta,
                vega,
                curvature
            );
        }
    }

    println!("\n=== Expected vs Actual ===");
    println!("Expected Benchmark: 131769205053");
    println!("Actual Benchmark:   {:.0}", simm.simm);
    let diff = (simm.simm - 131769205053.0).abs();
    println!("Difference:         {:.0} ({})", diff, if diff <= 1.0 { "PASS ✓" } else { "FAIL ✗" });
}
