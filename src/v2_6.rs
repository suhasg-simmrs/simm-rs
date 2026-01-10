#![allow(dead_code)]

use crate::constants::*;
use crate::wnc::WeightsAndCorr;

/// SIMM version 2.6 parameter set
pub struct V2_6;

impl V2_6 {
    fn matrix_lookup_12(
        data: &[[f64; 12]; 12],
        row_labels: &[&str],
        col_labels: &[&str],
        row_key: &str,
        col_key: &str,
    ) -> f64 {
        let i = row_labels
            .iter()
            .position(|x| *x == row_key)
            .unwrap_or_else(|| panic!("Row key not found: {}", row_key));
        let j = col_labels
            .iter()
            .position(|x| *x == col_key)
            .unwrap_or_else(|| panic!("Col key not found: {}", col_key));

        data[i][j]
    }

    fn matrix_lookup_17(
        data: &[[f64; 17]; 17],
        row_labels: &[&str],
        col_labels: &[&str],
        row_key: &str,
        col_key: &str,
    ) -> f64 {
        let i = row_labels
            .iter()
            .position(|x| *x == row_key)
            .unwrap_or_else(|| panic!("Row key not found: {}", row_key));
        let j = col_labels
            .iter()
            .position(|x| *x == col_key)
            .unwrap_or_else(|| panic!("Col key not found: {}", col_key));

        data[i][j]
    }

    fn matrix_lookup_6(
        data: &[[f64; 6]; 6],
        row_labels: &[&str],
        col_labels: &[&str],
        row_key: &str,
        col_key: &str,
    ) -> f64 {
        let i = row_labels
            .iter()
            .position(|x| *x == row_key)
            .unwrap_or_else(|| panic!("Row key not found: {}", row_key));
        let j = col_labels
            .iter()
            .position(|x| *x == col_key)
            .unwrap_or_else(|| panic!("Col key not found: {}", col_key));

        data[i][j]
    }
}

/* ===========================
   === SIMM v2.6 PARAMETERS ==
   ===========================

   Based on https://www.isda.org/a/b4ugE/ISDA-SIMM_v2.6_PUBLIC.pdf
*/

pub const REG_VOL_CCY_BUCKET: [&str; 14] = [
    "USD", "EUR", "GBP", "CHF", "AUD", "NZD", "CAD",
    "SEK", "NOK", "DKK", "HKD", "KRW", "SGD", "TWD",
];

pub const LOW_VOL_CCY_BUCKET: [&str; 1] = ["JPY"];

pub static REG_VOL_RW: &[(&str, f64)] = &[
    ("2w", 109.0),
    ("1m", 105.0),
    ("3m", 90.0),
    ("6m", 71.0),
    ("1y", 66.0),
    ("2y", 66.0),
    ("3y", 64.0),
    ("5y", 60.0),
    ("10y", 60.0),
    ("15y", 61.0),
    ("20y", 61.0),
    ("30y", 67.0),
];

pub fn reg_vol_rw_lookup(tenor: &str) -> f64 {
    REG_VOL_RW.iter()
        .find(|(t, _)| *t == tenor)
        .unwrap_or_else(|| panic!("Unknown tenor: {}", tenor))
        .1
}

pub static LOW_VOL_RW: &[(&str, f64)] = &[
    ("2w",  15.0),
    ("1m",  18.0),
    ("3m",   9.0),
    ("6m",  11.0),
    ("1y",  13.0),
    ("2y",  15.0),
    ("3y",  19.0),
    ("5y",  23.0),
    ("10y", 23.0),
    ("15y", 22.0),
    ("20y", 22.0),
    ("30y", 23.0),
];

pub fn low_vol_rw_lookup(tenor: &str) -> f64 {
    LOW_VOL_RW
        .iter()
        .find(|(t, _)| *t == tenor)
        .unwrap_or_else(|| panic!("Unknown tenor: {}", tenor))
        .1
}

pub static HIGH_VOL_RW: &[(&str, f64)] = &[
    ("2w",   163.0),
    ("1m",   109.0),
    ("3m",    87.0),
    ("6m",    89.0),
    ("1y",   102.0),
    ("2y",    96.0),
    ("3y",   101.0),
    ("5y",    97.0),
    ("10y",   97.0),
    ("15y",  102.0),
    ("20y",  106.0),
    ("30y",  101.0),
];

pub fn high_vol_rw_lookup(tenor: &str) -> f64 {
    HIGH_VOL_RW
        .iter()
        .find(|(t, _)| *t == tenor)
        .unwrap_or_else(|| panic!("Unknown tenor: {}", tenor))
        .1
}

