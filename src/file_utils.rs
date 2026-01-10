use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use serde_json;

use crate::agg_margins::SIMM;
use crate::engine_config::EngineConfig;
use crate::simm_utils::{Crif, product_list};
use crate::wnc::WeightsAndCorr;

/// Read CSV file into list of lists with all values as strings
pub fn read_csv_to_list<P: AsRef<Path>>(filepath: P) -> Result<Crif> {
    let mut reader = csv::Reader::from_path(filepath)
        .context("Failed to open CSV file")?;

    let mut data = Vec::new();

    // Get headers
    let headers = reader.headers()
        .context("Failed to read CSV headers")?;
    data.push(headers.iter().map(|s| s.to_string()).collect());

    // Read all records
    for result in reader.records() {
        let record = result.context("Failed to read CSV record")?;
        data.push(record.iter().map(|s| s.to_string()).collect());
    }

    Ok(data)
}

/// Read JSON file into list of lists (CRIF format)
pub fn read_json_to_list<P: AsRef<Path>>(filepath: P) -> Result<Crif> {
    let file = File::open(filepath)
        .context("Failed to open JSON file")?;
    let reader = BufReader::new(file);

    let records: Vec<HashMap<String, String>> = serde_json::from_reader(reader)
        .context("Failed to parse JSON file")?;

    if records.is_empty() {
        return Ok(Vec::new());
    }

    // Extract headers from the first record's keys
    let headers: Vec<String> = vec![
        "ProductClass".to_string(),
        "RiskType".to_string(),
        "Qualifier".to_string(),
        "Bucket".to_string(),
        "Label1".to_string(),
        "Label2".to_string(),
        "Amount".to_string(),
        "AmountCurrency".to_string(),
        "AmountUSD".to_string(),
    ];

    let mut data = Vec::new();
    data.push(headers.clone());

    // Convert each record to a vector following header order
    for record in records {
        let row: Vec<String> = headers.iter()
            .map(|h| record.get(h).cloned().unwrap_or_default())
            .collect();
        data.push(row);
    }

    Ok(data)
}

/// Parse CSV content from a string into a Crif list
pub fn parse_csv_from_string(csv_content: &str) -> anyhow::Result<Vec<Vec<String>>> {
    let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
    let mut data = Vec::new();

    // Get headers
    let headers = reader.headers()?;
    data.push(headers.iter().map(|s| s.to_string()).collect());

    // Read all records
    for result in reader.records() {
        let record = result?;
        data.push(record.iter().map(|s| s.to_string()).collect());
    }

    Ok(data)
}

/// Convert list of lists to list of HashMaps for easier access
pub fn list_to_dict_list(data_list: &Crif) -> Vec<HashMap<String, String>> {
    if data_list.is_empty() {
        return Vec::new();
    }

    let header = &data_list[0];
    let mut result = Vec::new();

    for row in data_list.iter().skip(1) {
        let mut row_dict = HashMap::new();
        for (i, col_name) in header.iter().enumerate() {
            let value = if i < row.len() {
                row[i].clone()
            } else {
                String::new()
            };
            row_dict.insert(col_name.clone(), value);
        }
        result.push(row_dict);
    }

    result
}

