#![allow(dead_code)]

use crate::constants::*;
use crate::wnc::WeightsAndCorr;

/// SIMM version 2.5 parameter set
pub struct V2_5;

impl V2_5 {
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
   === DUMMY DATA TABLES ===
   ===========================

   Replace all of these with the real SIMM values.
*/

pub const REG_VOL_CCY_BUCKET: [&str; 14] = [
    "USD", "EUR", "GBP", "CHF", "AUD", "NZD", "CAD",
    "SEK", "NOK", "DKK", "HKD", "KRW", "SGD", "TWD",
];

pub const LOW_VOL_CCY_BUCKET: [&str; 1] = ["JPY"];

pub static REG_VOL_RW: &[(&str, f64)] = &[
    ("2w", 115.0),
    ("1m", 112.0),
    ("3m", 96.0),
    ("6m", 74.0),
    ("1y", 66.0),
    ("2y", 61.0),
    ("3y", 56.0),
    ("5y", 52.0),
    ("10y", 53.0),
    ("15y", 57.0),
    ("20y", 60.0),
    ("30y", 66.0),
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
    ("3y",  18.0),
    ("5y",  20.0),
    ("10y", 19.0),
    ("15y", 19.0),
    ("20y", 20.0),
    ("30y", 23.0),
];

/// Lookup function for low volatility RW
pub fn low_vol_rw_lookup(tenor: &str) -> f64 {
    LOW_VOL_RW
        .iter()
        .find(|(t, _)| *t == tenor)
        .unwrap_or_else(|| panic!("Unknown tenor: {}", tenor))
        .1
}

pub static HIGH_VOL_RW: &[(&str, f64)] = &[
    ("2w",   119.0),
    ("1m",    93.0),
    ("3m",    80.0),
    ("6m",    82.0),
    ("1y",    90.0),
    ("2y",    92.0),
    ("3y",    95.0),
    ("5y",    95.0),
    ("10y",   94.0),
    ("15y",  108.0),
    ("20y",  105.0),
    ("30y",  101.0),
];

/// Lookup function for high volatility RW
pub fn high_vol_rw_lookup(tenor: &str) -> f64 {
    HIGH_VOL_RW
        .iter()
        .find(|(t, _)| *t == tenor)
        .unwrap_or_else(|| panic!("Unknown tenor: {}", tenor))
        .1
}

pub const INFLATION_RW : f64 = 63.0;
pub const CCY_BASIS_SWAP_SPREAD_RW : f64 = 21.0;
pub const IR_HVR : f64 = 0.44;
pub const IR_VRW : f64 = 0.18;

pub const IR_CORR: [[f64; 12]; 12] = [
    [1.00, 0.74, 0.63, 0.55, 0.45, 0.36, 0.32, 0.28, 0.23, 0.20, 0.18, 0.16],
    [0.74, 1.00, 0.80, 0.69, 0.52, 0.41, 0.35, 0.29, 0.24, 0.18, 0.17, 0.16],
    [0.63, 0.80, 1.00, 0.85, 0.67, 0.53, 0.45, 0.39, 0.32, 0.24, 0.22, 0.22],
    [0.55, 0.69, 0.85, 1.00, 0.83, 0.71, 0.62, 0.54, 0.45, 0.36, 0.35, 0.33],
    [0.45, 0.52, 0.67, 0.83, 1.00, 0.94, 0.86, 0.78, 0.65, 0.58, 0.55, 0.53],
    [0.36, 0.41, 0.53, 0.71, 0.94, 1.00, 0.95, 0.89, 0.78, 0.72, 0.68, 0.67],
    [0.32, 0.35, 0.45, 0.62, 0.86, 0.95, 1.00, 0.96, 0.87, 0.80, 0.77, 0.74],
    [0.28, 0.29, 0.39, 0.54, 0.78, 0.89, 0.96, 1.00, 0.94, 0.89, 0.86, 0.84],
    [0.23, 0.24, 0.32, 0.45, 0.65, 0.78, 0.87, 0.94, 1.00, 0.97, 0.95, 0.94],
    [0.20, 0.18, 0.24, 0.36, 0.58, 0.72, 0.80, 0.89, 0.97, 1.00, 0.98, 0.98],
    [0.18, 0.17, 0.22, 0.35, 0.55, 0.68, 0.77, 0.86, 0.95, 0.98, 1.00, 0.99],
    [0.16, 0.16, 0.22, 0.33, 0.53, 0.67, 0.74, 0.84, 0.94, 0.98, 0.99, 1.00],
];

pub const SUB_CURVES_CORR: f64 = 0.99;
pub const INFLATION_CORR: f64 = 0.37;
pub const CCY_BASIS_SPREAD_CORR: f64 = 0.01;
pub const IR_GAMMA_DIFF_CCY: f64 = 0.24;
pub const CREDIT_Q_RW: [f64; 13] = [
    665.0, // 0 = Residual
    75.0,  // 1
    91.0,  // 2
    78.0,  // 3
    55.0,  // 4
    67.0,  // 5
    47.0,  // 6
    187.0, // 7
    665.0, // 8
    262.0, // 9
    251.0, // 10
    172.0, // 11
    247.0, // 12
];

pub const CREDIT_Q_V_RW: f64 = 0.74;
pub const CREDIT_Q_BASE_CORR_WEIGHT: f64 = 10.0;
pub const CREDIT_Q_CORR: [f64; 4] = [0.93, 0.42, 0.5, 0.24];

pub const CREDIT_Q_CORR_NON_RES: [[f64; 12]; 12] = [
    [1.00, 0.36, 0.38, 0.35, 0.37, 0.33, 0.36, 0.31, 0.32, 0.33, 0.32, 0.30],
    [0.36, 1.00, 0.46, 0.44, 0.45, 0.43, 0.33, 0.36, 0.38, 0.39, 0.40, 0.36],
    [0.38, 0.46, 1.00, 0.49, 0.49, 0.47, 0.34, 0.36, 0.41, 0.42, 0.43, 0.39],
    [0.35, 0.44, 0.49, 1.00, 0.48, 0.48, 0.31, 0.34, 0.38, 0.42, 0.41, 0.37],
    [0.37, 0.45, 0.49, 0.48, 1.00, 0.48, 0.33, 0.35, 0.39, 0.42, 0.43, 0.38],
    [0.33, 0.43, 0.47, 0.48, 0.48, 1.00, 0.29, 0.32, 0.36, 0.39, 0.40, 0.35],
    [0.36, 0.33, 0.34, 0.31, 0.33, 0.29, 1.00, 0.28, 0.32, 0.31, 0.30, 0.28],
    [0.31, 0.36, 0.36, 0.34, 0.35, 0.32, 0.28, 1.00, 0.33, 0.34, 0.33, 0.30],
    [0.32, 0.38, 0.41, 0.38, 0.39, 0.36, 0.32, 0.33, 1.00, 0.38, 0.36, 0.34],
    [0.33, 0.39, 0.42, 0.42, 0.42, 0.39, 0.31, 0.34, 0.38, 1.00, 0.38, 0.36],
    [0.32, 0.40, 0.43, 0.41, 0.43, 0.40, 0.30, 0.33, 0.36, 0.38, 1.00, 0.35],
    [0.30, 0.36, 0.39, 0.37, 0.38, 0.35, 0.28, 0.30, 0.34, 0.36, 0.35, 1.00],
];

pub const CREDIT_NON_Q_RW: [f64;3] = [
    1300.0, // 0
    280.0,
    1300.0
];

pub const CREDIT_NON_Q_VRW: f64 = 0.74;
pub const CREDIT_NON_Q_CORR: [f64; 3] = [0.82,0.27,0.5];
pub const CR_GAMMA_DIFF_CCY: f64 = 0.40;

pub const EQUITY_RW: [f64; 13] = [
    34.0, // 0 = Residual
    26.0, // 1
    28.0, // 2
    34.0, // 3
    28.0, // 4
    23.0, // 5
    25.0, // 6
    29.0, // 7
    27.0, // 8
    32.0, // 9
    32.0, // 10
    18.0, // 11
    18.0, // 12
];

pub const EQUITY_HVR: f64 = 0.58;
pub const EQUITY_VRW: f64 = 0.45;
pub const EQUITY_VRW_BUCKET_12: f64 = 0.96;

pub const EQUITY_CORR: [f64; 13] = [
    0.0,  // 0 = Residual
    0.18, // 1
    0.23, // 2
    0.30, // 3
    0.26, // 4
    0.23, // 5
    0.35, // 6
    0.36, // 7
    0.33, // 8
    0.19, // 9
    0.20, // 10
    0.45, // 11
    0.45, // 12
];

pub const EQUITY_CORR_NON_RES: [[f64; 12]; 12] = [
    [1.00, 0.20, 0.20, 0.20, 0.13, 0.16, 0.16, 0.16, 0.17, 0.12, 0.18, 0.18],
    [0.20, 1.00, 0.25, 0.23, 0.14, 0.17, 0.18, 0.17, 0.19, 0.13, 0.19, 0.19],
    [0.20, 0.25, 1.00, 0.24, 0.13, 0.17, 0.18, 0.16, 0.20, 0.13, 0.18, 0.18],
    [0.20, 0.23, 0.24, 1.00, 0.17, 0.22, 0.22, 0.22, 0.21, 0.16, 0.24, 0.24],
    [0.13, 0.14, 0.13, 0.17, 1.00, 0.27, 0.26, 0.27, 0.15, 0.20, 0.30, 0.30],
    [0.16, 0.17, 0.17, 0.22, 0.27, 1.00, 0.34, 0.33, 0.18, 0.24, 0.38, 0.38],
    [0.16, 0.18, 0.18, 0.22, 0.26, 0.34, 1.00, 0.32, 0.18, 0.24, 0.37, 0.37],
    [0.16, 0.17, 0.16, 0.22, 0.27, 0.33, 0.32, 1.00, 0.18, 0.23, 0.37, 0.37],
    [0.17, 0.19, 0.20, 0.21, 0.15, 0.18, 0.18, 0.18, 1.00, 0.14, 0.20, 0.20],
    [0.12, 0.13, 0.13, 0.16, 0.20, 0.24, 0.24, 0.23, 0.14, 1.00, 0.25, 0.25],
    [0.18, 0.19, 0.18, 0.24, 0.30, 0.38, 0.37, 0.37, 0.20, 0.25, 1.00, 0.45],
    [0.18, 0.19, 0.18, 0.24, 0.30, 0.38, 0.37, 0.37, 0.20, 0.25, 0.45, 1.00],
];

pub const COMMODITY_RW: [f64; 18] = [
    0.0,  // 0 = unused (or could be Residual if you have one)
    27.0, // 1
    29.0, // 2
    33.0, // 3
    25.0, // 4
    35.0, // 5
    24.0, // 6
    40.0, // 7
    53.0, // 8
    44.0, // 9
    58.0, // 10
    20.0, // 11
    21.0, // 12
    13.0, // 13
    16.0, // 14
    13.0, // 15
    58.0, // 16
    17.0, // 17
];

pub const COMMODITY_HVR : f64 = 0.69;
pub const COMMODITY_VRW : f64 = 0.60;

pub const COMMODITY_CORR: [f64; 18] = [
    0.0,  // dummy for zero element
    0.84, // 1
    0.98, // 2
    0.96, // 3
    0.97, // 4
    0.98, // 5
    0.88, // 6
    0.98, // 7
    0.49, // 8
    0.80, // 9
    0.46, // 10
    0.55, // 11
    0.46, // 12
    0.66, // 13
    0.18, // 14
    0.21, // 15
    0.00, // 16
    0.36, // 17
];

pub const COMMODITY_CORR_NON_RES: [[f64; 17]; 17] = [
    [1.00, 0.33, 0.21, 0.27, 0.29, 0.21, 0.48, 0.16, 0.41, 0.23, 0.18, 0.02, 0.21, 0.19, 0.15, 0.00, 0.24],
    [0.33, 1.00, 0.94, 0.94, 0.89, 0.21, 0.19, 0.13, 0.21, 0.21, 0.41, 0.27, 0.31, 0.29, 0.21, 0.00, 0.60],
    [0.21, 0.94, 1.00, 0.91, 0.85, 0.12, 0.20, 0.09, 0.19, 0.20, 0.36, 0.18, 0.22, 0.23, 0.23, 0.00, 0.54],
    [0.27, 0.94, 0.91, 1.00, 0.84, 0.14, 0.24, 0.13, 0.21, 0.19, 0.39, 0.25, 0.23, 0.27, 0.18, 0.00, 0.59],
    [0.29, 0.89, 0.85, 0.84, 1.00, 0.15, 0.17, 0.09, 0.16, 0.21, 0.38, 0.28, 0.28, 0.27, 0.18, 0.00, 0.55],
    [0.21, 0.21, 0.12, 0.14, 0.15, 1.00, 0.33, 0.53, 0.26, 0.09, 0.21, 0.04, 0.11, 0.10, 0.09, 0.00, 0.24],
    [0.48, 0.19, 0.20, 0.24, 0.17, 0.33, 1.00, 0.31, 0.72, 0.24, 0.14,-0.12, 0.19, 0.14, 0.08, 0.00, 0.24],
    [0.16, 0.13, 0.09, 0.13, 0.09, 0.53, 0.31, 1.00, 0.24, 0.04, 0.13,-0.07, 0.04, 0.06, 0.01, 0.00, 0.16],
    [0.41, 0.21, 0.19, 0.21, 0.16, 0.26, 0.72, 0.24, 1.00, 0.21, 0.18,-0.07, 0.12, 0.12, 0.10, 0.00, 0.21],
    [0.23, 0.21, 0.20, 0.19, 0.21, 0.09, 0.24, 0.04, 0.21, 1.00, 0.14, 0.11, 0.11, 0.10, 0.07, 0.00, 0.14],
    [0.18, 0.41, 0.36, 0.39, 0.38, 0.21, 0.14, 0.13, 0.18, 0.14, 1.00, 0.28, 0.30, 0.25, 0.18, 0.00, 0.38],
    [0.02, 0.27, 0.18, 0.25, 0.28, 0.04,-0.12,-0.07,-0.07, 0.11, 0.28, 1.00, 0.18, 0.18, 0.08, 0.00, 0.21],
    [0.21, 0.31, 0.22, 0.23, 0.28, 0.11, 0.19, 0.04, 0.12, 0.11, 0.30, 0.18, 1.00, 0.34, 0.16, 0.00, 0.34],
    [0.19, 0.29, 0.23, 0.27, 0.27, 0.10, 0.14, 0.06, 0.12, 0.10, 0.25, 0.18, 0.34, 1.00, 0.13, 0.00, 0.26],
    [0.15, 0.21, 0.23, 0.18, 0.18, 0.09, 0.08, 0.01, 0.10, 0.07, 0.18, 0.08, 0.16, 0.13, 1.00, 0.00, 0.21],
    [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 1.00, 0.00],
    [0.24, 0.60, 0.54, 0.59, 0.55, 0.24, 0.24, 0.16, 0.21, 0.14, 0.38, 0.21, 0.34, 0.26, 0.21, 0.00, 1.00],
];

pub const HIGH_VOL_CURRENCY_GROUP : &[&str] = &["BRL", "RUB", "TRY", "ZAR"];

#[derive(Debug, Clone, Copy)]
pub enum RiskLevel {
    Regular,
    High,
}

pub const fn fx_rw(outer: RiskLevel, inner: RiskLevel) -> f64 {
    match (outer, inner) {
        (RiskLevel::Regular, RiskLevel::Regular) => 7.4,
        (RiskLevel::Regular, RiskLevel::High) => 13.6,
        (RiskLevel::High, RiskLevel::Regular) => 13.6,
        (RiskLevel::High, RiskLevel::High) => 14.6,
    }
}

pub const FX_HVR : f64 = 0.52;
pub const FX_VRW : f64 = 0.47;

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
    [0.50, 0.27],  // Regular -> [Regular, High]
    [0.27, 0.42],  // High -> [Regular, High]
];