pub const INFLATION_RW : f64 = 61.0;
pub const CCY_BASIS_SWAP_SPREAD_RW : f64 = 21.0;
pub const IR_HVR : f64 = 0.47;
pub const IR_VRW : f64 = 0.23;

pub const IR_CORR: [[f64; 12]; 12] = [
    [1.00, 0.77, 0.67, 0.59, 0.48, 0.39, 0.34, 0.30, 0.25, 0.23, 0.21, 0.20],
    [0.77, 1.00, 0.84, 0.74, 0.56, 0.43, 0.36, 0.31, 0.26, 0.21, 0.19, 0.19],
    [0.67, 0.84, 1.00, 0.88, 0.69, 0.55, 0.47, 0.40, 0.34, 0.27, 0.25, 0.25],
    [0.59, 0.74, 0.88, 1.00, 0.86, 0.73, 0.65, 0.57, 0.49, 0.40, 0.38, 0.37],
    [0.48, 0.56, 0.69, 0.86, 1.00, 0.94, 0.87, 0.79, 0.68, 0.60, 0.57, 0.55],
    [0.39, 0.43, 0.55, 0.73, 0.94, 1.00, 0.96, 0.91, 0.80, 0.74, 0.70, 0.69],
    [0.34, 0.36, 0.47, 0.65, 0.87, 0.96, 1.00, 0.97, 0.88, 0.81, 0.77, 0.76],
    [0.30, 0.31, 0.40, 0.57, 0.79, 0.91, 0.97, 1.00, 0.95, 0.90, 0.86, 0.85],
    [0.25, 0.26, 0.34, 0.49, 0.68, 0.80, 0.88, 0.95, 1.00, 0.97, 0.94, 0.94],
    [0.23, 0.21, 0.27, 0.40, 0.60, 0.74, 0.81, 0.90, 0.97, 1.00, 0.98, 0.97],
    [0.21, 0.19, 0.25, 0.38, 0.57, 0.70, 0.77, 0.86, 0.94, 0.98, 1.00, 0.99],
    [0.20, 0.19, 0.25, 0.37, 0.55, 0.69, 0.76, 0.85, 0.94, 0.97, 0.99, 1.00],
];

pub const SUB_CURVES_CORR: f64 = 0.993;
pub const INFLATION_CORR: f64 = 0.24;
pub const CCY_BASIS_SPREAD_CORR: f64 = 0.04;
pub const IR_GAMMA_DIFF_CCY: f64 = 0.32;

pub const CREDIT_Q_RW: [f64; 13] = [
    343.0, // 0 = Residual
    75.0,  // 1
    90.0,  // 2
    84.0,  // 3
    54.0,  // 4
    62.0,  // 5
    48.0,  // 6
    185.0, // 7
    343.0, // 8
    255.0, // 9
    250.0, // 10
    214.0, // 11
    173.0, // 12
];

pub const CREDIT_Q_V_RW: f64 = 0.76;
pub const CREDIT_Q_BASE_CORR_WEIGHT: f64 = 10.0;
pub const CREDIT_Q_CORR: [f64; 4] = [0.93, 0.46, 0.5, 0.29];

pub const CREDIT_Q_CORR_NON_RES: [[f64; 12]; 12] = [
    [1.00, 0.38, 0.38, 0.35, 0.37, 0.34, 0.42, 0.32, 0.34, 0.33, 0.34, 0.33],
    [0.38, 1.00, 0.48, 0.46, 0.48, 0.46, 0.39, 0.40, 0.41, 0.41, 0.43, 0.40],
    [0.38, 0.48, 1.00, 0.50, 0.51, 0.50, 0.40, 0.39, 0.45, 0.44, 0.47, 0.42],
    [0.35, 0.46, 0.50, 1.00, 0.50, 0.50, 0.37, 0.37, 0.41, 0.43, 0.45, 0.40],
    [0.37, 0.48, 0.51, 0.50, 1.00, 0.50, 0.39, 0.38, 0.43, 0.43, 0.46, 0.42],
    [0.34, 0.46, 0.50, 0.50, 0.50, 1.00, 0.37, 0.35, 0.39, 0.41, 0.44, 0.41],
    [0.42, 0.39, 0.40, 0.37, 0.39, 0.37, 1.00, 0.33, 0.37, 0.37, 0.35, 0.35],
    [0.32, 0.40, 0.39, 0.37, 0.38, 0.35, 0.33, 1.00, 0.36, 0.37, 0.37, 0.36],
    [0.34, 0.41, 0.45, 0.41, 0.43, 0.39, 0.37, 0.36, 1.00, 0.41, 0.40, 0.38],
    [0.33, 0.41, 0.44, 0.43, 0.43, 0.41, 0.37, 0.37, 0.41, 1.00, 0.41, 0.39],
    [0.34, 0.43, 0.47, 0.45, 0.46, 0.44, 0.35, 0.37, 0.40, 0.41, 1.00, 0.40],
    [0.33, 0.40, 0.42, 0.40, 0.42, 0.41, 0.35, 0.36, 0.38, 0.39, 0.40, 1.00],
];