/// Calculate total SIMM for a specific risk measure using product class aggregation
pub fn calculate_simm_by_measure(
    breakdown_list: &Crif,
    portfolio_crif: &Crif,
    measure_name: &str,
    wnc: &dyn WeightsAndCorr,
) -> f64 {
    let product_classes = product_list(portfolio_crif);

    // If no product classes (AddOn-only case), return 0 for all risk measures
    if product_classes.is_empty() {
        return 0.0;
    }

    // Convert breakdown to list of dicts for easier access
    let breakdown_dicts = list_to_dict_list(breakdown_list);

    let mut product_simm_values = Vec::new();

    for product_class in &product_classes {
        // Filter to this product class
        let df_product: Vec<_> = breakdown_dicts
            .iter()
            .filter(|row| row.get("Product Class").map(|s| s.as_str()) == Some(product_class))
            .collect();

        // Sum the measure value for each of the 6 risk classes
        let risk_class_list = ["Rates", "FX", "CreditQ", "CreditNonQ", "Equity", "Commodity"];
        let mut dict_risk_class_measure = HashMap::new();

        for &risk_class in &risk_class_list {
            // Filter to this risk class and measure
            let matching_rows: Vec<_> = df_product
                .iter()
                .filter(|row| {
                    row.get("Risk Class").map(|s| s.as_str()) == Some(risk_class)
                        && row.get("Risk Measure").map(|s| s.as_str()) == Some(measure_name)
                })
                .collect();

            let total = if !matching_rows.is_empty() {
                let mut sum = 0.0;
                for row in matching_rows {
                    if let Some(val) = row.get("SIMM_RiskMeasure") {
                        if !val.is_empty() && val != "-" {
                            if let Ok(num) = val.parse::<f64>() {
                                sum += num;
                            }
                        }
                    }
                }
                sum
            } else {
                0.0
            };

            dict_risk_class_measure.insert(risk_class, total);
        }

        // Aggregate across risk classes using psi correlations (same as simm_product)
        let mut simm_product = 0.0;
        for i in 0..6 {
            for j in 0..6 {
                let psi = if i == j {
                    1.0
                } else {
                    wnc.psi(risk_class_list[i], risk_class_list[j])
                        .unwrap_or(0.0)
                };

                simm_product += psi
                    * dict_risk_class_measure
                        .get(risk_class_list[i])
                        .copied()
                        .unwrap_or(0.0)
                    * dict_risk_class_measure
                        .get(risk_class_list[j])
                        .copied()
                        .unwrap_or(0.0);
            }
        }

        product_simm_values.push(simm_product.sqrt());
    }

    // Sum across all product classes (no correlation between products)
    product_simm_values.iter().sum()
}

/// Format value - use hyphen for zero values
fn _format_value(val: f64) -> String {
    if val == 0.0 {
        "-".to_string()
    } else {
        // Round using ROUND_HALF_UP behavior
        let rounded = (val + 0.5).floor() as i64;
        rounded.to_string()
    }
}

/// Process a CRIF file and generate summary output
pub fn process_crif_file<P: AsRef<Path>>(
    crif_path: P,
    output_path: P,
    cfg: &EngineConfig,
) -> Result<HashMap<String, String>> {
    // Read CRIF as list of lists
    let crif = read_csv_to_list(&crif_path)?;

    // Calculate SIMM using the correct version from config
    let wnc = crate::load_wnc(cfg);
    let portfolio = SIMM::from_crif(crif.clone(), cfg, wnc.as_ref())?;

    // Get breakdown (list of lists)
    let breakdown_list = &portfolio.simm_break_down;

    // Calculate totals for each measure
    let delta_total = calculate_simm_by_measure(breakdown_list, &crif, "Delta", wnc.as_ref());
    let vega_total = calculate_simm_by_measure(breakdown_list, &crif, "Vega", wnc.as_ref());
    let curvature_total = calculate_simm_by_measure(breakdown_list, &crif, "Curvature", wnc.as_ref());
    let basecorr_total = calculate_simm_by_measure(breakdown_list, &crif, "BaseCorr", wnc.as_ref());

    // Get AddOn if it exists
    let mut addon_total = 0.0;
    if breakdown_list.len() > 1 {
        // Has data rows
        let header = &breakdown_list[0];
        if let Some(addon_idx) = header.iter().position(|s| s == "Add-On") {
            for row in breakdown_list.iter().skip(1) {
                if addon_idx < row.len() && !row[addon_idx].is_empty() {
                    if let Ok(val) = row[addon_idx].parse::<f64>() {
                        addon_total = val;
                        break;
                    }
                }
            }
        }
    }

    // Use the actual calculated SIMM as benchmark
    let benchmark = portfolio.simm;

    // Create summary output
    let mut summary_data = HashMap::new();
    summary_data.insert("SIMM Delta".to_string(), _format_value(delta_total));
    summary_data.insert("SIMM Vega".to_string(), _format_value(vega_total));
    summary_data.insert("SIMM Curvature".to_string(), _format_value(curvature_total));
    summary_data.insert("SIMM Base Corr".to_string(), _format_value(basecorr_total));
    summary_data.insert("SIMM AddOn".to_string(), _format_value(addon_total));
    summary_data.insert("SIMM Benchmark".to_string(), _format_value(benchmark));

    // Write summary to CSV
    let mut file = File::create(output_path.as_ref())
        .context("Failed to create output file")?;

    // Write header
    writeln!(
        file,
        "SIMM Delta,SIMM Vega,SIMM Curvature,SIMM Base Corr,SIMM AddOn,SIMM Benchmark"
    )?;

    // Write values
    writeln!(
        file,
        "{},{},{},{},{},{}",
        summary_data.get("SIMM Delta").unwrap(),
        summary_data.get("SIMM Vega").unwrap(),
        summary_data.get("SIMM Curvature").unwrap(),
        summary_data.get("SIMM Base Corr").unwrap(),
        summary_data.get("SIMM AddOn").unwrap(),
        summary_data.get("SIMM Benchmark").unwrap()
    )?;

    Ok(summary_data)
}

