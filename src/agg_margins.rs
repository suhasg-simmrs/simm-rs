use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::constants::margin_by_risk_class;
use crate::engine_config::EngineConfig;
use crate::margin_risk_class::{MarginByRiskClass, filter_rows};
use crate::simm_utils::{Crif, get_column_index, product_list};
use crate::wnc::WeightsAndCorr;

/// Main SIMM calculator
pub struct SIMM<'a> {
    crif_list: Crif,
    pub simm: f64,
    pub simm_break_down: Crif,
    calc_currency: String,
    exchange_rate: f64,
    wnc: &'a dyn WeightsAndCorr,
}

impl<'a> SIMM<'a> {
    /// Create SIMM calculator from CRIF list
    ///
    /// # Arguments
    /// * `crif` - List of lists where first row is header, subsequent rows are data
    /// * `cfg` - Engine configuration (calculation currency, exchange rate, etc.)
    /// * `wnc` - Weights and correlations implementation
    pub fn from_crif(crif: Crif, cfg: &EngineConfig, wnc: &'a dyn WeightsAndCorr) -> Result<Self> {
        if crif.is_empty() {
            return Err(anyhow::anyhow!("crif list must have at least a header row"));
        }

        let mut simm = SIMM {
            crif_list: crif,
            simm: 0.0,
            simm_break_down: Vec::new(),
            calc_currency: cfg.calculation_currency.clone(),
            exchange_rate: cfg.exchange_rate,
            wnc,
        };

        simm.calculate_simm()?;
        Ok(simm)
    }

    /// Create SIMM calculator from CSV file
    ///
    /// # Arguments
    /// * `csv_path` - Path to CSV file
    /// * `cfg` - Engine configuration
    /// * `wnc` - Weights and correlations implementation
    pub fn from_csv<P: AsRef<Path>>(
        csv_path: P,
        cfg: &EngineConfig,
        wnc: &'a dyn WeightsAndCorr,
    ) -> Result<Self> {
        let mut reader = csv::Reader::from_path(csv_path)
            .context("Failed to open CSV file")?;

        let mut crif_list: Vec<Vec<String>> = Vec::new();

        // Get headers
        let headers = reader.headers()
            .context("Failed to read CSV headers")?;
        crif_list.push(headers.iter().map(|s| s.to_string()).collect());

        // Read all records
        for result in reader.records() {
            let record = result.context("Failed to read CSV record")?;
            crif_list.push(record.iter().map(|s| s.to_string()).collect());
        }

        Self::from_crif(crif_list, cfg, wnc)
    }

    /// Helper method to filter CRIF by a single column value
    ///
    /// # Arguments
    /// * `column_name` - Name of the column to filter by
    /// * `value` - Value to match
    ///
    /// # Returns
    /// Filtered CRIF data as list of lists [header, row1, row2, ...]
    fn filter_crif_by_column(&self, column_name: &str, value: &str) -> Crif {
        let mut conditions = HashMap::new();
        conditions.insert(column_name.to_string(), vec![value.to_string()]);
        filter_rows(&self.crif_list, &conditions)
    }