// High Volatility FX Correlations
pub const FX_HIGH_VOL_CORR: [[f64; 2]; 2] = [
    [0.85, 0.54],  // Regular -> [Regular, High]
    [0.54, 0.50],  // High -> [Regular, High]
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
            Currency::USD | Currency::EUR | Currency::GBP => 230,

            // Regular volatility, less well-traded
            Currency::AUD | Currency::CAD | Currency::CHF |
            Currency::DKK | Currency::HKD | Currency::KRW |
            Currency::NOK | Currency::NZD | Currency::SEK |
            Currency::SGD | Currency::TWD => 44,

            // Low volatility
            Currency::JPY => 70,

            // All other currencies
            Currency::Others => 33,
        }
    }

    const fn ir_vega_ct(self) -> u32 {
        match self {
            // Regular volatility, well-traded
            Currency::USD | Currency::EUR | Currency::GBP => 3300,

            // Regular volatility, less well-traded
            Currency::AUD | Currency::CAD | Currency::CHF |
            Currency::DKK | Currency::HKD | Currency::KRW |
            Currency::NOK | Currency::NZD | Currency::SEK |
            Currency::SGD | Currency::TWD => 470,

            // Low volatility
            Currency::JPY => 570,

            // All other currencies
            Currency::Others => 120,
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
            (CreditQuality::Qualifying, 1) => 0.91,  // Sovereigns including central banks
            (CreditQuality::Qualifying, 7) => 0.91,  // Sovereigns including central banks
            (CreditQuality::Qualifying, 2..=6) => 0.19,  // Corporate entities
            (CreditQuality::Qualifying, 8..=12) => 0.19, // Corporate entities
            (CreditQuality::Qualifying, 0) => 0.19,  // Residual

            // Non-Qualifying
            (CreditQuality::NonQualifying, 1) => 9.5,  // Investment Grades (RMBS & CMBS)
            (CreditQuality::NonQualifying, 2) => 0.5,  // High Yield/Non-rated (RMBS & CMBS)
            (CreditQuality::NonQualifying, 0) => 0.5,  // Residual

            // Default case for any other bucket numbers
            _ => 0.19,
        }
    }

    const fn vega_ct(self) -> u32 {
        match self {
            CreditQuality::Qualifying => 260,
            CreditQuality::NonQualifying => 145,
        }
    }
}

