use crate::constants::*;
use crate::wnc::WeightsAndCorr;
use crate::v2_5::*;

/// Compute delta capital charge K for a given risk class and bucket
///
/// # Arguments
/// * `wnc` - Weights and correlations provider
/// * `risk_class` - Risk class (e.g., "Rates", "Risk_CreditQ", etc.)
/// * `list_ws` - List of weighted sensitivities
/// * `list_cr` - List of concentration risk factors (optional)
/// * `bucket` - Bucket identifiers (optional)
/// * `tenor` - Tenor labels (optional, for rates)
/// * `index` - Index labels (optional, for credit/rates)
/// * `calculation_currency` - Calculation currency (default "USD")
pub fn k_delta(
    wnc: &dyn WeightsAndCorr,
    risk_class: &str,
    list_ws: &[f64],
    list_cr: Option<&[f64]>,
    bucket: Option<&[&str]>,
    tenor: Option<&[&str]>,
    index: Option<&[&str]>,
    calculation_currency: &str,
) -> f64 {
    let n = list_ws.len();

    // K = sum of WS^2
    let mut k: f64 = list_ws.iter().map(|ws| ws.powi(2)).sum();

    // Add correlation terms
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }

            let mut rho: f64 = 1.0;
            let mut phi: f64 = 1.0;
            let mut f: f64 = 1.0;

            // Calculate rho based on risk class
            if risk_class == "Rates" {
                // Determine phi for rates
                if let Some(idx) = index {
                    if idx[i] == idx[j] {
                        phi = 1.0;
                    } else if idx[i] == "XCcy" || idx[j] == "XCcy" {
                        phi = CCY_BASIS_SPREAD_CORR;
                    } else if idx[i] == "Inf" || idx[j] == "Inf" {
                        phi = INFLATION_CORR;
                    } else {
                        phi = SUB_CURVES_CORR;
                    }

                    // Calculate rho for rates
                    if idx[i] != "Inf" && idx[i] != "XCcy" && idx[j] != "Inf" && idx[j] != "XCcy" {
                        if let Some(ten) = tenor {
                            rho = wnc.rho("Risk_IRCurve", ten[i], ten[j], None).unwrap_or(1.0);
                        }
                    } else {
                        rho = 1.0;
                    }
                }

                f = 1.0; // For rates, f is always 1
            } else if LIST_CREDIT_Q.contains(&risk_class) || LIST_CREDIT_NON_Q.contains(&risk_class) {
                // Credit
                if let Some(idx) = index {
                    rho = wnc.rho(risk_class, idx[i], idx[j], None).unwrap_or(1.0);
                }

                // Calculate f for credit
                if let Some(cr) = list_cr {
                    let min_cr = cr[i].min(cr[j]);
                    let max_cr = cr[i].max(cr[j]);
                    if max_cr > 0.0 {
                        f = min_cr / max_cr;
                    }
                }

                phi = 1.0;
            } else {
                // Equity, Commodity, FX
                if LIST_EQUITY.contains(&risk_class) || LIST_COMMODITY.contains(&risk_class) {
                    if let Some(bkt) = bucket {
                        rho = wnc.rho(risk_class, "", "", Some(bkt[0])).unwrap_or(1.0);
                    }
                } else if LIST_FX.contains(&risk_class) {
                    // FX correlation logic
                    if let Some(bkt) = bucket {
                        let currency1 = bkt[i];
                        let currency2 = bkt[j];

                        let is_ccy1_high_vol = HIGH_VOL_CURRENCY_GROUP.contains(&currency1);
                        let is_ccy2_high_vol = HIGH_VOL_CURRENCY_GROUP.contains(&currency2);
                        let is_calc_ccy_high_vol = HIGH_VOL_CURRENCY_GROUP.contains(&calculation_currency);

                        if !is_calc_ccy_high_vol {
                            // Regular volatility calculation currency
                            rho = match (is_ccy1_high_vol, is_ccy2_high_vol) {
                                (true, true) => fx_reg_vol_corr(VolatilityLevel::High, VolatilityLevel::High),
                                (true, false) => fx_reg_vol_corr(VolatilityLevel::High, VolatilityLevel::Regular),
                                (false, true) => fx_reg_vol_corr(VolatilityLevel::Regular, VolatilityLevel::High),
                                (false, false) => fx_reg_vol_corr(VolatilityLevel::Regular, VolatilityLevel::Regular),
                            };
                        } else {
                            // High volatility calculation currency
                            rho = match (is_ccy1_high_vol, is_ccy2_high_vol) {
                                (true, true) => fx_high_vol_corr(VolatilityLevel::High, VolatilityLevel::High),
                                (true, false) => fx_high_vol_corr(VolatilityLevel::High, VolatilityLevel::Regular),
                                (false, true) => fx_high_vol_corr(VolatilityLevel::Regular, VolatilityLevel::High),
                                (false, false) => fx_high_vol_corr(VolatilityLevel::Regular, VolatilityLevel::Regular),
                            };
                        }
                    }
                }

                // Calculate f for non-rates
                if let Some(cr) = list_cr {
                    let min_cr = cr[i].min(cr[j]);
                    let max_cr = cr[i].max(cr[j]);
                    if max_cr > 0.0 {
                        f = min_cr / max_cr;
                    }
                }

                phi = 1.0;
            }

            k += rho * list_ws[i] * list_ws[j] * phi * f;
        }
    }

    k.sqrt()
}