    /// Calculate margin by risk class
    ///
    /// # Arguments
    /// * `crif` - CRIF data
    ///
    /// # Returns
    /// Dict of margins by risk class and measure
    fn simm_risk_class(&self, crif: &Crif) -> HashMap<String, HashMap<String, f64>> {
        let margin = MarginByRiskClass::new(crif.clone(), self.calc_currency.clone(), self.wnc);

        // Get results from each margin calculation
        let ir_delta = margin.ir_delta_margin();
        let delta = margin.delta_margin();
        let ir_vega = margin.ir_vega_margin();
        let vega = margin.vega_margin();
        let ir_curvature = margin.ir_curvature_margin();
        let curvature = margin.curvature_margin();
        let base_corr = margin.base_corr_margin();

        // Sum all the margin components
        let dict_margin = margin_by_risk_class();
        let mut df_margin_aggregated: HashMap<String, HashMap<String, f64>> = HashMap::new();

        for (risk_class, measures) in &dict_margin {
            let mut measure_map = HashMap::new();
            for measure in measures {
                let total = ir_delta
                    .get(*risk_class)
                    .and_then(|m| m.get(*measure))
                    .copied()
                    .unwrap_or(0.0)
                    + delta
                        .get(*risk_class)
                        .and_then(|m| m.get(*measure))
                        .copied()
                        .unwrap_or(0.0)
                    + ir_vega
                        .get(*risk_class)
                        .and_then(|m| m.get(*measure))
                        .copied()
                        .unwrap_or(0.0)
                    + vega
                        .get(*risk_class)
                        .and_then(|m| m.get(*measure))
                        .copied()
                        .unwrap_or(0.0)
                    + ir_curvature
                        .get(*risk_class)
                        .and_then(|m| m.get(*measure))
                        .copied()
                        .unwrap_or(0.0)
                    + curvature
                        .get(*risk_class)
                        .and_then(|m| m.get(*measure))
                        .copied()
                        .unwrap_or(0.0)
                    + base_corr
                        .get(*risk_class)
                        .and_then(|m| m.get(*measure))
                        .copied()
                        .unwrap_or(0.0);

                measure_map.insert(measure.to_string(), total);
            }
            df_margin_aggregated.insert(risk_class.to_string(), measure_map);
        }

        // BaseCorr only presents in the CreditQ
        for (risk_class, measures) in df_margin_aggregated.iter_mut() {
            if risk_class != "CreditQ" {
                measures.remove("BaseCorr");
            }
        }

        // Apply exchange rate
        for measures in df_margin_aggregated.values_mut() {
            for value in measures.values_mut() {
                *value *= self.exchange_rate;
            }
        }

        df_margin_aggregated
    }

    /// Calculate SIMM for a product class
    ///
    /// # Arguments
    /// * `product_class` - Product class name
    ///
    /// # Returns
    /// SIMM value for the product class
    fn simm_product(&self, product_class: &str) -> Result<f64> {
        let crif = self.filter_crif_by_column("ProductClass", product_class);
        let simm_by_risk_class = self.simm_risk_class(&crif);

        let risk_class_list = ["Rates", "FX", "CreditQ", "CreditNonQ", "Equity", "Commodity"];
        let mut dict_simm_risk_class = HashMap::new();

        for risk_class in &risk_class_list {
            let simm_rc: f64 = simm_by_risk_class
                .get(*risk_class)
                .map(|m| m.values().sum())
                .unwrap_or(0.0);
            dict_simm_risk_class.insert(*risk_class, simm_rc);
        }

        let mut simm_product = 0.0;
        for i in 0..6 {
            for j in 0..6 {
                let psi = if i == j {
                    1.0
                } else {
                    self.wnc
                        .psi(risk_class_list[i], risk_class_list[j])
                        .unwrap_or(0.0)
                };

                simm_product += psi
                    * dict_simm_risk_class
                        .get(risk_class_list[i])
                        .copied()
                        .unwrap_or(0.0)
                    * dict_simm_risk_class
                        .get(risk_class_list[j])
                        .copied()
                        .unwrap_or(0.0);
            }
        }

        Ok(simm_product.sqrt())
    }

