use std::collections::HashMap;
use statrs::distribution::{ContinuousCDF, Normal};

use crate::agg_sensitivities::{k_delta, k_vega, k_curvature};
use crate::constants::*;
use crate::simm_utils::{self, Crif};
use crate::wnc::WeightsAndCorr;
use crate::v2_5::*;

/// Initialize margin dictionary with zeros
pub fn init_margin_dict() -> HashMap<String, HashMap<String, f64>> {
    let mut result = HashMap::new();

    for risk_class in &["Rates", "CreditQ", "CreditNonQ", "Equity", "Commodity", "FX"] {
        let mut measures = HashMap::new();
        measures.insert("Delta".to_string(), 0.0);
        measures.insert("Vega".to_string(), 0.0);
        measures.insert("Curvature".to_string(), 0.0);
        if *risk_class == "CreditQ" {
            measures.insert("BaseCorr".to_string(), 0.0);
        }
        result.insert(risk_class.to_string(), measures);
    }

    result
}

/// Get column index from CRIF
pub fn get_column_index(crif: &Crif, column_name: &str) -> Option<usize> {
    simm_utils::get_column_index(crif, column_name)
}

/// Get value from a row by column name
#[allow(dead_code)]
pub fn get_column_value(row: &[String], crif: &Crif, column_name: &str) -> Option<String> {
    let idx = get_column_index(crif, column_name)?;
    if idx < row.len() {
        Some(row[idx].clone())
    } else {
        None
    }
}

/// Filter rows based on conditions
pub fn filter_rows(crif: &Crif, conditions: &HashMap<String, Vec<String>>) -> Crif {
    if crif.len() <= 1 {
        return if !crif.is_empty() {
            vec![crif[0].clone()]
        } else {
            Vec::new()
        };
    }

    let mut result = vec![crif[0].clone()];

    // Get column indices for each condition
    let mut col_indices = HashMap::new();
    for (col_name, _) in conditions {
        if let Some(idx) = get_column_index(crif, col_name) {
            col_indices.insert(col_name.clone(), idx);
        }
    }

    // Filter rows
    for row in crif.iter().skip(1) {
        let mut matches = true;
        for (col_name, expected_values) in conditions {
            if let Some(&idx) = col_indices.get(col_name) {
                if idx >= row.len() {
                    matches = false;
                    break;
                }

                let actual_value = &row[idx];
                if !expected_values.contains(actual_value) {
                    matches = false;
                    break;
                }
            } else {
                matches = false;
                break;
            }
        }

        if matches {
            result.push(row.clone());
        }
    }

    result
}

/// Get unique values from a column
pub fn unique_values(crif: &Crif, column_name: &str) -> Vec<String> {
    simm_utils::unique_column_values(crif, column_name)
}

/// Sum all values in a column
pub fn sum_column(crif: &Crif, _column_name: &str) -> f64 {
    simm_utils::sum_sensitivities(crif)
}

/// Drop rows that match ALL conditions
pub fn drop_rows(crif: &Crif, conditions: &HashMap<String, String>) -> Crif {
    if crif.len() <= 1 {
        return if !crif.is_empty() {
            vec![crif[0].clone()]
        } else {
            Vec::new()
        };
    }

    let mut result = vec![crif[0].clone()];

    // Get column indices
    let mut col_indices = HashMap::new();
    for (col_name, _) in conditions {
        if let Some(idx) = get_column_index(crif, col_name) {
            col_indices.insert(col_name.clone(), idx);
        }
    }

    // Keep rows that don't match ALL conditions
    for row in crif.iter().skip(1) {
        let mut all_match = true;
        for (col_name, expected_value) in conditions {
            if let Some(&idx) = col_indices.get(col_name) {
                if idx >= row.len() || &row[idx] != expected_value {
                    all_match = false;
                    break;
                }
            } else {
                all_match = false;
                break;
            }
        }

        // Only add row if NOT all conditions matched
        if !all_match {
            result.push(row.clone());
        }
    }

    result
}

/// Convert a column to a list
pub fn to_list(crif: &Crif, column_name: &str) -> Vec<String> {
    let idx = match get_column_index(crif, column_name) {
        Some(i) => i,
        None => return Vec::new(),
    };

    crif.iter()
        .skip(1)
        .filter_map(|row| {
            if idx < row.len() {
                Some(row[idx].clone())
            } else {
                None
            }
        })
        .collect()
}

/// MarginByRiskClass calculator
pub struct MarginByRiskClass<'a> {
    pub crif: Crif,
    pub calculation_currency: String,
    pub wnc: &'a dyn WeightsAndCorr,
    pub list_risk_types: Vec<String>,
}

impl<'a> MarginByRiskClass<'a> {
    pub fn new(crif: Crif, calculation_currency: String, wnc: &'a dyn WeightsAndCorr) -> Self {
        let list_risk_types = unique_values(&crif, "RiskType");
        Self {
            crif,
            calculation_currency,
            wnc,
            list_risk_types,
        }
    }

    /// Calculate all margins
    pub fn calculate_all(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut results = init_margin_dict();

        // Delta margins
        let ir_delta = self.ir_delta_margin();
        let delta = self.delta_margin();

        // Vega margins
        let ir_vega = self.ir_vega_margin();
        let vega = self.vega_margin();

        // Curvature margins
        let ir_curvature = self.ir_curvature_margin();
        let curvature = self.curvature_margin();

        // BaseCorr margin
        let base_corr = self.base_corr_margin();

        // Merge all results
        for (risk_class, measures) in ir_delta.iter().chain(delta.iter())
            .chain(ir_vega.iter()).chain(vega.iter())
            .chain(ir_curvature.iter()).chain(curvature.iter())
            .chain(base_corr.iter())
        {
            if let Some(result_measures) = results.get_mut(risk_class) {
                for (measure, value) in measures {
                    *result_measures.entry(measure.clone()).or_insert(0.0) += value;
                }
            }
        }

        results
    }