/// Compute vega capital charge K for a given risk class and bucket
///
/// # Arguments
/// * `wnc` - Weights and correlations provider
/// * `risk_class` - Risk class
/// * `vr` - List of vega risk values
/// * `vcr` - List of vega concentration risk factors (optional)
/// * `bucket` - Bucket identifier (optional)
/// * `index` - Index labels (optional)
pub fn k_vega(
    wnc: &dyn WeightsAndCorr,
    risk_class: &str,
    vr: &[f64],
    vcr: Option<&[f64]>,
    bucket: Option<&str>,
    index: Option<&[&str]>,
) -> f64 {
    let n = vr.len();

    // K = sum of VR^2
    let mut k: f64 = vr.iter().map(|v| v.powi(2)).sum();

    // Create empty index if not provided
    let empty_index = vec![""; n];
    let idx = index.unwrap_or(&empty_index);

    // Add correlation terms
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }

            let mut rho: f64 = 1.0;
            let mut f: f64 = 1.0;

            // Calculate rho based on risk class
            if risk_class == "Rates" {
                if idx[i] == "Inf" && idx[j] == "Inf" {
                    rho = 1.0;
                } else if idx[i] == "Inf" || idx[j] == "Inf" {
                    rho = INFLATION_CORR;
                } else {
                    rho = wnc.rho("Risk_IRVol", idx[i], idx[j], None).unwrap_or(1.0);
                }

                f = 1.0; // For rates, f is always 1
            } else if LIST_EQUITY.contains(&risk_class) || LIST_COMMODITY.contains(&risk_class) {
                rho = wnc.rho(risk_class, "", "", bucket).unwrap_or(1.0);

                // Calculate f
                if let Some(vcr_vals) = vcr {
                    let min_vcr = vcr_vals[i].min(vcr_vals[j]);
                    let max_vcr = vcr_vals[i].max(vcr_vals[j]);
                    if max_vcr > 0.0 {
                        f = min_vcr / max_vcr;
                    }
                }
            } else if LIST_FX.contains(&risk_class) {
                rho = FX_VEGA_CORR;

                // Calculate f
                if let Some(vcr_vals) = vcr {
                    let min_vcr = vcr_vals[i].min(vcr_vals[j]);
                    let max_vcr = vcr_vals[i].max(vcr_vals[j]);
                    if max_vcr > 0.0 {
                        f = min_vcr / max_vcr;
                    }
                }
            } else if risk_class == "Risk_CreditVol" || risk_class == "Risk_CreditVolNonQ" {
                rho = wnc.rho(risk_class, idx[i], idx[j], None).unwrap_or(1.0);

                // Calculate f
                if let Some(vcr_vals) = vcr {
                    let min_vcr = vcr_vals[i].min(vcr_vals[j]);
                    let max_vcr = vcr_vals[i].max(vcr_vals[j]);
                    if max_vcr > 0.0 {
                        f = min_vcr / max_vcr;
                    }
                }
            }

            k += f * rho * vr[i] * vr[j];
        }
    }

    k.sqrt()
}

/// Compute curvature capital charge K for a given risk class and bucket
///
/// # Arguments
/// * `wnc` - Weights and correlations provider
/// * `risk_class` - Risk class
/// * `cvr_list` - List of curvature risk values
/// * `bucket` - Bucket identifier (optional)
/// * `index` - Index labels (optional)
pub fn k_curvature(
    wnc: &dyn WeightsAndCorr,
    risk_class: &str,
    cvr_list: &[f64],
    bucket: Option<&str>,
    index: Option<&[&str]>,
) -> f64 {
    let n = cvr_list.len();

    // K = sum of CVR^2
    let mut k: f64 = cvr_list.iter().map(|cvr| cvr.powi(2)).sum();

    // Create empty index if not provided
    let empty_index = vec![""; n];
    let idx = index.unwrap_or(&empty_index);

    // Add correlation terms
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }

            let mut rho: f64 = 1.0;

            // Calculate rho based on risk class
            if risk_class == "Rates" {
                if idx[i] == "Inf" && idx[j] == "Inf" {
                    rho = 1.0;
                } else if idx[i] == "Inf" || idx[j] == "Inf" {
                    rho = INFLATION_CORR;
                } else {
                    rho = wnc.rho("Risk_IRVol", idx[i], idx[j], None).unwrap_or(1.0);
                }
            } else if LIST_EQUITY.contains(&risk_class) || LIST_COMMODITY.contains(&risk_class) {
                rho = wnc.rho(risk_class, "", "", bucket).unwrap_or(1.0);
            } else if LIST_FX.contains(&risk_class) {
                rho = FX_VEGA_CORR;
            } else if risk_class == "Risk_CreditVol" || risk_class == "Risk_CreditVolNonQ" {
                rho = wnc.rho(risk_class, idx[i], idx[j], None).unwrap_or(1.0);
            }

            k += rho.powi(2) * cvr_list[i] * cvr_list[j];
        }
    }

    k.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2_5::V2_5;

    #[test]
    fn test_k_delta_simple() {
        let wnc = V2_5;
        let list_ws = vec![100.0, 200.0, 150.0];
        let result = k_delta(&wnc, "Risk_CreditQ", &list_ws, None, None, None, None, "USD");
        assert!(result > 0.0);
    }

    #[test]
    fn test_k_vega_simple() {
        let wnc = V2_5;
        let vr = vec![50.0, 75.0, 100.0];
        let result = k_vega(&wnc, "Risk_CreditVol", &vr, None, None, None);
        assert!(result > 0.0);
    }

    #[test]
    fn test_k_curvature_simple() {
        let wnc = V2_5;
        let cvr = vec![30.0, 40.0, 50.0];
        let index = vec!["2y", "5y", "10y"];
        let result = k_curvature(&wnc, "Rates", &cvr, None, Some(&index));
        assert!(result > 0.0);
    }
}