    /// Calculate detailed results for a product class
    ///
    /// # Arguments
    /// * `product_class` - Product class name
    ///
    /// # Returns
    /// List of result rows
    fn results_product_class(&self, product_class: &str) -> Result<Vec<HashMap<String, String>>> {
        let crif = self.filter_crif_by_column("ProductClass", product_class);
        let dict_results = self.simm_risk_class(&crif);

        let mut result_rows = Vec::new();

        for (risk_class, dic_sensi_type) in &dict_results {
            let values: Vec<f64> = dic_sensi_type.values().copied().collect();
            let all_zeros = values.iter().all(|&v| v == 0.0);

            if !all_zeros {
                for (risk_measure, &value) in dic_sensi_type {
                    let mut row = HashMap::new();
                    row.insert("Product Class".to_string(), product_class.to_string());
                    row.insert("Risk Class".to_string(), risk_class.clone());
                    row.insert("Risk Measure".to_string(), risk_measure.clone());
                    row.insert("SIMM_RiskMeasure".to_string(), format!("{:.2}", value));
                    result_rows.push(row);
                }

                // Calculate risk class total
                let im_risk_class: f64 = values.iter().sum();
                let mut row = HashMap::new();
                row.insert("Product Class".to_string(), product_class.to_string());
                row.insert("Risk Class".to_string(), risk_class.clone());
                row.insert("SIMM_RiskClass".to_string(), format!("{:.2}", im_risk_class));
                result_rows.push(row);
            }
        }

        Ok(result_rows)
    }

    /// Calculate add-on margin
    ///
    /// # Returns
    /// Add-on margin value
    fn addon_margin(&self) -> Result<f64> {
        let amount_idx = get_column_index(&self.crif_list, "AmountUSD")
            .ok_or_else(|| anyhow::anyhow!("AmountUSD column not found"))?;
        let risk_type_idx = get_column_index(&self.crif_list, "RiskType")
            .ok_or_else(|| anyhow::anyhow!("RiskType column not found"))?;
        let qualifier_idx = get_column_index(&self.crif_list, "Qualifier")
            .ok_or_else(|| anyhow::anyhow!("Qualifier column not found"))?;

        // Fixed addon
        let fixed: f64 = self.crif_list
            .iter()
            .skip(1) // Skip header
            .filter(|row| {
                risk_type_idx < row.len() &&
                row[risk_type_idx] == "Param_AddOnFixedAmount"
            })
            .filter_map(|row| {
                if amount_idx < row.len() {
                    row[amount_idx].parse::<f64>().ok()
                } else {
                    None
                }
            })
            .sum();

        // Factor * Notional per qualifier
        let mut qualifier_map: HashMap<String, (f64, f64)> = HashMap::new();

        for row in self.crif_list.iter().skip(1) {
            if risk_type_idx >= row.len() || qualifier_idx >= row.len() || amount_idx >= row.len() {
                continue;
            }

            let risk_type = &row[risk_type_idx];
            if risk_type != "Param_AddOnNotionalFactor" && risk_type != "Notional" {
                continue;
            }

            let qualifier = &row[qualifier_idx];
            let amount = row[amount_idx].parse::<f64>().unwrap_or(0.0);

            let entry = qualifier_map.entry(qualifier.clone()).or_insert((0.0, 0.0));

            if risk_type == "Param_AddOnNotionalFactor" {
                entry.0 += amount / 100.0; // factor
            } else if risk_type == "Notional" {
                entry.1 += amount; // notional
            }
        }

        let mut addon = fixed;
        for (factor, notional) in qualifier_map.values() {
            addon += factor * notional;
        }

        Ok(addon)
    }