    /// Delta Margin for Rates Risk Classes Only
    pub fn ir_delta_margin(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut updates = init_margin_dict();

        // Check if rates risk types exist
        if !self.list_risk_types.contains(&"Risk_IRCurve".to_string())
            && !self.list_risk_types.contains(&"Risk_Inflation".to_string())
            && !self.list_risk_types.contains(&"Risk_XCcyBasis".to_string())
        {
            return updates;
        }

        let mut dict_cr = HashMap::new();
        let mut list_k = Vec::new();
        let mut list_s = Vec::new();

        let mut conditions = HashMap::new();
        conditions.insert(
            "RiskType".to_string(),
            vec!["Risk_IRCurve".to_string(), "Risk_Inflation".to_string(), "Risk_XCcyBasis".to_string()],
        );
        let crif = filter_rows(&self.crif, &conditions);
        let currency_list = unique_values(&crif, "Qualifier");

        for currency in &currency_list {
            let mut list_ws = Vec::new();
            let mut tenor_k = Vec::new();
            let mut index = Vec::new();

            // Filter by currency
            let mut curr_cond = HashMap::new();
            curr_cond.insert("Qualifier".to_string(), vec![currency.clone()]);
            let crif_currency = filter_rows(&crif, &curr_cond);

            // Drop XCcyBasis for CR calculation
            let mut drop_cond = HashMap::new();
            drop_cond.insert("RiskType".to_string(), "Risk_XCcyBasis".to_string());
            let crif_wo_xccybasis = drop_rows(&crif_currency, &drop_cond);

            // Concentration Threshold
            let t = self.wnc.t("Rates", "Delta", Some(currency), None).unwrap_or(1.0);
            let cr = simm_utils::concentration_threshold(simm_utils::sum_sensitivities(&crif_wo_xccybasis), t);
            dict_cr.insert(currency.clone(), cr);

            // Process each rates risk type
            let risk_types_in_currency = unique_values(&crif_currency, "RiskType");

            for risk_class in &risk_types_in_currency {
                if !["Risk_IRCurve", "Risk_Inflation", "Risk_XCcyBasis"].contains(&risk_class.as_str()) {
                    continue;
                }

                let mut rc_cond = HashMap::new();
                rc_cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                let crif_risk_class = filter_rows(&crif_currency, &rc_cond);

                let sensitivities = simm_utils::sum_sensitivities(&crif_risk_class);

                if risk_class == "Risk_Inflation" {
                    let rw = INFLATION_RW;
                    let ws = rw * sensitivities * cr;
                    list_ws.push(ws);
                    tenor_k.push("Inf".to_string());
                    index.push("Inf".to_string());
                } else if risk_class == "Risk_XCcyBasis" {
                    let rw = CCY_BASIS_SWAP_SPREAD_RW;
                    let ws = rw * sensitivities;
                    list_ws.push(ws);
                    tenor_k.push("XCcy".to_string());
                    index.push("XCcy".to_string());
                } else if risk_class == "Risk_IRCurve" {
                    let subcurve_list = unique_values(&crif_risk_class, "Label2");

                    for subcurve in &subcurve_list {
                        let mut sc_cond = HashMap::new();
                        sc_cond.insert("Label2".to_string(), vec![subcurve.clone()]);
                        let crif_subcurve = filter_rows(&crif_risk_class, &sc_cond);

                        for tenor in simm_utils::tenor_list(&crif_subcurve) {
                            let mut t_cond = HashMap::new();
                            t_cond.insert("Label1".to_string(), vec![tenor.clone()]);
                            let crif_tenor = filter_rows(&crif_subcurve, &t_cond);

                            let s = simm_utils::sum_sensitivities(&crif_tenor);

                            // Determine RW based on currency volatility
                            let rw = if REG_VOL_CCY_BUCKET.contains(&currency.as_str()) {
                                reg_vol_rw_lookup(&tenor)
                            } else if LOW_VOL_CCY_BUCKET.contains(&currency.as_str()) {
                                low_vol_rw_lookup(&tenor)
                            } else {
                                high_vol_rw_lookup(&tenor)
                            };

                            let ws = rw * s * cr;
                            list_ws.push(ws);
                            tenor_k.push(tenor.clone());
                            index.push(subcurve.clone());
                        }
                    }
                }
            }

            let tenor_refs: Vec<&str> = tenor_k.iter().map(|s| s.as_str()).collect();
            let index_refs: Vec<&str> = index.iter().map(|s| s.as_str()).collect();

            let k = k_delta(
                self.wnc,
                "Rates",
                &list_ws,
                None,
                None,
                Some(&tenor_refs),
                Some(&index_refs),
                &self.calculation_currency,
            );
            list_k.push(k);

            let s_b = list_ws.iter().sum::<f64>().min(k).max(-k);
            list_s.push(s_b);
        }

        let mut k_squared_sum: f64 = list_k.iter().map(|x| x.powi(2)).sum();

        for i in 0..currency_list.len() {
            for j in 0..currency_list.len() {
                if i == j {
                    continue;
                }

                let currency_b = &currency_list[i];
                let currency_c = &currency_list[j];

                let cr_b = *dict_cr.get(currency_b).unwrap_or(&1.0);
                let cr_c = *dict_cr.get(currency_c).unwrap_or(&1.0);
                let g = cr_b.min(cr_c) / cr_b.max(cr_c);

                let gamma = if currency_list.len() > 1 {
                    IR_GAMMA_DIFF_CCY
                } else {
                    1.0
                };

                k_squared_sum += gamma * list_s[i] * list_s[j] * g;
            }
        }

        if let Some(rates) = updates.get_mut("Rates") {
            *rates.get_mut("Delta").unwrap() += k_squared_sum.sqrt();
        }

        updates
    }