/// Compare calculated output with expected output
/// Allows for rounding tolerance of 1 for large numbers (> 1 billion)
pub fn compare_csv_files<P: AsRef<Path>>(calc_path: P, expected_path: P) -> Result<(bool, Vec<String>)> {
    let calc_file = File::open(calc_path.as_ref())
        .context("Failed to open calculated output file")?;
    let expected_file = File::open(expected_path.as_ref())
        .context("Failed to open expected output file")?;

    let calc_lines: Vec<String> = BufReader::new(calc_file)
        .lines()
        .collect::<std::io::Result<_>>()?;
    let expected_lines: Vec<String> = BufReader::new(expected_file)
        .lines()
        .collect::<std::io::Result<_>>()?;

    // Filter out empty trailing lines
    let calc_lines: Vec<&str> = calc_lines.iter().map(|s| s.as_str()).filter(|s| !s.is_empty()).collect();
    let expected_lines: Vec<&str> = expected_lines.iter().map(|s| s.as_str()).filter(|s| !s.is_empty()).collect();

    if calc_lines == expected_lines {
        return Ok((true, Vec::new()));
    }

    let mut differences = Vec::new();
    let max_len = calc_lines.len().max(expected_lines.len());

    for i in 0..max_len {
        let calc = calc_lines.get(i).copied().unwrap_or("");
        let expected = expected_lines.get(i).copied().unwrap_or("");

        if calc != expected {
            // For data lines (not header), check if difference is just rounding
            if i > 0 && !calc.is_empty() && !expected.is_empty() {
                if _lines_match_with_tolerance(calc, expected) {
                    continue; // Accept small rounding differences
                }
            }
            differences.push(format!(
                "Line {}:\n  Expected: {}\n  Got:      {}",
                i + 1,
                expected,
                calc
            ));
        }
    }

    Ok((differences.is_empty(), differences))
}