    /// Main SIMM calculation
    fn calculate_simm(&mut self) -> Result<()> {
        let mut addon_ms = 0.0;

        // Get distinct product classes using utility function
        let product_classes = product_list(&self.crif_list);

        let mut all_results = Vec::new();

        for product_class in &product_classes {
            let result_rows = self.results_product_class(product_class)?;
            let simm_prod = self.simm_product(product_class)?;

            // Add SIMM_ProductClass to each row
            for mut row in result_rows {
                row.insert(
                    "SIMM_ProductClass".to_string(),
                    format!("{:.2}", simm_prod),
                );
                all_results.push(row);
            }

            self.simm += simm_prod;

            // Check for product class multiplier
            let amount_idx = get_column_index(&self.crif_list, "AmountUSD").unwrap();
            let risk_type_idx = get_column_index(&self.crif_list, "RiskType").unwrap();
            let qualifier_idx = get_column_index(&self.crif_list, "Qualifier").unwrap();

            let ms_result: f64 = self.crif_list
                .iter()
                .skip(1)
                .filter(|row| {
                    risk_type_idx < row.len() &&
                    qualifier_idx < row.len() &&
                    row[risk_type_idx] == "Param_ProductClassMultiplier" &&
                    &row[qualifier_idx] == product_class
                })
                .filter_map(|row| {
                    if amount_idx < row.len() {
                        row[amount_idx].parse::<f64>().ok()
                    } else {
                        None
                    }
                })
                .sum();

            if ms_result != 0.0 {
                let ms = ms_result - 1.0;
                addon_ms += simm_prod * ms;
            }
        }

        let addon_margin = (addon_ms + self.addon_margin()?).round() * 100.0 / 100.0; // round to 2 decimals
        self.simm += addon_margin;

        // Handle case where there are no product classes (only AddOn data)
        let has_real_products = !product_classes.is_empty();

        if !has_real_products {
            // Create minimal structure for AddOn-only case
            if addon_margin.abs() > 0.0 {
                let header = vec![
                    "SIMM Total".to_string(),
                    "Add-On".to_string(),
                    "Product Class".to_string(),
                    "SIMM_ProductClass".to_string(),
                    "Risk Class".to_string(),
                    "SIMM_RiskClass".to_string(),
                    "Risk Measure".to_string(),
                    "SIMM_RiskMeasure".to_string(),
                ];
                let row = vec![
                    format!("{:.2}", self.simm),
                    format!("{:.2}", addon_margin),
                    "".to_string(),
                    "0".to_string(),
                    "".to_string(),
                    "0".to_string(),
                    "".to_string(),
                    "0".to_string(),
                ];
                self.simm_break_down = vec![header, row];
            } else {
                let header = vec![
                    "SIMM Total".to_string(),
                    "Product Class".to_string(),
                    "SIMM_ProductClass".to_string(),
                    "Risk Class".to_string(),
                    "SIMM_RiskClass".to_string(),
                    "Risk Measure".to_string(),
                    "SIMM_RiskMeasure".to_string(),
                ];
                let row = vec![
                    format!("{:.2}", self.simm),
                    "".to_string(),
                    "0".to_string(),
                    "".to_string(),
                    "0".to_string(),
                    "".to_string(),
                    "0".to_string(),
                ];
                self.simm_break_down = vec![header, row];
            }
        } else {
            // Build breakdown as list of lists
            if addon_margin.abs() > 0.0 {
                let header = vec![
                    "SIMM Total".to_string(),
                    "Add-On".to_string(),
                    "Product Class".to_string(),
                    "SIMM_ProductClass".to_string(),
                    "Risk Class".to_string(),
                    "SIMM_RiskClass".to_string(),
                    "Risk Measure".to_string(),
                    "SIMM_RiskMeasure".to_string(),
                ];
                let mut rows = Vec::new();
                for result in &all_results {
                    let row = vec![
                        format!("{:.2}", self.simm),
                        format!("{:.2}", addon_margin),
                        result.get("Product Class").cloned().unwrap_or_default(),
                        result.get("SIMM_ProductClass").cloned().unwrap_or_default(),
                        result.get("Risk Class").cloned().unwrap_or_default(),
                        result
                            .get("SIMM_RiskClass")
                            .cloned()
                            .unwrap_or_else(|| "".to_string()),
                        result
                            .get("Risk Measure")
                            .cloned()
                            .unwrap_or_else(|| "".to_string()),
                        result
                            .get("SIMM_RiskMeasure")
                            .cloned()
                            .unwrap_or_else(|| "".to_string()),
                    ];
                    rows.push(row);
                }
                self.simm_break_down = vec![header];
                self.simm_break_down.extend(rows);
            } else {
                let header = vec![
                    "SIMM Total".to_string(),
                    "Product Class".to_string(),
                    "SIMM_ProductClass".to_string(),
                    "Risk Class".to_string(),
                    "SIMM_RiskClass".to_string(),
                    "Risk Measure".to_string(),
                    "SIMM_RiskMeasure".to_string(),
                ];
                let mut rows = Vec::new();
                for result in &all_results {
                    let row = vec![
                        format!("{:.2}", self.simm),
                        result.get("Product Class").cloned().unwrap_or_default(),
                        result.get("SIMM_ProductClass").cloned().unwrap_or_default(),
                        result.get("Risk Class").cloned().unwrap_or_default(),
                        result
                            .get("SIMM_RiskClass")
                            .cloned()
                            .unwrap_or_else(|| "".to_string()),
                        result
                            .get("Risk Measure")
                            .cloned()
                            .unwrap_or_else(|| "".to_string()),
                        result
                            .get("SIMM_RiskMeasure")
                            .cloned()
                            .unwrap_or_else(|| "".to_string()),
                    ];
                    rows.push(row);
                }
                self.simm_break_down = vec![header];
                self.simm_break_down.extend(rows);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2_5::V2_5;

    #[test]
    fn test_simm_basic() {
        let crif = vec![
            vec![
                "ProductClass".to_string(),
                "RiskType".to_string(),
                "Qualifier".to_string(),
                "Bucket".to_string(),
                "Label1".to_string(),
                "AmountUSD".to_string(),
            ],
            vec![
                "Rates".to_string(),
                "Risk_IRCurve".to_string(),
                "USD".to_string(),
                "1".to_string(),
                "2w".to_string(),
                "1000".to_string(),
            ],
        ];

        let cfg = EngineConfig {
            weights_and_corr_version: "2_5".to_string(),
            calculation_currency: "USD".to_string(),
            exchange_rate: 1.0,
        };

        let wnc = V2_5;
        let simm = SIMM::from_crif(crif, &cfg, &wnc);
        assert!(simm.is_ok());

        let simm = simm.unwrap();
        assert!(simm.simm >= 0.0);
        assert!(!simm.simm_break_down.is_empty());
    }

    #[test]
    fn test_simm_comprehensive() {
        // Create a more comprehensive CRIF with multiple risk types
        let crif = vec![
            vec![
                "ProductClass".to_string(),
                "RiskType".to_string(),
                "Qualifier".to_string(),
                "Bucket".to_string(),
                "Label1".to_string(),
                "Label2".to_string(),
                "AmountUSD".to_string(),
                "AmountCurrency".to_string(),
            ],
            // IR Curve Delta
            vec![
                "Rates".to_string(),
                "Risk_IRCurve".to_string(),
                "USD".to_string(),
                "1".to_string(),
                "2w".to_string(),
                "".to_string(),
                "10000".to_string(),
                "USD".to_string(),
            ],
            vec![
                "Rates".to_string(),
                "Risk_IRCurve".to_string(),
                "USD".to_string(),
                "1".to_string(),
                "1y".to_string(),
                "".to_string(),
                "20000".to_string(),
                "USD".to_string(),
            ],
            // FX Delta
            vec![
                "FX".to_string(),
                "Risk_FX".to_string(),
                "EURUSD".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "5000".to_string(),
                "USD".to_string(),
            ],
        ];

        let cfg = EngineConfig {
            weights_and_corr_version: "2_5".to_string(),
            calculation_currency: "USD".to_string(),
            exchange_rate: 1.0,
        };

        let wnc = V2_5;
        let simm = SIMM::from_crif(crif, &cfg, &wnc).unwrap();

        // Verify SIMM was calculated
        assert!(simm.simm > 0.0);

        // Verify breakdown has header and data
        assert!(simm.simm_break_down.len() > 1);

        // Verify header structure
        let header = &simm.simm_break_down[0];
        assert!(header.contains(&"SIMM Total".to_string()));
        assert!(header.contains(&"Product Class".to_string()));
        assert!(header.contains(&"Risk Class".to_string()));
        assert!(header.contains(&"Risk Measure".to_string()));
    }
}