    /// Delta Margin for non-Rates risk classes
    pub fn delta_margin(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut updates = init_margin_dict();

        let allowed = ["Risk_FX", "Risk_CreditQ", "Risk_CreditNonQ", "Risk_Equity", "Risk_Commodity"];
        let list_risk_classes: Vec<String> = self
            .list_risk_types
            .iter()
            .filter(|rc| allowed.contains(&rc.as_str()))
            .cloned()
            .collect();

        if list_risk_classes.is_empty() {
            return updates;
        }

        for risk_class in &list_risk_classes {
            let mut k_res = 0.0;
            let mut list_k = Vec::new();
            let mut list_s = Vec::new();

            if risk_class == "Risk_FX" {
                let mut list_ws = Vec::new();
                let mut list_cr = Vec::new();

                let mut cond = HashMap::new();
                cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                let crif_fx = filter_rows(&self.crif, &cond);

                let currency_list = unique_values(&crif_fx, "Qualifier");

                for currency in &currency_list {
                    let mut curr_cond = HashMap::new();
                    curr_cond.insert("Qualifier".to_string(), vec![currency.clone()]);
                    let crif_currency = filter_rows(&crif_fx, &curr_cond);

                    let t = self.wnc.t(risk_class, "Delta", Some(currency), None).unwrap_or(1.0);
                    let sensitivities = simm_utils::sum_sensitivities(&crif_currency);
                    let cr = simm_utils::concentration_threshold(sensitivities, t);
                    list_cr.push(cr);

                    let is_given_high = HIGH_VOL_CURRENCY_GROUP.contains(&currency.as_str());
                    let is_calc_high = HIGH_VOL_CURRENCY_GROUP.contains(&self.calculation_currency.as_str());

                    let rw = if currency == &self.calculation_currency {
                        0.0
                    } else {
                        fx_rw(
                            if is_calc_high { RiskLevel::High } else { RiskLevel::Regular },
                            if is_given_high { RiskLevel::High } else { RiskLevel::Regular },
                        )
                    };

                    list_ws.push(sensitivities * cr * rw);
                }

                let currency_refs: Vec<&str> = currency_list.iter().map(|s| s.as_str()).collect();
                let k = k_delta(
                    self.wnc,
                    risk_class,
                    &list_ws,
                    Some(&list_cr),
                    Some(&currency_refs),
                    None,
                    None,
                    &self.calculation_currency,
                );

                if let Some(fx) = updates.get_mut("FX") {
                    *fx.get_mut("Delta").unwrap() += k;
                }
            } else if ["Risk_CreditQ", "Risk_CreditNonQ", "Risk_Equity", "Risk_Commodity"].contains(&risk_class.as_str()) {
                let mut cond = HashMap::new();
                cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                let crif_others = filter_rows(&self.crif, &cond);
                let bucket_list = simm_utils::bucket_list(&crif_others);

                for bucket in &bucket_list {
                    let crif_bucket = if *bucket == 0 {
                        let mut b_cond = HashMap::new();
                        b_cond.insert("Bucket".to_string(), vec!["Residual".to_string()]);
                        filter_rows(&crif_others, &b_cond)
                    } else {
                        let mut b_cond = HashMap::new();
                        b_cond.insert("Bucket".to_string(), vec![bucket.to_string()]);
                        filter_rows(&crif_others, &b_cond)
                    };

                    let rw = self.wnc.rw(risk_class, &bucket.to_string()).unwrap_or(1.0);
                    let t = self.wnc.t(risk_class, "Delta", None, Some(&bucket.to_string())).unwrap_or(1.0);

                    let mut list_ws = Vec::new();
                    let mut list_cr_local = Vec::new();
                    let mut index = Vec::new();

                    let qualifier_list = unique_values(&crif_bucket, "Qualifier");

                    for qualifier in &qualifier_list {
                        let mut q_cond = HashMap::new();
                        q_cond.insert("Qualifier".to_string(), vec![qualifier.clone()]);
                        let crif_qualifier = filter_rows(&crif_bucket, &q_cond);

                        if ["Risk_CreditQ", "Risk_CreditNonQ"].contains(&risk_class.as_str()) {
                            let sensitivities_cr = simm_utils::sum_sensitivities(&crif_qualifier);
                            let cr = 1.0_f64.max((sensitivities_cr.abs() / t).sqrt());

                            let label2_list = unique_values(&crif_qualifier, "Label2");
                            for label2 in &label2_list {
                                for tenor in simm_utils::tenor_list(&crif_qualifier) {
                                    let mut t_cond = HashMap::new();
                                    t_cond.insert("Label1".to_string(), vec![tenor.clone()]);
                                    t_cond.insert("Label2".to_string(), vec![label2.clone()]);
                                    let crif_tenor = filter_rows(&crif_qualifier, &t_cond);

                                    let sensitivities = simm_utils::sum_sensitivities(&crif_tenor);
                                    list_ws.push(rw * sensitivities * cr);
                                    list_cr_local.push(cr);

                                    if *bucket == 0 {
                                        index.push("Res".to_string());
                                    } else if risk_class == "Risk_CreditQ" {
                                        index.push(qualifier.clone());
                                    } else {
                                        index.push(label2.clone());
                                    }
                                }
                            }
                        } else {
                            // Equity, Commodity
                            let sensitivities = simm_utils::sum_sensitivities(&crif_qualifier);
                            let cr = 1.0_f64.max((sensitivities.abs() / t).sqrt());
                            list_cr_local.push(cr);
                            list_ws.push(rw * sensitivities * cr);
                        }
                    }

                    let index_refs: Vec<&str> = index.iter().map(|s| s.as_str()).collect();
                    let k = k_delta(
                        self.wnc,
                        risk_class,
                        &list_ws,
                        Some(&list_cr_local),
                        Some(&[&bucket.to_string()]),
                        None,
                        if index.is_empty() { None } else { Some(&index_refs) },
                        &self.calculation_currency,
                    );

                    if *bucket == 0 {
                        k_res += k;
                    } else {
                        list_k.push(k);
                    }

                    let s_b = list_ws.iter().sum::<f64>().min(k).max(-k);
                    list_s.push(s_b);
                }

                // Calculate aggregated K
                let bucket_list_non_res: Vec<usize> = bucket_list.iter().filter(|&&b| b != 0).copied().collect();

                if risk_class != "Risk_FX" {
                    let mut k_squared_sum: f64 = list_k.iter().map(|x| x.powi(2)).sum();

                    for i in 0..bucket_list_non_res.len() {
                        for j in 0..bucket_list_non_res.len() {
                            if i == j {
                                continue;
                            }

                            let bucket1 = bucket_list_non_res[i];
                            let bucket2 = bucket_list_non_res[j];

                            let gamma = self.wnc.gamma(risk_class, &bucket1.to_string(), &bucket2.to_string()).unwrap_or(0.0);

                            k_squared_sum += gamma * list_s[i] * list_s[j];
                        }
                    }

                    let total = k_squared_sum.sqrt() + k_res;

                    if LIST_CREDIT_Q.contains(&risk_class.as_str()) {
                        if let Some(cq) = updates.get_mut("CreditQ") {
                            *cq.get_mut("Delta").unwrap() += total;
                        }
                    } else if LIST_CREDIT_NON_Q.contains(&risk_class.as_str()) {
                        if let Some(cnq) = updates.get_mut("CreditNonQ") {
                            *cnq.get_mut("Delta").unwrap() += total;
                        }
                    } else if LIST_EQUITY.contains(&risk_class.as_str()) {
                        if let Some(eq) = updates.get_mut("Equity") {
                            *eq.get_mut("Delta").unwrap() += total;
                        }
                    } else if LIST_COMMODITY.contains(&risk_class.as_str()) {
                        if let Some(com) = updates.get_mut("Commodity") {
                            *com.get_mut("Delta").unwrap() += k_squared_sum.sqrt();
                        }
                    }
                }
            }
        }

        updates
    }