pub const CREDIT_NON_Q_RW: [f64;3] = [
    1300.0, // 0 = Residual
    280.0,  // 1
    1300.0  // 2
];

pub const CREDIT_NON_Q_VRW: f64 = 0.76;
pub const CREDIT_NON_Q_CORR: [f64; 3] = [0.83, 0.32, 0.5];
pub const CR_GAMMA_DIFF_CCY: f64 = 0.43;

pub const EQUITY_RW: [f64; 13] = [
    50.0, // 0 = Residual
    30.0, // 1
    33.0, // 2
    36.0, // 3
    29.0, // 4
    26.0, // 5
    25.0, // 6
    34.0, // 7
    28.0, // 8
    36.0, // 9
    50.0, // 10
    19.0, // 11
    19.0, // 12
];

pub const EQUITY_HVR: f64 = 0.60;
pub const EQUITY_VRW: f64 = 0.45;
pub const EQUITY_VRW_BUCKET_12: f64 = 0.96;

pub const EQUITY_CORR: [f64; 13] = [
    0.0,  // 0 = Residual
    0.18, // 1
    0.20, // 2
    0.28, // 3
    0.24, // 4
    0.25, // 5
    0.36, // 6
    0.35, // 7
    0.37, // 8
    0.23, // 9
    0.27, // 10
    0.45, // 11
    0.45, // 12
];

pub const EQUITY_CORR_NON_RES: [[f64; 12]; 12] = [
    [1.00, 0.18, 0.19, 0.19, 0.14, 0.16, 0.15, 0.16, 0.18, 0.12, 0.19, 0.19],
    [0.18, 1.00, 0.22, 0.21, 0.15, 0.18, 0.17, 0.19, 0.20, 0.14, 0.21, 0.21],
    [0.19, 0.22, 1.00, 0.22, 0.13, 0.16, 0.18, 0.17, 0.22, 0.13, 0.20, 0.20],
    [0.19, 0.21, 0.22, 1.00, 0.17, 0.22, 0.22, 0.23, 0.22, 0.17, 0.26, 0.26],
    [0.14, 0.15, 0.13, 0.17, 1.00, 0.29, 0.26, 0.29, 0.14, 0.24, 0.32, 0.32],
    [0.16, 0.18, 0.16, 0.22, 0.29, 1.00, 0.34, 0.36, 0.17, 0.30, 0.39, 0.39],
    [0.15, 0.17, 0.18, 0.22, 0.26, 0.34, 1.00, 0.33, 0.16, 0.28, 0.36, 0.36],
    [0.16, 0.19, 0.17, 0.23, 0.29, 0.36, 0.33, 1.00, 0.17, 0.29, 0.40, 0.40],
    [0.18, 0.20, 0.22, 0.22, 0.14, 0.17, 0.16, 0.17, 1.00, 0.13, 0.21, 0.21],
    [0.12, 0.14, 0.13, 0.17, 0.24, 0.30, 0.28, 0.29, 0.13, 1.00, 0.30, 0.30],
    [0.19, 0.21, 0.20, 0.26, 0.32, 0.39, 0.36, 0.40, 0.21, 0.30, 1.00, 0.45],
    [0.19, 0.21, 0.20, 0.26, 0.32, 0.39, 0.36, 0.40, 0.21, 0.30, 0.45, 1.00],
];

pub const COMMODITY_RW: [f64; 18] = [
    0.0,  // 0 = unused (or could be Residual if you have one)
    48.0, // 1
    29.0, // 2
    33.0, // 3
    25.0, // 4
    35.0, // 5
    30.0, // 6
    60.0, // 7
    52.0, // 8
    68.0, // 9
    63.0, // 10
    21.0, // 11
    21.0, // 12
    15.0, // 13
    16.0, // 14
    13.0, // 15
    68.0, // 16
    17.0, // 17
];

pub const COMMODITY_HVR : f64 = 0.74;
pub const COMMODITY_VRW : f64 = 0.55;

pub const COMMODITY_CORR: [f64; 18] = [
    0.0,  // dummy for zero element
    0.83, // 1
    0.97, // 2
    0.93, // 3
    0.97, // 4
    0.98, // 5
    0.90, // 6
    0.98, // 7
    0.49, // 8
    0.80, // 9
    0.46, // 10
    0.58, // 11
    0.53, // 12
    0.62, // 13
    0.16, // 14
    0.18, // 15
    0.00, // 16
    0.38, // 17
];