pub const CREDIT_DELTA_CT_QUALIFYING: [f64; 13] = [
0.19, // 0: Residual
0.91, // 1: Sovereigns including central banks
0.19, // 2: Corporate entities
0.19, // 3: Corporate entities
0.19, // 4: Corporate entities
0.19, // 5: Corporate entities
0.19, // 6: Corporate entities
0.91, // 7: Sovereigns including central banks
0.19, // 8: Corporate entities
0.19, // 9: Corporate entities
0.19, // 10: Corporate entities
0.19, // 11: Corporate entities
0.19, // 12: Corporate entities
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
    0.6,   // 0: Residual - Not classified
    10.0,  // 1: Emerging Markets - Large Cap
    10.0,  // 2: Emerging Markets - Large Cap
    10.0,  // 3: Emerging Markets - Large Cap
    10.0,  // 4: Emerging Markets - Large Cap
    21.0,  // 5: Developed Markets - Large Cap
    21.0,  // 6: Developed Markets - Large Cap
    21.0,  // 7: Developed Markets - Large Cap
    21.0,  // 8: Developed Markets - Large Cap
    1.4,   // 9: Emerging Markets - Small Cap
    0.6,   // 10: Developed Markets - Small Cap
    2100.0, // 11: Indexes, Funds, ETFs, Volatility Indexes
    2100.0, // 12: Indexes, Funds, ETFs, Volatility Indexes
];