    /// IR Vega Margin
    pub fn ir_vega_margin(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut updates = init_margin_dict();

        if !self.list_risk_types.contains(&"Risk_IRVol".to_string())
            && !self.list_risk_types.contains(&"Risk_InflationVol".to_string())
        {
            return updates;
        }

        let mut list_k = Vec::new();
        let mut dict_s = HashMap::new();
        let mut dict_vcr = HashMap::new();

        let currency_list = unique_values(&self.crif, "Qualifier");

        for currency in &currency_list {
            let mut cond = HashMap::new();
            cond.insert("RiskType".to_string(), vec!["Risk_IRVol".to_string(), "Risk_InflationVol".to_string()]);
            cond.insert("Qualifier".to_string(), vec![currency.clone()]);
            let crif_currency = filter_rows(&self.crif, &cond);

            let mut vr = Vec::new();
            let mut index = Vec::new();

            let sensitivities_cr = simm_utils::sum_sensitivities(&crif_currency);
            let vt = self.wnc.t("Rates", "Vega", Some(currency), None).unwrap_or(1.0);
            let vcr = 1.0_f64.max((sensitivities_cr.abs() / vt).sqrt());
            dict_vcr.insert(currency.clone(), vcr);

            let risk_types_in_currency = unique_values(&crif_currency, "RiskType");

            for risk_class in &risk_types_in_currency {
                if !["Risk_IRVol", "Risk_InflationVol"].contains(&risk_class.as_str()) {
                    continue;
                }

                let mut rc_cond = HashMap::new();
                rc_cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                let crif_risk_class = filter_rows(&crif_currency, &rc_cond);

                for tenor in simm_utils::tenor_list(&crif_risk_class) {
                    let mut t_cond = HashMap::new();
                    t_cond.insert("Label1".to_string(), vec![tenor.clone()]);
                    let crif_tenor = filter_rows(&crif_risk_class, &t_cond);

                    let sensitivities = simm_utils::sum_sensitivities(&crif_tenor);
                    vr.push(IR_VRW * sensitivities * vcr);

                    if risk_class == "Risk_IRVol" {
                        index.push(tenor);
                    } else {
                        index.push("Inf".to_string());
                    }
                }
            }

            let index_refs: Vec<&str> = index.iter().map(|s| s.as_str()).collect();
            let k = k_vega(self.wnc, "Rates", &vr, None, None, Some(&index_refs));
            list_k.push(k);

            let s = vr.iter().sum::<f64>().min(k).max(-k);
            dict_s.insert(currency.clone(), s);
        }

        let mut k_squared_sum: f64 = list_k.iter().map(|x| x.powi(2)).sum();

        for i in 0..currency_list.len() {
            for j in 0..currency_list.len() {
                if i == j {
                    continue;
                }

                let currency_b = &currency_list[i];
                let currency_c = &currency_list[j];

                let vcr_b = *dict_vcr.get(currency_b).unwrap_or(&1.0);
                let vcr_c = *dict_vcr.get(currency_c).unwrap_or(&1.0);
                let g = vcr_b.min(vcr_c) / vcr_b.max(vcr_c);
                let gamma = IR_GAMMA_DIFF_CCY;

                k_squared_sum += gamma * dict_s[currency_b] * dict_s[currency_c] * g;
            }
        }

        if let Some(rates) = updates.get_mut("Rates") {
            *rates.get_mut("Vega").unwrap() += k_squared_sum.sqrt();
        }

        updates
    }