pub const COMMODITY_CORR_NON_RES: [[f64; 17]; 17] = [
    [1.00, 0.22, 0.18, 0.21, 0.20, 0.24,  0.49,  0.16,  0.38, 0.14, 0.10,  0.02, 0.12, 0.11,  0.02, 0.00, 0.17],
    [0.22, 1.00, 0.92, 0.90, 0.88, 0.25,  0.08,  0.19,  0.17, 0.17, 0.42,  0.28, 0.36, 0.27,  0.20, 0.00, 0.64],
    [0.18, 0.92, 1.00, 0.87, 0.84, 0.16,  0.07,  0.15,  0.10, 0.18, 0.33,  0.22, 0.27, 0.23,  0.16, 0.00, 0.54],
    [0.21, 0.90, 0.87, 1.00, 0.77, 0.19,  0.11,  0.18,  0.16, 0.14, 0.32,  0.22, 0.28, 0.22,  0.11, 0.00, 0.58],
    [0.20, 0.88, 0.84, 0.77, 1.00, 0.19,  0.09,  0.12,  0.13, 0.18, 0.42,  0.34, 0.32, 0.29,  0.13, 0.00, 0.59],
    [0.24, 0.25, 0.16, 0.19, 0.19, 1.00,  0.31,  0.62,  0.23, 0.10, 0.21,  0.05, 0.18, 0.10,  0.08, 0.00, 0.28],
    [0.49, 0.08, 0.07, 0.11, 0.09, 0.31,  1.00,  0.21,  0.79, 0.17, 0.10, -0.08, 0.10, 0.07, -0.02, 0.00, 0.13],
    [0.16, 0.19, 0.15, 0.18, 0.12, 0.62,  0.21,  1.00,  0.16, 0.08, 0.13, -0.07, 0.07, 0.05,  0.02, 0.00, 0.19],
    [0.38, 0.17, 0.10, 0.16, 0.13, 0.23,  0.79,  0.16,  1.00, 0.15, 0.09, -0.06, 0.06, 0.06,  0.01, 0.00, 0.16],
    [0.14, 0.17, 0.18, 0.14, 0.18, 0.10,  0.17,  0.08,  0.15, 1.00, 0.16,  0.09, 0.14, 0.09,  0.03, 0.00, 0.11],
    [0.10, 0.42, 0.33, 0.32, 0.42, 0.21,  0.10,  0.13,  0.09, 0.16, 1.00,  0.36, 0.30, 0.25,  0.18, 0.00, 0.37],
    [0.02, 0.28, 0.22, 0.22, 0.34, 0.05, -0.08, -0.07, -0.06, 0.09, 0.36,  1.00, 0.20, 0.18,  0.11, 0.00, 0.26],
    [0.12, 0.36, 0.27, 0.28, 0.32, 0.18,  0.10,  0.07,  0.06, 0.14, 0.30,  0.20, 1.00, 0.28,  0.19, 0.00, 0.39],
    [0.11, 0.27, 0.23, 0.22, 0.29, 0.10,  0.07,  0.05,  0.06, 0.09, 0.25,  0.18, 0.28, 1.00,  0.13, 0.00, 0.26],
    [0.02, 0.20, 0.16, 0.11, 0.13, 0.08, -0.02,  0.02,  0.01, 0.03, 0.18,  0.11, 0.19, 0.13,  1.00, 0.00, 0.21],
    [0.00, 0.00, 0.00, 0.00, 0.00, 0.00,  0.00,  0.00,  0.00, 0.00, 0.00,  0.00, 0.00, 0.00,  0.00, 1.00, 0.00],
    [0.17, 0.64, 0.54, 0.58, 0.59, 0.28,  0.13,  0.19,  0.16, 0.11, 0.37,  0.26, 0.39, 0.26,  0.21, 0.00, 1.00],
];

pub const HIGH_VOL_CURRENCY_GROUP : &[&str] = &["BRL", "RUB", "TRY"];

#[derive(Debug, Clone, Copy)]
pub enum RiskLevel {
    Regular,
    High,
}

pub const fn fx_rw(outer: RiskLevel, inner: RiskLevel) -> f64 {
    match (outer, inner) {
        (RiskLevel::Regular, RiskLevel::Regular) => 7.4,
        (RiskLevel::Regular, RiskLevel::High) => 14.7,
        (RiskLevel::High, RiskLevel::Regular) => 14.7,
        (RiskLevel::High, RiskLevel::High) => 21.4,
    }
}

