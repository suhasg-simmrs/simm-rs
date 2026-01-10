use crate::constants::SIMM_TENOR_LIST;

/// Type alias for CRIF data (Common Risk Interchange Format)
/// First row is the header, subsequent rows are data
pub type Crif = Vec<Vec<String>>;

/// Get the index of a column by name from the header row
pub fn get_column_index(crif: &Crif, column_name: &str) -> Option<usize> {
    if crif.is_empty() {
        return None;
    }
    crif[0].iter().position(|x| x == column_name)
}

/// Get all values from a column
pub fn get_column_values(crif: &Crif, column_name: &str) -> Vec<Option<String>> {
    let idx = match get_column_index(crif, column_name) {
        Some(i) => i,
        None => return Vec::new(),
    };

    crif.iter()
        .skip(1)
        .map(|row| {
            if idx < row.len() {
                Some(row[idx].clone())
            } else {
                None
            }
        })
        .collect()
}

/// Calculate Concentration Threshold
pub fn concentration_threshold(sum_s: f64, t: f64) -> f64 {
    1.0_f64.max((sum_s.abs() / t).sqrt())
}

/// Sum Sensitivities (AmountUSD) from CRIF
pub fn sum_sensitivities(crif: &Crif) -> f64 {
    let amount_values = get_column_values(crif, "AmountUSD");
    let mut total = 0.0;

    for val in amount_values {
        if let Some(v) = val {
            if !v.is_empty() {
                if let Ok(num) = v.parse::<f64>() {
                    total += num;
                }
            }
        }
    }

    total
}

/// Extract tenors as a list from CRIF
pub fn tenor_list(crif: &Crif) -> Vec<String> {
    let label_1_values = get_column_values(crif, "Label1");
    let mut tenors = Vec::new();

    for x in label_1_values {
        if let Some(val) = x {
            let tenor = val.to_lowercase();
            if SIMM_TENOR_LIST.contains(&tenor.as_str()) && !tenors.contains(&tenor) {
                tenors.push(tenor);
            }
        }
    }

    tenors
}

/// Extract distinct values only from a list
#[allow(dead_code)]
pub fn unique_list(values: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for val in values {
        if seen.insert(val.clone()) {
            result.push(val);
        }
    }

    result
}

/// Extract unique values from a column in CRIF
pub fn unique_column_values(crif: &Crif, column_name: &str) -> Vec<String> {
    let values = get_column_values(crif, column_name);
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for val in values {
        if let Some(v) = val {
            if !v.is_empty() && v != "nan" && seen.insert(v.clone()) {
                result.push(v);
            }
        }
    }

    result
}

/// Extract currency pairs from CRIF as a list
pub fn currency_pair_list(crif: &Crif) -> Vec<String> {
    let qualifier_values = get_column_values(crif, "Qualifier");
    let mut currency_pairs = Vec::new();

    for x in qualifier_values {
        if let Some(val) = x {
            if !val.is_empty() && val != "nan" && val.len() == 6 {
                // Prevent duplicates (e.g., KRWUSD is identical to USDKRW)
                let reversed = format!("{}{}", &val[3..6], &val[0..3]);
                if !currency_pairs.contains(&reversed) {
                    currency_pairs.push(val);
                }
            }
        }
    }

    // Remove duplicates
    let mut seen = std::collections::HashSet::new();
    currency_pairs.retain(|x| seen.insert(x.clone()));

    currency_pairs
}

/// Extract product classes from CRIF
pub fn product_list(crif: &Crif) -> Vec<String> {
    let product_values = get_column_values(crif, "ProductClass");
    let mut products = Vec::new();

    for x in product_values {
        if let Some(val) = x {
            if !val.is_empty() && val != "nan" {
                products.push(val);
            }
        }
    }

    // Remove duplicates
    let mut seen = std::collections::HashSet::new();
    products.retain(|x| seen.insert(x.clone()));

    products
}

/// Extract buckets from CRIF
pub fn bucket_list(crif: &Crif) -> Vec<usize> {
    let bucket_values = get_column_values(crif, "Bucket");
    let mut buckets = Vec::new();
    let mut has_residual = false;

    for x in bucket_values {
        if let Some(val) = x {
            if val == "Residual" {
                has_residual = true;
            } else if !val.is_empty() && val != "nan" {
                if let Ok(num) = val.parse::<usize>() {
                    buckets.push(num);
                }
            }
        }
    }

    // Residual goes to 0
    if has_residual {
        buckets.push(0);
    }

    // Remove duplicates
    let mut seen = std::collections::HashSet::new();
    buckets.retain(|x| seen.insert(*x));

    buckets
}

/// Scaling Function of time t (for Curvature Margin)
pub fn scaling_func(t: &str) -> f64 {
    let t_lower = t.to_lowercase();

    if t_lower == "2w" {
        return 0.5;
    } else if t_lower.contains('m') {
        // Month tenor
        if let Ok(months) = t_lower.replace('m', "").parse::<f64>() {
            let t_days = (365.0 / 12.0) * months;
            return 0.5 * (1.0_f64).min(14.0 / t_days);
        }
    } else if t_lower.contains('y') {
        // Year tenor
        if let Ok(years) = t_lower.replace('y', "").parse::<f64>() {
            let t_days = 365.0 * years;
            return 0.5 * (1.0_f64).min(14.0 / t_days);
        }
    }

    0.5 // Default fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concentration_threshold() {
        let result = concentration_threshold(100.0, 25.0);
        assert_eq!(result, 2.0);

        let result2 = concentration_threshold(10.0, 100.0);
        assert_eq!(result2, 1.0); // max(1, sqrt(0.1)) = 1
    }

    #[test]
    fn test_scaling_func() {
        assert_eq!(scaling_func("2w"), 0.5);

        let result_1y = scaling_func("1y");
        assert!(result_1y > 0.0 && result_1y <= 0.5);

        let result_3m = scaling_func("3m");
        assert!(result_3m > 0.0 && result_3m <= 0.5);
    }

    #[test]
    fn test_get_column_index() {
        let crif = vec![
            vec!["ProductClass".to_string(), "RiskType".to_string(), "Amount".to_string()],
            vec!["Rates".to_string(), "Risk_IRCurve".to_string(), "1000".to_string()],
        ];

        assert_eq!(get_column_index(&crif, "RiskType"), Some(1));
        assert_eq!(get_column_index(&crif, "NonExistent"), None);
    }
}