    /// Vega Margin for non-Rates risk classes
    pub fn vega_margin(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut updates = init_margin_dict();

        let allowed = ["Risk_CreditVol", "Risk_CreditVolNonQ", "Risk_EquityVol", "Risk_CommodityVol", "Risk_FXVol"];
        let list_risk_classes: Vec<String> = self
            .list_risk_types
            .iter()
            .filter(|rc| allowed.contains(&rc.as_str()))
            .cloned()
            .collect();

        for risk_class in &list_risk_classes {
            if risk_class == "Risk_FXVol" {
                let mut list_vr = Vec::new();
                let mut list_vcr = Vec::new();

                for currency_pair in simm_utils::currency_pair_list(&self.crif) {
                    let reversed = format!("{}{}", &currency_pair[3..6], &currency_pair[0..3]);
                    let mut cond = HashMap::new();
                    cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                    cond.insert("Qualifier".to_string(), vec![currency_pair.clone(), reversed]);
                    let crif_fx = filter_rows(&self.crif, &cond);

                    let is_ccy1_high = HIGH_VOL_CURRENCY_GROUP.contains(&&currency_pair[0..3]);
                    let is_ccy2_high = HIGH_VOL_CURRENCY_GROUP.contains(&&currency_pair[3..6]);

                    let rw = fx_rw(
                        if is_ccy2_high { RiskLevel::High } else { RiskLevel::Regular },
                        if is_ccy1_high { RiskLevel::High } else { RiskLevel::Regular },
                    );

                    let normal = Normal::new(0.0, 1.0).unwrap();
                    let sigma = rw * (365.0_f64 / 14.0_f64).sqrt() / normal.inverse_cdf(0.99);
                    let sensitivities = simm_utils::sum_sensitivities(&crif_fx);
                    let vr_ik = FX_HVR * sigma * sensitivities;
                    let vt = self.wnc.t(risk_class, "Vega", Some(&currency_pair), None).unwrap_or(1.0);
                    let vcr = 1.0_f64.max((vr_ik.abs() / vt).sqrt());
                    list_vcr.push(vcr);

                    let vr_k = FX_VRW * vr_ik * vcr;
                    list_vr.push(vr_k);
                }

                let k = k_vega(self.wnc, risk_class, &list_vr, Some(&list_vcr), None, None);
                if let Some(fx) = updates.get_mut("FX") {
                    *fx.get_mut("Vega").unwrap() += k;
                }
            } else {
                // Equity, Commodity, Credit
                let mut k_res = 0.0;
                let mut list_k = Vec::new();
                let mut list_s = Vec::new();

                let mut cond = HashMap::new();
                cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                let crif_risk_type = filter_rows(&self.crif, &cond);
                let bucket_list = simm_utils::bucket_list(&crif_risk_type);

                for bucket in &bucket_list {
                    let crif_bucket = if *bucket == 0 {
                        let mut b_cond = HashMap::new();
                        b_cond.insert("Bucket".to_string(), vec!["Residual".to_string()]);
                        filter_rows(&crif_risk_type, &b_cond)
                    } else {
                        let mut b_cond = HashMap::new();
                        b_cond.insert("Bucket".to_string(), vec![bucket.to_string()]);
                        filter_rows(&crif_risk_type, &b_cond)
                    };

                    let mut vr = Vec::new();
                    let mut list_vcr_local = Vec::new();
                    let mut index = Vec::new();

                    let qualifier_list = unique_values(&crif_bucket, "Qualifier");

                    for qualifier in &qualifier_list {
                        let mut q_cond = HashMap::new();
                        q_cond.insert("Qualifier".to_string(), vec![qualifier.clone()]);
                        let crif_qualifier = filter_rows(&crif_bucket, &q_cond);

                        let rw = self.wnc.rw(risk_class, &bucket.to_string()).unwrap_or(1.0);
                        let normal = Normal::new(0.0, 1.0).unwrap();
                        let sigma = rw * (365.0_f64 / 14.0_f64).sqrt() / normal.inverse_cdf(0.99);

                        if ["Risk_EquityVol", "Risk_CommodityVol"].contains(&risk_class.as_str()) {
                            let hvr = if risk_class == "Risk_EquityVol" { EQUITY_HVR } else { COMMODITY_HVR };
                            let vrw = if risk_class == "Risk_EquityVol" {
                                if *bucket == 12 { EQUITY_VRW_BUCKET_12 } else { EQUITY_VRW }
                            } else {
                                COMMODITY_VRW
                            };

                            let mut vr_ik = Vec::new();
                            let sensitivities = simm_utils::sum_sensitivities(&crif_qualifier);
                            vr_ik.push(hvr * sigma * sensitivities);

                            let vr_i: f64 = vr_ik.iter().sum();
                            let vt = self.wnc.t(risk_class, "Vega", None, Some(&bucket.to_string())).unwrap_or(1.0);
                            let vcr = 1.0_f64.max((vr_i.abs() / vt).sqrt());

                            list_vcr_local.push(vcr);
                            vr.push(vr_i * vrw * vcr);
                        } else {
                            // Credit
                            let vt = self.wnc.t(risk_class, "Vega", None, Some(&bucket.to_string())).unwrap_or(1.0);
                            let sensitivities_vt = simm_utils::sum_sensitivities(&crif_qualifier);
                            let vcr = 1.0_f64.max((sensitivities_vt.abs() / vt).sqrt());

                            let label2_list = unique_values(&crif_qualifier, "Label2");
                            for label2 in &label2_list {
                                for tenor in simm_utils::tenor_list(&crif_qualifier) {
                                    let mut t_cond = HashMap::new();
                                    t_cond.insert("Label1".to_string(), vec![tenor]);
                                    t_cond.insert("Label2".to_string(), vec![label2.clone()]);
                                    let crif_tenor = filter_rows(&crif_qualifier, &t_cond);

                                    let sensitivities = simm_utils::sum_sensitivities(&crif_tenor);

                                    let vrw = if risk_class == "Risk_CreditVol" {
                                        CREDIT_Q_V_RW
                                    } else {
                                        CREDIT_NON_Q_VRW
                                    };

                                    vr.push(vrw * sensitivities * vcr);
                                    list_vcr_local.push(vcr);

                                    if *bucket == 0 {
                                        index.push("Res".to_string());
                                    } else if risk_class == "Risk_CreditVol" {
                                        index.push(qualifier.clone());
                                    } else {
                                        index.push(label2.clone());
                                    }
                                }
                            }
                        }
                    }

                    let index_refs: Vec<&str> = index.iter().map(|s| s.as_str()).collect();
                    let k = k_vega(
                        self.wnc,
                        risk_class,
                        &vr,
                        Some(&list_vcr_local),
                        Some(&bucket.to_string()),
                        if index.is_empty() { None } else { Some(&index_refs) },
                    );

                    if *bucket == 0 {
                        k_res += k;
                    } else {
                        list_k.push(k);
                    }

                    let s = vr.iter().sum::<f64>().min(k).max(-k);
                    list_s.push(s);
                }

                let bucket_list_non_res: Vec<usize> = bucket_list.iter().filter(|&&b| b != 0).copied().collect();

                let mut k_squared_sum: f64 = list_k.iter().map(|x| x.powi(2)).sum();

                for i in 0..bucket_list_non_res.len() {
                    for j in 0..bucket_list_non_res.len() {
                        if i == j {
                            continue;
                        }

                        let bucket_i = &bucket_list_non_res[i].to_string();
                        let bucket_j = &bucket_list_non_res[j].to_string();

                        let gamma = if risk_class == "Risk_CreditVolNonQ" {
                            self.wnc.gamma(risk_class, bucket_i, bucket_j).unwrap_or(0.0)
                        } else {
                            self.wnc.gamma(risk_class, bucket_i, bucket_j).unwrap_or(0.0)
                        };

                        k_squared_sum += gamma * list_s[i] * list_s[j];
                    }
                }

                let total = k_squared_sum.sqrt() + k_res;

                if LIST_CREDIT_Q.contains(&risk_class.as_str()) {
                    if let Some(cq) = updates.get_mut("CreditQ") {
                        *cq.get_mut("Vega").unwrap() += total;
                    }
                } else if LIST_CREDIT_NON_Q.contains(&risk_class.as_str()) {
                    if let Some(cnq) = updates.get_mut("CreditNonQ") {
                        *cnq.get_mut("Vega").unwrap() += total;
                    }
                } else if LIST_EQUITY.contains(&risk_class.as_str()) {
                    if let Some(eq) = updates.get_mut("Equity") {
                        *eq.get_mut("Vega").unwrap() += total;
                    }
                } else if LIST_COMMODITY.contains(&risk_class.as_str()) {
                    if let Some(com) = updates.get_mut("Commodity") {
                        *com.get_mut("Vega").unwrap() += total;
                    }
                }
            }
        }

        updates
    }