/// Check if two CSV lines match within rounding tolerance
/// Allows difference of 1 for numbers > 1 billion
fn _lines_match_with_tolerance(calc: &str, expected: &str) -> bool {
    let calc_parts: Vec<&str> = calc.split(',').collect();
    let expected_parts: Vec<&str> = expected.split(',').collect();

    if calc_parts.len() != expected_parts.len() {
        return false;
    }

    for (c, e) in calc_parts.iter().zip(expected_parts.iter()) {
        if c == e {
            continue;
        }

        // Try to parse as numbers
        if let (Ok(calc_num), Ok(exp_num)) = (c.parse::<i64>(), e.parse::<i64>()) {
            let diff = (calc_num - exp_num).abs();
            // Allow difference of 1 for large numbers (> 1 billion)
            if exp_num.abs() > 1_000_000_000 && diff <= 1 {
                continue;
            }
            // For smaller numbers, must match exactly
            if diff == 0 {
                continue;
            }
            return false;
        }

        // If not numbers or don't match, return false
        if c != e {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv_to_list() {
        let crif = read_csv_to_list("tests_2_5/C1_crif.csv").unwrap();
        assert!(!crif.is_empty());
        assert_eq!(crif[0][0], "ProductClass");
    }

    #[test]
    fn test_list_to_dict_list() {
        let crif = vec![
            vec!["ProductClass".to_string(), "RiskType".to_string()],
            vec!["Rates".to_string(), "Risk_IRCurve".to_string()],
        ];

        let dicts = list_to_dict_list(&crif);
        assert_eq!(dicts.len(), 1);
        assert_eq!(dicts[0].get("ProductClass").unwrap(), "Rates");
    }
}
    #[test]
    fn test_product_list_c99()  {
        use crate::simm_utils::product_list;
        use crate::file_utils::read_csv_to_list;

        let crif = read_csv_to_list("tests_2_5/C99_crif.csv").unwrap();
        let products = product_list(&crif);

        println!("Product list for C99: {:?}", products);
        assert!(!products.is_empty(), "Product list should not be empty!");
    }

    #[test]
    fn test_c99_debug() {
        use crate::v2_5::V2_5;

        let cfg = EngineConfig {
            weights_and_corr_version: "2_5".to_string(),
            calculation_currency: "USD".to_string(),
            exchange_rate: 1.0,
        };

        let crif = read_csv_to_list("tests_2_5/C99_crif.csv").unwrap();

        println!("\nCRIF Data:");
        for (i, row) in crif.iter().enumerate() {
            println!("  {}: {:?}", i, row);
        }

        let wnc = V2_5;
        let portfolio = SIMM::from_crif(crif.clone(), &cfg, &wnc).unwrap();

        println!("\nSIMM Total: {}", portfolio.simm);

        println!("\nBreakdown:");
        for (i, row) in portfolio.simm_break_down.iter().enumerate() {
            println!("  {}: {:?}", i, row);
        }

        let delta_total = calculate_simm_by_measure(&portfolio.simm_break_down, &crif, "Delta", &wnc);
        println!("\nDelta Total: {}", delta_total);
    }

    #[test]
    fn test_c99_margin_debug() {
        use crate::margin_risk_class::MarginByRiskClass;
        use crate::v2_5::V2_5;

        let crif = read_csv_to_list("tests_2_5/C99_crif.csv").unwrap();

        let wnc = V2_5;
        let margin = MarginByRiskClass::new(crif.clone(), "USD".to_string(), &wnc);

        println!("\nlist_risk_types: {:?}", margin.list_risk_types);

        let delta = margin.delta_margin();
        println!("\nDelta margins:");
        for (risk_class, measures) in &delta {
            println!("  {}: {:?}", risk_class, measures);
        }

        let ir_delta = margin.ir_delta_margin();
        println!("\nIR Delta margins:");
        for (risk_class, measures) in &ir_delta {
            println!("  {}: {:?}", risk_class, measures);
        }
    }

    #[test]
    fn test_c298_specific() {
        let cfg = EngineConfig {
            weights_and_corr_version: "2_5".to_string(),
            calculation_currency: "USD".to_string(),
            exchange_rate: 1.0,
        };

        let crif_path = "tests_2_5/C298_crif.csv";
        let calc_output_path = "tests_2_5/C298_calc_output_test.csv";
        let expected_output_path = "tests_2_5/C298_expected_output.csv";

        // Process the CRIF file
        let summary = process_crif_file(crif_path, calc_output_path, &cfg)
            .expect("Failed to process CRIF file");

        println!("\nC298 Summary:");
        for (k, v) in &summary {
            println!("  {}: {}", k, v);
        }

        // Compare with expected output
        let (matches, differences) = compare_csv_files(calc_output_path, expected_output_path)
            .expect("Failed to compare files");

        println!("\nMatches: {}", matches);
        if !matches {
            println!("Differences:");
            for diff in &differences {
                println!("{}", diff);
            }
        }
    }