// p.27, 78: Delta Concentration Thresholds for Commodity Risk
pub const COMMODITY_DELTA_CT: [f64; 18] = [
    52.0,   // 0: dummy (using "Other" value as default)
    310.0,  // 1: Coal
    2100.0, // 2: Crude Oil
    1700.0, // 3: Oil Fractions
    1700.0, // 4: Oil Fractions
    1700.0, // 5: Oil Fractions
    3200.0, // 6: Natural Gas
    3200.0, // 7: Natural Gas
    2700.0, // 8: Power
    2700.0, // 9: Power
    52.0,   // 10: Freight, Dry or Wet
    530.0,  // 11: Base metals
    1600.0, // 12: Precious Metals
    100.0,  // 13: Agricultural
    100.0,  // 14: Agricultural
    100.0,  // 15: Agricultural
    52.0,   // 16: Other
    4000.0, // 17: Indices
];

pub const FX_CATEGORY1: [&str; 7] = [
    "USD", "EUR", "JPY", "GBP", "AUD", "CHF", "CAD"
];

// Frequently traded
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
            FxCategory::Category1 => 5100, // Significantly material
            FxCategory::Category2 => 1200, // Frequently traded
            FxCategory::Others => 190,     // All other currencies
        }
    }

    const fn vega_ct(self, other: FxCategory) -> u32 {
        use FxCategory::*;

        match (self, other) {
            (Category1, Category1) => 2800,
            (Category1, Category2) | (Category2, Category1) => 1300,
            (Category1, Others) | (Others, Category1) => 550,
            (Category2, Category2) => 490,
            (Category2, Others) | (Others, Category2) => 310,
            (Others, Others) => 200,
        }
    }

    // Helper to determine category from currency code
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
    40,   // 0: Residual - Not classified
    210,  // 1: Emerging Markets - Large Cap
    210,  // 2: Emerging Markets - Large Cap
    210,  // 3: Emerging Markets - Large Cap
    210,  // 4: Emerging Markets - Large Cap
    1300, // 5: Developed Markets - Large Cap
    1300, // 6: Developed Markets - Large Cap
    1300, // 7: Developed Markets - Large Cap
    1300, // 8: Developed Markets - Large Cap
    40,   // 9: Emerging Markets - Small Cap
    200,  // 10: Developed Markets - Small Cap
    5900, // 11: Indexes, Funds, ETFs, Volatility Indexes
    5900, // 12: Indexes, Funds, ETFs, Volatility Indexes
];