    /// IR Curvature Margin
    pub fn ir_curvature_margin(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut updates = init_margin_dict();

        if !self.list_risk_types.contains(&"Risk_IRVol".to_string())
            && !self.list_risk_types.contains(&"Risk_InflationVol".to_string())
        {
            return updates;
        }

        let mut list_k = Vec::new();
        let mut list_s = Vec::new();
        let mut cvr_sum = 0.0;
        let mut cvr_abs_sum = 0.0;

        let currency_list = unique_values(&self.crif, "Qualifier");

        for currency in &currency_list {
            let mut cond = HashMap::new();
            cond.insert("RiskType".to_string(), vec!["Risk_IRVol".to_string(), "Risk_InflationVol".to_string()]);
            cond.insert("Qualifier".to_string(), vec![currency.clone()]);
            let crif_currency = filter_rows(&self.crif, &cond);

            // Exception check
            let risk_types = unique_values(&crif_currency, "RiskType");
            if risk_types == vec!["Risk_InflationVol"]
                && sum_column(&crif_currency, "AmountUSD") == 0.0
                && currency == &self.calculation_currency
            {
                return updates;
            }

            let mut cvr_ik = Vec::new();
            let mut index = Vec::new();

            for risk_class in &risk_types {
                if !["Risk_IRVol", "Risk_InflationVol"].contains(&risk_class.as_str()) {
                    continue;
                }

                let mut rc_cond = HashMap::new();
                rc_cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                let crif_risk_class = filter_rows(&crif_currency, &rc_cond);

                for tenor in unique_values(&crif_risk_class, "Label1") {
                    let mut t_cond = HashMap::new();
                    t_cond.insert("Label1".to_string(), vec![tenor.clone()]);
                    let crif_tenor = filter_rows(&crif_risk_class, &t_cond);

                    let sensitivities = simm_utils::sum_sensitivities(&crif_tenor);
                    let cvr = simm_utils::scaling_func(&tenor) * sensitivities;
                    cvr_ik.push(cvr);
                    cvr_sum += cvr;
                    cvr_abs_sum += cvr.abs();

                    if risk_class == "Risk_IRVol" {
                        index.push(tenor);
                    } else {
                        index.push("Inf".to_string());
                    }
                }
            }

            let index_refs: Vec<&str> = index.iter().map(|s| s.as_str()).collect();
            let k = k_curvature(self.wnc, "Rates", &cvr_ik, None, Some(&index_refs));
            list_k.push(k);

            let s = cvr_ik.iter().sum::<f64>().min(k).max(-k);
            list_s.push(s);
        }

        let theta = if cvr_abs_sum != 0.0 {
            (cvr_sum / cvr_abs_sum).min(0.0)
        } else {
            0.0
        };

        let normal = Normal::new(0.0, 1.0).unwrap();
        let lambda = (normal.inverse_cdf(0.995).powi(2) - 1.0) * (1.0 + theta) - theta;

        let mut k: f64 = list_k.iter().map(|x| x.powi(2)).sum();

        for i in 0..list_s.len() {
            for j in 0..list_s.len() {
                if i == j {
                    continue;
                }

                let gamma = IR_GAMMA_DIFF_CCY;
                k += list_s[i] * list_s[j] * gamma.powi(2);
            }
        }

        if let Some(rates) = updates.get_mut("Rates") {
            *rates.get_mut("Curvature").unwrap() += (cvr_sum + lambda * k.sqrt()).max(0.0) / IR_HVR.powi(2);
        }

        updates
    }