pub const FX_HVR : f64 = 0.57;
pub const FX_VRW : f64 = 0.48;

#[derive(Debug, Clone, Copy)]
pub enum VolatilityLevel {
    Regular,
    High,
}

impl VolatilityLevel {
    const fn as_index(self) -> usize {
        match self {
            VolatilityLevel::Regular => 0,
            VolatilityLevel::High => 1,
        }
    }
}

// Regular Volatility FX Correlations
pub const FX_REG_VOL_CORR: [[f64; 2]; 2] = [
    [0.50, 0.25],  // Regular -> [Regular, High]
    [0.25, -0.05], // High -> [Regular, High]
];

// High Volatility FX Correlations
pub const FX_HIGH_VOL_CORR: [[f64; 2]; 2] = [
    [0.88, 0.72],  // Regular -> [Regular, High]
    [0.72, 0.50],  // High -> [Regular, High]
];

// Helper functions for cleaner access
pub const fn fx_reg_vol_corr(outer: VolatilityLevel, inner: VolatilityLevel) -> f64 {
    FX_REG_VOL_CORR[outer.as_index()][inner.as_index()]
}

pub const fn fx_high_vol_corr(outer: VolatilityLevel, inner: VolatilityLevel) -> f64 {
    FX_HIGH_VOL_CORR[outer.as_index()][inner.as_index()]
}

pub const FX_VEGA_CORR:f64 = 0.5;

static SIMM_TENORS: &[&str] = &[
    "2w", "1m", "3m", "6m", "1y", "2y", "3y", "5y", "10y", "15y", "20y", "30y"
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    AUD,
    CAD,
    CHF,
    DKK,
    HKD,
    KRW,
    NOK,
    NZD,
    SEK,
    SGD,
    TWD,
    JPY,
    Others,
}

impl Currency {
    const fn ir_delta_ct(self) -> u32 {
        match self {
            // Regular volatility, well-traded
            Currency::USD | Currency::EUR | Currency::GBP => 330,

            // Regular volatility, less well-traded
            Currency::AUD | Currency::CAD | Currency::CHF |
            Currency::DKK | Currency::HKD | Currency::KRW |
            Currency::NOK | Currency::NZD | Currency::SEK |
            Currency::SGD | Currency::TWD => 130,

            // Low volatility
            Currency::JPY => 61,

            // All other currencies
            Currency::Others => 30,
        }
    }

    const fn ir_vega_ct(self) -> u32 {
        match self {
            // Regular volatility, well-traded
            Currency::USD | Currency::EUR | Currency::GBP => 4900,

            // Regular volatility, less well-traded
            Currency::AUD | Currency::CAD | Currency::CHF |
            Currency::DKK | Currency::HKD | Currency::KRW |
            Currency::NOK | Currency::NZD | Currency::SEK |
            Currency::SGD | Currency::TWD => 520,

            // Low volatility
            Currency::JPY => 970,

            // All other currencies
            Currency::Others => 74,
        }
    }
}