pub const COMMODITY_VEGA_CT: [u32; 18] = [
    65,   // 0: dummy (using "Other" value as default)
    210,  // 1: Coal
    2700, // 2: Crude Oil
    290,  // 3: Oil Fractions
    290,  // 4: Oil Fractions
    290,  // 5: Oil Fractions
    5000, // 6: Natural Gas
    5000, // 7: Natural Gas
    920,  // 8: Power
    920,  // 9: Power
    100,  // 10: Freight, Dry or Wet
    350,  // 11: Base metals
    720,  // 12: Precious Metals
    500,  // 13: Agricultural
    500,  // 14: Agricultural
    500,  // 15: Agricultural
    65,   // 16: Other
    65,   // 17: Indices
];

pub const CORR_PARAMS: [[f64; 6]; 6] = [
    [1.00, 0.29, 0.13, 0.28, 0.46, 0.32],
    [0.29, 1.00, 0.54, 0.71, 0.52, 0.38],
    [0.13, 0.54, 1.00, 0.46, 0.41, 0.12],
    [0.28, 0.71, 0.46, 1.00, 0.49, 0.35],
    [0.46, 0.52, 0.41, 0.49, 1.00, 0.41],
    [0.32, 0.38, 0.12, 0.35, 0.41, 1.00],
];