    /// Curvature Margin for non-Rates risk classes
    pub fn curvature_margin(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut updates = init_margin_dict();

        let allowed = ["Risk_CreditVol", "Risk_CreditVolNonQ", "Risk_EquityVol", "Risk_CommodityVol", "Risk_FXVol"];
        let list_risk_classes: Vec<String> = self
            .list_risk_types
            .iter()
            .filter(|rc| allowed.contains(&rc.as_str()))
            .cloned()
            .collect();

        for risk_class in &list_risk_classes {
            if risk_class == "Risk_FXVol" {
                let mut list_cvr = Vec::new();
                let mut cvr_sum = 0.0;
                let mut cvr_abs_sum = 0.0;

                for currency_pair in simm_utils::currency_pair_list(&self.crif) {
                    let reversed = format!("{}{}", &currency_pair[3..6], &currency_pair[0..3]);
                    let mut cond = HashMap::new();
                    cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                    cond.insert("Qualifier".to_string(), vec![currency_pair.clone(), reversed]);
                    let df = filter_rows(&self.crif, &cond);

                    let is_ccy1_high = HIGH_VOL_CURRENCY_GROUP.contains(&&currency_pair[0..3]);
                    let is_ccy2_high = HIGH_VOL_CURRENCY_GROUP.contains(&&currency_pair[3..6]);

                    let rw = fx_rw(
                        if is_ccy2_high { RiskLevel::High } else { RiskLevel::Regular },
                        if is_ccy1_high { RiskLevel::High } else { RiskLevel::Regular },
                    );

                    let normal = Normal::new(0.0, 1.0).unwrap();
                    let sigma = rw * (365.0_f64 / 14.0_f64).sqrt() / normal.inverse_cdf(0.99);
                    let vega_list = to_list(&df, "AmountUSD");
                    let tenor_list = to_list(&df, "Label1");

                    let mut cvr = 0.0;
                    for (k, vega) in tenor_list.iter().zip(vega_list.iter()) {
                        if !vega.is_empty() {
                            if let Ok(v) = vega.parse::<f64>() {
                                cvr += simm_utils::scaling_func(k) * sigma * v;
                            }
                        }
                    }

                    list_cvr.push(cvr);
                    cvr_sum += cvr;
                    cvr_abs_sum += cvr.abs();
                }

                let k = k_curvature(self.wnc, risk_class, &list_cvr, None, None);

                let theta = if cvr_abs_sum != 0.0 {
                    (cvr_sum / cvr_abs_sum).min(0.0)
                } else {
                    0.0
                };

                let normal = Normal::new(0.0, 1.0).unwrap();
                let lambda = (normal.inverse_cdf(0.995).powi(2) - 1.0) * (1.0 + theta) - theta;

                if let Some(fx) = updates.get_mut("FX") {
                    *fx.get_mut("Curvature").unwrap() += (cvr_sum + lambda * k).max(0.0);
                }
            } else {
                // Equity, Commodity, Credit
                let mut k_res = 0.0;
                let mut list_k = Vec::new();
                let mut list_s = Vec::new();
                let mut buckets_in_list_s = Vec::new();  // Track which buckets are in list_s
                let mut cvr_sum = 0.0;
                let mut cvr_sum_res = 0.0;
                let mut cvr_abs_sum = 0.0;
                let mut cvr_abs_sum_res = 0.0;

                let mut cond = HashMap::new();
                cond.insert("RiskType".to_string(), vec![risk_class.clone()]);
                let crif_filtered = filter_rows(&self.crif, &cond);
                let bucket_list = simm_utils::bucket_list(&crif_filtered);

                for bucket in &bucket_list {
                    let crif_bucket = if *bucket == 0 {
                        let mut b_cond = HashMap::new();
                        b_cond.insert("Bucket".to_string(), vec!["Residual".to_string()]);
                        filter_rows(&crif_filtered, &b_cond)
                    } else {
                        let mut b_cond = HashMap::new();
                        b_cond.insert("Bucket".to_string(), vec![bucket.to_string()]);
                        filter_rows(&crif_filtered, &b_cond)
                    };

                    let mut cvr_i = Vec::new();
                    let mut index = Vec::new();

                    for qualifier in unique_values(&crif_bucket, "Qualifier") {
                        let mut q_cond = HashMap::new();
                        q_cond.insert("Qualifier".to_string(), vec![qualifier.clone()]);
                        let crif_qualifier = filter_rows(&crif_bucket, &q_cond);

                        let tenor_list = to_list(&crif_qualifier, "Label1");
                        let vega_list = to_list(&crif_qualifier, "AmountUSD");

                        let rw = self.wnc.rw(risk_class, &bucket.to_string()).unwrap_or(1.0);
                        let normal = Normal::new(0.0, 1.0).unwrap();
                        let sigma = rw * (365.0_f64 / 14.0_f64).sqrt() / normal.inverse_cdf(0.99);

                        if ["Risk_EquityVol", "Risk_CommodityVol"].contains(&risk_class.as_str()) {
                            // No curvature for equity bucket 12
                            let sigma = if risk_class == "Risk_EquityVol" && *bucket == 12 {
                                0.0
                            } else {
                                sigma
                            };

                            let mut cvr_ik = Vec::new();
                            for (k, vega) in tenor_list.iter().zip(vega_list.iter()) {
                                if !vega.is_empty() {
                                    if let Ok(v) = vega.parse::<f64>() {
                                        cvr_ik.push(simm_utils::scaling_func(k) * sigma * v);
                                    }
                                }
                            }
                            cvr_i.push(cvr_ik.iter().sum());
                        } else {
                            // Credit
                            let label2_list = unique_values(&crif_qualifier, "Label2");
                            for label2 in &label2_list {
                                for tenor in simm_utils::tenor_list(&crif_qualifier) {
                                    let mut t_cond = HashMap::new();
                                    t_cond.insert("Label1".to_string(), vec![tenor.clone()]);
                                    t_cond.insert("Label2".to_string(), vec![label2.clone()]);
                                    let crif_tenor = filter_rows(&crif_qualifier, &t_cond);

                                    let sensitivities = simm_utils::sum_sensitivities(&crif_tenor);

                                    if *bucket == 0 {
                                        index.push("Res".to_string());
                                    } else if risk_class == "Risk_CreditVol" {
                                        index.push(qualifier.clone());
                                    } else {
                                        index.push(label2.clone());
                                    }

                                    cvr_i.push(simm_utils::scaling_func(&tenor) * sensitivities);
                                }
                            }
                        }
                    }

                    let index_refs: Vec<&str> = index.iter().map(|s| s.as_str()).collect();
                    let k = k_curvature(
                        self.wnc,
                        risk_class,
                        &cvr_i,
                        Some(&bucket.to_string()),
                        if index.is_empty() { None } else { Some(&index_refs) },
                    );

                    if *bucket == 0 {
                        k_res += k;
                        cvr_sum_res += cvr_i.iter().sum::<f64>();
                        cvr_abs_sum_res += cvr_i.iter().map(|x| x.abs()).sum::<f64>();
                    } else {
                        if !(risk_class == "Risk_EquityVol" && *bucket == 12) {
                            list_k.push(k);
                            let s = cvr_i.iter().sum::<f64>().min(k).max(-k);
                            list_s.push(s);
                            buckets_in_list_s.push(*bucket);  // Track this bucket
                        }
                        cvr_sum += cvr_i.iter().sum::<f64>();
                        cvr_abs_sum += cvr_i.iter().map(|x| x.abs()).sum::<f64>();
                    }
                }

                let bucket_list_unique: Vec<usize> = bucket_list.iter().copied().collect();
                let has_residual = bucket_list_unique.contains(&0);
                let has_non_residual = bucket_list_unique.iter().any(|&b| b != 0);

                let (_theta, lambda, _theta_res, lambda_res) = {
                    let normal = Normal::new(0.0, 1.0).unwrap();
                    let ppf = normal.inverse_cdf(0.995).powi(2);

                    if has_residual && has_non_residual {
                        let t = if cvr_abs_sum != 0.0 { (cvr_sum / cvr_abs_sum).min(0.0) } else { 0.0 };
                        let l = (ppf - 1.0) * (1.0 + t) - t;
                        let t_res = if cvr_abs_sum_res != 0.0 { (cvr_sum_res / cvr_abs_sum_res).min(0.0) } else { 0.0 };
                        let l_res = (ppf - 1.0) * (1.0 + t_res) - t_res;
                        (t, l, t_res, l_res)
                    } else if !has_residual {
                        let t = if cvr_abs_sum != 0.0 { (cvr_sum / cvr_abs_sum).min(0.0) } else { 0.0 };
                        let l = (ppf - 1.0) * (1.0 + t) - t;
                        (t, l, 0.0, 0.0)
                    } else {
                        let t_res = if cvr_abs_sum_res != 0.0 { (cvr_sum_res / cvr_abs_sum_res).min(0.0) } else { 0.0 };
                        let l_res = (ppf - 1.0) * (1.0 + t_res) - t_res;
                        (0.0, 0.0, t_res, l_res)
                    }
                };

                let mut k_squared: f64 = list_k.iter().map(|x| x.powi(2)).sum();

                // Use buckets_in_list_s to ensure we only loop over buckets that are in list_s
                for i in 0..buckets_in_list_s.len() {
                    for j in 0..buckets_in_list_s.len() {
                        if i == j {
                            continue;
                        }

                        let bucket_i = &buckets_in_list_s[i].to_string();
                        let bucket_j = &buckets_in_list_s[j].to_string();

                        let gamma = if risk_class == "Risk_CreditVolNonQ" {
                            self.wnc.gamma(risk_class, bucket_i, bucket_j).unwrap_or(0.0)
                        } else {
                            self.wnc.gamma(risk_class, bucket_i, bucket_j).unwrap_or(0.0)
                        };

                        k_squared += list_s[i] * list_s[j] * gamma.powi(2);
                    }
                }

                let curvature_margin_non_res = if k_squared >= 0.0 {
                    (cvr_sum + lambda * k_squared.sqrt()).max(0.0)
                } else {
                    0.0
                };
                let curvature_margin_res = (cvr_sum_res + lambda_res * k_res).max(0.0);

                let total = curvature_margin_non_res + curvature_margin_res;

                if LIST_EQUITY.contains(&risk_class.as_str()) {
                    if let Some(eq) = updates.get_mut("Equity") {
                        *eq.get_mut("Curvature").unwrap() += total;
                    }
                } else if LIST_COMMODITY.contains(&risk_class.as_str()) {
                    if let Some(com) = updates.get_mut("Commodity") {
                        *com.get_mut("Curvature").unwrap() += total;
                    }
                } else if risk_class == "Risk_CreditVol" {
                    if let Some(cq) = updates.get_mut("CreditQ") {
                        *cq.get_mut("Curvature").unwrap() += total;
                    }
                } else if risk_class == "Risk_CreditVolNonQ" {
                    if let Some(cnq) = updates.get_mut("CreditNonQ") {
                        *cnq.get_mut("Curvature").unwrap() += total;
                    }
                }
            }
        }

        updates
    }