impl Currency {
    fn from_str(s: &str) -> Self {
        match s {
            "USD" => Currency::USD,
            "EUR" => Currency::EUR,
            "GBP" => Currency::GBP,
            "AUD" => Currency::AUD,
            "CAD" => Currency::CAD,
            "CHF" => Currency::CHF,
            "DKK" => Currency::DKK,
            "HKD" => Currency::HKD,
            "KRW" => Currency::KRW,
            "NOK" => Currency::NOK,
            "NZD" => Currency::NZD,
            "SEK" => Currency::SEK,
            "SGD" => Currency::SGD,
            "TWD" => Currency::TWD,
            "JPY" => Currency::JPY,
            _ => Currency::Others,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditQuality {
    Qualifying,
    NonQualifying,
}

impl CreditQuality {
    const fn delta_ct(self, bucket: u8) -> f64 {
        match (self, bucket) {
            // Qualifying
            (CreditQuality::Qualifying, 1) => 1.00,  // Sovereigns including central banks
            (CreditQuality::Qualifying, 7) => 1.00,  // Sovereigns including central banks
            (CreditQuality::Qualifying, 2..=6) => 0.17,  // Corporate entities
            (CreditQuality::Qualifying, 8..=12) => 0.17, // Corporate entities
            (CreditQuality::Qualifying, 0) => 0.17,  // Residual

            // Non-Qualifying
            (CreditQuality::NonQualifying, 1) => 9.5,  // Investment Grades (RMBS & CMBS)
            (CreditQuality::NonQualifying, 2) => 0.5,  // High Yield/Non-rated (RMBS & CMBS)
            (CreditQuality::NonQualifying, 0) => 0.5,  // Residual

            // Default case for any other bucket numbers
            _ => 0.17,
        }
    }

    const fn vega_ct(self) -> u32 {
        match self {
            CreditQuality::Qualifying => 360,
            CreditQuality::NonQualifying => 70,
        }
    }
}

pub const CREDIT_DELTA_CT_QUALIFYING: [f64; 13] = [
    0.17, // 0: Residual
    1.00, // 1: Sovereigns including central banks
    0.17, // 2: Corporate entities
    0.17, // 3: Corporate entities
    0.17, // 4: Corporate entities
    0.17, // 5: Corporate entities
    0.17, // 6: Corporate entities
    1.00, // 7: Sovereigns including central banks
    0.17, // 8: Corporate entities
    0.17, // 9: Corporate entities
    0.17, // 10: Corporate entities
    0.17, // 11: Corporate entities
    0.17, // 12: Corporate entities
];

pub const CREDIT_DELTA_CT_NON_QUALIFYING: [f64; 3] = [
    0.5,  // 0: Residual
    9.5,  // 1: Investment Grades (RMBS & CMBS)
    0.5,  // 2: High Yield/Non-rated (RMBS & CMBS)
];

pub fn credit_delta_ct(quality: CreditQuality, bucket: u8) -> Option<f64> {
    match quality {
        CreditQuality::Qualifying => {
            CREDIT_DELTA_CT_QUALIFYING.get(bucket as usize).copied()
        }
        CreditQuality::NonQualifying => {
            CREDIT_DELTA_CT_NON_QUALIFYING.get(bucket as usize).copied()
        }
    }
}

pub const EQUITY_DELTA_CT: [f64; 13] = [
    0.37,  // 0: Residual - Not classified
    3.0,   // 1: Emerging Markets - Large Cap
    3.0,   // 2: Emerging Markets - Large Cap
    3.0,   // 3: Emerging Markets - Large Cap
    3.0,   // 4: Emerging Markets - Large Cap
    12.0,  // 5: Developed Markets - Large Cap
    12.0,  // 6: Developed Markets - Large Cap
    12.0,  // 7: Developed Markets - Large Cap
    12.0,  // 8: Developed Markets - Large Cap
    0.64,  // 9: Emerging Markets - Small Cap
    0.37,  // 10: Developed Markets - Small Cap
    810.0, // 11: Indexes, Funds, ETFs, Volatility Indexes
    810.0, // 12: Indexes, Funds, ETFs, Volatility Indexes
];

pub const COMMODITY_DELTA_CT: [f64; 18] = [
    52.0,   // 0: dummy (using "Other" value as default)
    310.0,  // 1: Coal
    2100.0, // 2: Crude Oil
    1700.0, // 3: Oil Fractions
    1700.0, // 4: Oil Fractions
    1700.0, // 5: Oil Fractions
    2800.0, // 6: Natural Gas
    2800.0, // 7: Natural Gas
    2700.0, // 8: Power
    2700.0, // 9: Power
    52.0,   // 10: Freight, Dry or Wet
    530.0,  // 11: Base metals
    1300.0, // 12: Precious Metals
    100.0,  // 13: Agricultural
    100.0,  // 14: Agricultural
    100.0,  // 15: Agricultural
    52.0,   // 16: Other
    4000.0, // 17: Indices
];

pub const FX_CATEGORY1: [&str; 7] = [
    "USD", "EUR", "JPY", "GBP", "AUD", "CHF", "CAD"
];

pub const FX_CATEGORY2: [&str; 13] = [
    "BRL", "CNY", "HKD", "INR", "KRW", "MXN", "NOK",
    "NZD", "RUB", "SEK", "SGD", "TRY", "ZAR"
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxCategory {
    Category1, // Significantly material
    Category2, // Frequently traded
    Others,    // All other currencies
}

impl FxCategory {
    const fn delta_ct(self) -> u32 {
        match self {
            FxCategory::Category1 => 3300, // Significantly material
            FxCategory::Category2 => 880,  // Frequently traded
            FxCategory::Others => 170,     // All other currencies
        }
    }

    const fn vega_ct(self, other: FxCategory) -> u32 {
        use FxCategory::*;

        match (self, other) {
            (Category1, Category1) => 2800,
            (Category1, Category2) | (Category2, Category1) => 1400,
            (Category1, Others) | (Others, Category1) => 590,
            (Category2, Category2) => 520,
            (Category2, Others) | (Others, Category2) => 340,
            (Others, Others) => 210,
        }
    }

    fn from_currency(currency: &str) -> Self {
        if FX_CATEGORY1.contains(&currency) {
            FxCategory::Category1
        } else if FX_CATEGORY2.contains(&currency) {
            FxCategory::Category2
        } else {
            FxCategory::Others
        }
    }
}

pub const EQUITY_VEGA_CT: [u32; 13] = [
    39,   // 0: Residual - Not classified
    210,  // 1: Emerging Markets - Large Cap
    210,  // 2: Emerging Markets - Large Cap
    210,  // 3: Emerging Markets - Large Cap
    210,  // 4: Emerging Markets - Large Cap
    1300, // 5: Developed Markets - Large Cap
    1300, // 6: Developed Markets - Large Cap
    1300, // 7: Developed Markets - Large Cap
    1300, // 8: Developed Markets - Large Cap
    39,   // 9: Emerging Markets - Small Cap
    190,  // 10: Developed Markets - Small Cap
    6400, // 11: Indexes, Funds, ETFs, Volatility Indexes
    6400, // 12: Indexes, Funds, ETFs, Volatility Indexes
];

pub const COMMODITY_VEGA_CT: [u32; 18] = [
    69,   // 0: dummy (using "Other" value as default)
    390,  // 1: Coal
    2900, // 2: Crude Oil
    310,  // 3: Oil Fractions
    310,  // 4: Oil Fractions
    310,  // 5: Oil Fractions
    6300, // 6: Natural Gas
    6300, // 7: Natural Gas
    1200, // 8: Power
    1200, // 9: Power
    120,  // 10: Freight, Dry or Wet
    390,  // 11: Base metals
    1300, // 12: Precious Metals
    590,  // 13: Agricultural
    590,  // 14: Agricultural
    590,  // 15: Agricultural
    69,   // 16: Other
    69,   // 17: Indices
];

pub const CORR_PARAMS: [[f64; 6]; 6] = [
    [1.00, 0.04, 0.04, 0.07, 0.37, 0.14],
    [0.04, 1.00, 0.54, 0.70, 0.27, 0.37],
    [0.04, 0.54, 1.00, 0.46, 0.24, 0.15],
    [0.07, 0.70, 0.46, 1.00, 0.35, 0.39],
    [0.37, 0.27, 0.24, 0.35, 1.00, 0.35],
    [0.14, 0.37, 0.15, 0.39, 0.35, 1.00],
];

static RISK_CLASSES: &[&str] = &["Rates", "CreditQ", "CreditNonQ", "Equity", "Commodity", "FX"];

static BUCKET_LIST_12: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"];
static BUCKET_LIST_17: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17"];

/* ===========================
   === TRAIT IMPLEMENTATION ==
   =========================== */

impl WeightsAndCorr for V2_6 {
    fn rw(&self, risk_class: &str, bucket: &str) -> Option<f64> {
        let bucket_idx = if bucket == "Residual" {
            0
        } else {
            bucket.parse::<usize>().ok()?
        };

        if LIST_CREDIT_Q.contains(&risk_class) {
            CREDIT_Q_RW.get(bucket_idx).copied()
        } else if LIST_CREDIT_NON_Q.contains(&risk_class) {
            CREDIT_NON_Q_RW.get(bucket_idx).copied()
        } else if LIST_EQUITY.contains(&risk_class) {
            EQUITY_RW.get(bucket_idx).copied()
        } else if LIST_COMMODITY.contains(&risk_class) {
            COMMODITY_RW.get(bucket_idx).copied()
        } else {
            None
        }
    }

    fn rho(&self, risk_class: &str, index1: &str, index2: &str, bucket: Option<&str>) -> Option<f64> {
        if LIST_RATES.contains(&risk_class) {
            Some(Self::matrix_lookup_12(&IR_CORR, SIMM_TENORS, SIMM_TENORS, index1, index2))
        } else if LIST_CREDIT_Q.contains(&risk_class) {
            if risk_class == "Risk_BaseCorr" {
                Some(CREDIT_Q_CORR[3])
            } else if index1 == "Res" || index2 == "Res" {
                Some(CREDIT_Q_CORR[2])
            } else if index1 == index2 {
                Some(CREDIT_Q_CORR[0])
            } else {
                Some(CREDIT_Q_CORR[1])
            }
        } else if LIST_CREDIT_NON_Q.contains(&risk_class) {
            if index1 == "Res" || index2 == "Res" {
                Some(CREDIT_NON_Q_CORR[2])
            } else if index1 == index2 {
                Some(CREDIT_NON_Q_CORR[0])
            } else {
                Some(CREDIT_NON_Q_CORR[1])
            }
        } else if LIST_EQUITY.contains(&risk_class) {
            let bucket_str = bucket?;
            let bucket_idx = if bucket_str == "Residual" {
                0
            } else {
                bucket_str.parse::<usize>().ok()?
            };
            EQUITY_CORR.get(bucket_idx).copied()
        } else if LIST_COMMODITY.contains(&risk_class) {
            let bucket_str = bucket?;
            let bucket_idx = if bucket_str == "Residual" {
                0
            } else {
                bucket_str.parse::<usize>().ok()?
            };
            COMMODITY_CORR.get(bucket_idx).copied()
        } else {
            None
        }
    }

    fn gamma(&self, risk_class: &str, bucket1: &str, bucket2: &str) -> Option<f64> {
        if LIST_CREDIT_Q.contains(&risk_class) {
            Some(Self::matrix_lookup_12(&CREDIT_Q_CORR_NON_RES, BUCKET_LIST_12, BUCKET_LIST_12, bucket1, bucket2))
        } else if LIST_CREDIT_NON_Q.contains(&risk_class) {
            Some(CR_GAMMA_DIFF_CCY)
        } else if LIST_EQUITY.contains(&risk_class) {
            Some(Self::matrix_lookup_12(&EQUITY_CORR_NON_RES, BUCKET_LIST_12, BUCKET_LIST_12, bucket1, bucket2))
        } else if LIST_COMMODITY.contains(&risk_class) {
            Some(Self::matrix_lookup_17(&COMMODITY_CORR_NON_RES, BUCKET_LIST_17, BUCKET_LIST_17, bucket1, bucket2))
        } else {
            None
        }
    }

    fn t(&self, risk_class: &str, risk_type: &str, currency: Option<&str>, bucket: Option<&str>) -> Option<f64> {
        if risk_type == "Delta" {
            if risk_class == "Rates" {
                let ccy = currency.unwrap_or("Others");
                let ct = Currency::from_str(ccy).ir_delta_ct();
                Some(ct as f64 * 1_000_000.0)
            } else if LIST_CREDIT_Q.contains(&risk_class) {
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = CREDIT_DELTA_CT_QUALIFYING.get(bucket_idx)?;
                Some(ct * 1_000_000.0)
            } else if LIST_CREDIT_NON_Q.contains(&risk_class) {
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = CREDIT_DELTA_CT_NON_QUALIFYING.get(bucket_idx)?;
                Some(ct * 1_000_000.0)
            } else if LIST_EQUITY.contains(&risk_class) {
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = EQUITY_DELTA_CT.get(bucket_idx)?;
                Some(ct * 1_000_000.0)
            } else if LIST_COMMODITY.contains(&risk_class) {
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = COMMODITY_DELTA_CT.get(bucket_idx)?;
                Some(ct * 1_000_000.0)
            } else if LIST_FX.contains(&risk_class) {
                let ccy = currency?;
                let cat = FxCategory::from_currency(ccy);
                let ct = cat.delta_ct();
                Some(ct as f64 * 1_000_000.0)
            } else {
                None
            }
        } else if risk_type == "Vega" {
            if risk_class == "Rates" {
                let ccy = currency.unwrap_or("Others");
                let ct = Currency::from_str(ccy).ir_vega_ct();
                Some(ct as f64 * 1_000_000.0)
            } else if LIST_CREDIT_Q.contains(&risk_class) {
                let ct = CreditQuality::Qualifying.vega_ct();
                Some(ct as f64 * 1_000_000.0)
            } else if LIST_CREDIT_NON_Q.contains(&risk_class) {
                let ct = CreditQuality::NonQualifying.vega_ct();
                Some(ct as f64 * 1_000_000.0)
            } else if LIST_EQUITY.contains(&risk_class) {
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = EQUITY_VEGA_CT.get(bucket_idx)?;
                Some(*ct as f64 * 1_000_000.0)
            } else if LIST_COMMODITY.contains(&risk_class) {
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = COMMODITY_VEGA_CT.get(bucket_idx)?;
                Some(*ct as f64 * 1_000_000.0)
            } else if LIST_FX.contains(&risk_class) {
                let ccy_pair = currency?;
                if ccy_pair.len() != 6 {
                    return None;
                }
                let ccy1 = &ccy_pair[0..3];
                let ccy2 = &ccy_pair[3..6];
                let cat1 = FxCategory::from_currency(ccy1);
                let cat2 = FxCategory::from_currency(ccy2);
                let ct = cat1.vega_ct(cat2);
                Some(ct as f64 * 1_000_000.0)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn psi(&self, rc1: &str, rc2: &str) -> Option<f64> {
        Some(Self::matrix_lookup_6(&CORR_PARAMS, RISK_CLASSES, RISK_CLASSES, rc1, rc2))
    }
}