static RISK_CLASSES: &[&str] = &["Rates", "CreditQ", "CreditNonQ", "Equity", "Commodity", "FX"];

// Bucket labels for gamma lookups
static BUCKET_LIST_12: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"];
static BUCKET_LIST_17: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17"];

fn map_lookup(table: &[(&str, f64)], key: &str) -> f64 {
    table
        .iter()
        .find(|(k, _)| *k == key)
        .unwrap_or_else(|| panic!("Key not found: {}", key))
        .1
}





/* ===========================
   === TRAIT IMPLEMENTATION ==
   =========================== */

impl WeightsAndCorr for V2_5 {
    fn rw(&self, risk_class: &str, bucket: &str) -> Option<f64> {
        // Parse bucket string to usize
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
            // For rates: use tenor correlation matrix
            Some(Self::matrix_lookup_12(&IR_CORR, SIMM_TENORS, SIMM_TENORS, index1, index2))
        } else if LIST_CREDIT_Q.contains(&risk_class) {
            // For CreditQ
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
            // For CreditNonQ
            if index1 == "Res" || index2 == "Res" {
                Some(CREDIT_NON_Q_CORR[2])
            } else if index1 == index2 {
                Some(CREDIT_NON_Q_CORR[0])
            } else {
                Some(CREDIT_NON_Q_CORR[1])
            }
        } else if LIST_EQUITY.contains(&risk_class) {
            // For Equity: correlation by bucket
            let bucket_str = bucket?;
            let bucket_idx = if bucket_str == "Residual" {
                0
            } else {
                bucket_str.parse::<usize>().ok()?
            };
            EQUITY_CORR.get(bucket_idx).copied()
        } else if LIST_COMMODITY.contains(&risk_class) {
            // For Commodity: correlation by bucket
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
            // CreditQ: use 12x12 correlation matrix
            Some(Self::matrix_lookup_12(&CREDIT_Q_CORR_NON_RES, BUCKET_LIST_12, BUCKET_LIST_12, bucket1, bucket2))
        } else if LIST_CREDIT_NON_Q.contains(&risk_class) {
            // CreditNonQ: fixed gamma value for different currencies
            Some(CR_GAMMA_DIFF_CCY)
        } else if LIST_EQUITY.contains(&risk_class) {
            // Equity: use 12x12 correlation matrix
            Some(Self::matrix_lookup_12(&EQUITY_CORR_NON_RES, BUCKET_LIST_12, BUCKET_LIST_12, bucket1, bucket2))
        } else if LIST_COMMODITY.contains(&risk_class) {
            // Commodity: use 17x17 correlation matrix
            Some(Self::matrix_lookup_17(&COMMODITY_CORR_NON_RES, BUCKET_LIST_17, BUCKET_LIST_17, bucket1, bucket2))
        } else {
            None
        }
    }

    fn t(&self, risk_class: &str, risk_type: &str, currency: Option<&str>, bucket: Option<&str>) -> Option<f64> {
        if risk_type == "Delta" {
            if risk_class == "Rates" {
                // IR Delta CT
                let ccy = currency.unwrap_or("Others");
                let ct = Currency::from_str(ccy).ir_delta_ct();
                Some(ct as f64 * 1_000_000.0)
            } else if LIST_CREDIT_Q.contains(&risk_class) {
                // Credit Qualifying Delta CT
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = CREDIT_DELTA_CT_QUALIFYING.get(bucket_idx)?;
                Some(ct * 1_000_000.0)
            } else if LIST_CREDIT_NON_Q.contains(&risk_class) {
                // Credit Non-Qualifying Delta CT
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = CREDIT_DELTA_CT_NON_QUALIFYING.get(bucket_idx)?;
                Some(ct * 1_000_000.0)
            } else if LIST_EQUITY.contains(&risk_class) {
                // Equity Delta CT
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = EQUITY_DELTA_CT.get(bucket_idx)?;
                Some(ct * 1_000_000.0)
            } else if LIST_COMMODITY.contains(&risk_class) {
                // Commodity Delta CT
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = COMMODITY_DELTA_CT.get(bucket_idx)?;
                Some(ct * 1_000_000.0)
            } else if LIST_FX.contains(&risk_class) {
                // FX Delta CT
                let ccy = currency?;
                let cat = FxCategory::from_currency(ccy);
                let ct = cat.delta_ct();
                Some(ct as f64 * 1_000_000.0)
            } else {
                None
            }
        } else if risk_type == "Vega" {
            if risk_class == "Rates" {
                // IR Vega CT
                let ccy = currency.unwrap_or("Others");
                let ct = Currency::from_str(ccy).ir_vega_ct();
                Some(ct as f64 * 1_000_000.0)
            } else if LIST_CREDIT_Q.contains(&risk_class) {
                // Credit Qualifying Vega CT
                let ct = CreditQuality::Qualifying.vega_ct();
                Some(ct as f64 * 1_000_000.0)
            } else if LIST_CREDIT_NON_Q.contains(&risk_class) {
                // Credit Non-Qualifying Vega CT
                let ct = CreditQuality::NonQualifying.vega_ct();
                Some(ct as f64 * 1_000_000.0)
            } else if LIST_EQUITY.contains(&risk_class) {
                // Equity Vega CT
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = EQUITY_VEGA_CT.get(bucket_idx)?;
                Some(*ct as f64 * 1_000_000.0)
            } else if LIST_COMMODITY.contains(&risk_class) {
                // Commodity Vega CT
                let bucket_str = bucket?;
                let bucket_idx = if bucket_str == "Residual" {
                    0
                } else {
                    bucket_str.parse::<usize>().ok()?
                };
                let ct = COMMODITY_VEGA_CT.get(bucket_idx)?;
                Some(*ct as f64 * 1_000_000.0)
            } else if LIST_FX.contains(&risk_class) {
                // FX Vega CT - currency pair (6 characters: CCY1CCY2)
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