    /// Base Correlation Margin
    pub fn base_corr_margin(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut updates = init_margin_dict();

        let mut cond = HashMap::new();
        cond.insert("RiskType".to_string(), vec!["Risk_BaseCorr".to_string()]);
        let crif_base_corr = filter_rows(&self.crif, &cond);

        let mut list_ws = Vec::new();
        let qualifier_list = unique_values(&crif_base_corr, "Qualifier");

        for qualifier in &qualifier_list {
            let mut q_cond = HashMap::new();
            q_cond.insert("Qualifier".to_string(), vec![qualifier.clone()]);
            let crif_qualifier = filter_rows(&crif_base_corr, &q_cond);

            let rw = CREDIT_Q_BASE_CORR_WEIGHT;
            let sensitivities = simm_utils::sum_sensitivities(&crif_qualifier);
            let ws = rw * sensitivities;
            list_ws.push(ws);
        }

        let mut base_corr = 0.0;
        for i in 0..list_ws.len() {
            for j in 0..list_ws.len() {
                let rho = if i == j || qualifier_list[i] == qualifier_list[j] {
                    1.0
                } else {
                    self.wnc.rho("Risk_BaseCorr", "", "", None).unwrap_or(1.0)
                };

                base_corr += list_ws[i] * list_ws[j] * rho;
            }
        }

        if let Some(cq) = updates.get_mut("CreditQ") {
            *cq.get_mut("BaseCorr").unwrap() += base_corr.sqrt();
        }

        updates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_margin_dict() {
        let dict = init_margin_dict();
        assert_eq!(dict.len(), 6);
        assert!(dict.contains_key("Rates"));
        assert!(dict.contains_key("CreditQ"));
    }

    #[test]
    fn test_filter_rows() {
        let crif = vec![
            vec!["RiskType".to_string(), "Bucket".to_string()],
            vec!["Risk_FX".to_string(), "1".to_string()],
            vec!["Risk_Equity".to_string(), "2".to_string()],
            vec!["Risk_FX".to_string(), "2".to_string()],
        ];

        let mut conditions = HashMap::new();
        conditions.insert("RiskType".to_string(), vec!["Risk_FX".to_string()]);
        let result = filter_rows(&crif, &conditions);

        assert_eq!(result.len(), 3); // Header + 2 rows
    }
}
