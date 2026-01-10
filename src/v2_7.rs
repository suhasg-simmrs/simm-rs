#![allow(dead_code)]

use crate::constants::*;
use crate::wnc::WeightsAndCorr;

/// SIMM version 2.7 parameter set
pub struct V2_7;

impl V2_7 {
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
   === SIMM v2.7 PARAMETERS ==
   ===========================

   Based on https://www.isda.org/a/b4ugE/ISDA-SIMM_v2.7_PUBLIC.pdf
*/

pub const REG_VOL_CCY_BUCKET: [&str; 14] = [
    "USD", "EUR", "GBP", "CHF", "AUD", "NZD", "CAD",
    "SEK", "NOK", "DKK", "HKD", "KRW", "SGD", "TWD",
];

pub const LOW_VOL_CCY_BUCKET: [&str; 1] = ["JPY"];

pub static REG_VOL_RW: &[(&str, f64)] = &[
    ("2w", 109.0),
    ("1m", 106.0),
    ("3m", 91.0),
    ("6m", 69.0),
    ("1y", 68.0),
    ("2y", 68.0),
    ("3y", 66.0),
    ("5y", 61.0),
    ("10y", 59.0),
    ("15y", 56.0),
    ("20y", 57.0),
    ("30y", 65.0),
];

pub fn reg_vol_rw_lookup(tenor: &str) -> f64 {
    REG_VOL_RW.iter()
        .find(|(t, _)| *t == tenor)
        .unwrap_or_else(|| panic!("Unknown tenor: {}", tenor))
        .1
}

pub static LOW_VOL_RW: &[(&str, f64)] = &[
    ("2w",  15.0),
    ("1m",  21.0),
    ("3m",  10.0),
    ("6m",  10.0),
    ("1y",  11.0),
    ("2y",  15.0),
    ("3y",  18.0),
    ("5y",  23.0),
    ("10y", 25.0),
    ("15y", 23.0),
    ("20y", 23.0),
    ("30y", 25.0),
];

pub fn low_vol_rw_lookup(tenor: &str) -> f64 {
    LOW_VOL_RW
        .iter()
        .find(|(t, _)| *t == tenor)
        .unwrap_or_else(|| panic!("Unknown tenor: {}", tenor))
        .1
}

pub static HIGH_VOL_RW: &[(&str, f64)] = &[
    ("2w",   171.0),
    ("1m",   102.0),
    ("3m",    94.0),
    ("6m",    96.0),
    ("1y",   105.0),
    ("2y",    96.0),
    ("3y",    99.0),
    ("5y",    93.0),
    ("10y",   99.0),
    ("15y",  100.0),
    ("20y",  101.0),
    ("30y",   96.0),
];

pub fn high_vol_rw_lookup(tenor: &str) -> f64 {
    HIGH_VOL_RW
        .iter()
        .find(|(t, _)| *t == tenor)
        .unwrap_or_else(|| panic!("Unknown tenor: {}", tenor))
        .1
}

pub const INFLATION_RW : f64 = 52.0;
pub const CCY_BASIS_SWAP_SPREAD_RW : f64 = 21.0;
pub const IR_HVR : f64 = 0.69;
pub const IR_VRW : f64 = 0.20;

pub const IR_CORR: [[f64; 12]; 12] = [
    [1.00, 0.75, 0.67, 0.57, 0.43, 0.33, 0.28, 0.24, 0.19, 0.17, 0.16, 0.15],
    [0.75, 1.00, 0.85, 0.72, 0.52, 0.38, 0.30, 0.24, 0.19, 0.14, 0.16, 0.15],
    [0.67, 0.85, 1.00, 0.88, 0.67, 0.52, 0.44, 0.37, 0.30, 0.23, 0.21, 0.21],
    [0.57, 0.72, 0.88, 1.00, 0.86, 0.73, 0.64, 0.56, 0.47, 0.41, 0.38, 0.37],
    [0.43, 0.52, 0.67, 0.86, 1.00, 0.94, 0.86, 0.78, 0.67, 0.61, 0.57, 0.56],
    [0.33, 0.38, 0.52, 0.73, 0.94, 1.00, 0.96, 0.91, 0.80, 0.74, 0.70, 0.69],
    [0.28, 0.30, 0.44, 0.64, 0.86, 0.96, 1.00, 0.97, 0.87, 0.81, 0.77, 0.76],
    [0.24, 0.24, 0.37, 0.56, 0.78, 0.91, 0.97, 1.00, 0.94, 0.90, 0.86, 0.85],
    [0.19, 0.19, 0.30, 0.47, 0.67, 0.80, 0.87, 0.94, 1.00, 0.97, 0.94, 0.94],
    [0.17, 0.14, 0.23, 0.41, 0.61, 0.74, 0.81, 0.90, 0.97, 1.00, 0.97, 0.97],
    [0.16, 0.12, 0.21, 0.38, 0.57, 0.70, 0.77, 0.86, 0.94, 0.97, 1.00, 0.99],
    [0.15, 0.12, 0.21, 0.37, 0.56, 0.69, 0.76, 0.85, 0.94, 0.97, 0.99, 1.00],
];

pub const SUB_CURVES_CORR: f64 = 0.99;
pub const INFLATION_CORR: f64 = 0.26;
pub const CCY_BASIS_SPREAD_CORR: f64 = -0.05;
pub const IR_GAMMA_DIFF_CCY: f64 = 0.30;

pub const CREDIT_Q_RW: [f64; 13] = [
    363.0, // 0 = Residual
    69.0,  // 1
    75.0,  // 2
    69.0,  // 3
    47.0,  // 4
    58.0,  // 5
    48.0,  // 6
    153.0, // 7
    363.0, // 8
    156.0, // 9
    188.0, // 10
    299.0, // 11
    119.0, // 12
];

pub const CREDIT_Q_V_RW: f64 = 0.29;
pub const CREDIT_Q_BASE_CORR_WEIGHT: f64 = 9.9;
pub const CREDIT_Q_CORR: [f64; 4] = [0.94, 0.47, 0.5, 0.31];

pub const CREDIT_Q_CORR_NON_RES: [[f64; 12]; 12] = [
    [1.00, 0.41, 0.39, 0.35, 0.38, 0.36, 0.43, 0.29, 0.36, 0.36, 0.36, 0.37],
    [0.41, 1.00, 0.48, 0.45, 0.48, 0.45, 0.40, 0.35, 0.43, 0.43, 0.42, 0.44],
    [0.39, 0.48, 1.00, 0.49, 0.50, 0.50, 0.41, 0.32, 0.46, 0.45, 0.43, 0.48],
    [0.35, 0.45, 0.49, 1.00, 0.50, 0.49, 0.38, 0.30, 0.42, 0.44, 0.41, 0.47],
    [0.38, 0.48, 0.50, 0.50, 1.00, 0.51, 0.40, 0.31, 0.44, 0.45, 0.43, 0.49],
    [0.36, 0.45, 0.50, 0.49, 0.51, 1.00, 0.39, 0.29, 0.42, 0.43, 0.41, 0.49],
    [0.43, 0.40, 0.41, 0.38, 0.40, 0.39, 1.00, 0.28, 0.37, 0.38, 0.37, 0.39],
    [0.29, 0.35, 0.32, 0.30, 0.31, 0.29, 0.28, 1.00, 0.30, 0.30, 0.29, 0.31],
    [0.36, 0.43, 0.46, 0.42, 0.44, 0.42, 0.37, 0.30, 1.00, 0.42, 0.40, 0.44],
    [0.36, 0.43, 0.45, 0.44, 0.45, 0.43, 0.38, 0.30, 0.42, 1.00, 0.40, 0.45],
    [0.36, 0.42, 0.43, 0.41, 0.43, 0.41, 0.37, 0.29, 0.40, 0.40, 1.00, 0.42],
    [0.37, 0.44, 0.48, 0.47, 0.49, 0.49, 0.39, 0.31, 0.44, 0.45, 0.42, 1.00],
];

pub const CREDIT_NON_Q_RW: [f64;3] = [
    2900.0, // 0 = Residual
    280.0,  // 1
    2900.0  // 2
];

pub const CREDIT_NON_Q_VRW: f64 = 0.29;
pub const CREDIT_NON_Q_CORR: [f64; 3] = [0.85, 0.29, 0.5];
pub const CR_GAMMA_DIFF_CCY: f64 = 0.51;

pub const EQUITY_RW: [f64; 13] = [
    39.0, // 0 = Residual
    27.0, // 1
    30.0, // 2
    31.0, // 3
    27.0, // 4
    23.0, // 5
    24.0, // 6
    26.0, // 7
    27.0, // 8
    33.0, // 9
    39.0, // 10
    15.0, // 11
    15.0, // 12
];

pub const EQUITY_HVR: f64 = 0.62;
pub const EQUITY_VRW: f64 = 0.25;
pub const EQUITY_VRW_BUCKET_12: f64 = 0.56;

pub const EQUITY_CORR: [f64; 13] = [
    0.0,  // 0 = Residual
    0.14, // 1
    0.16, // 2
    0.23, // 3
    0.21, // 4
    0.23, // 5
    0.32, // 6
    0.32, // 7
    0.35, // 8
    0.21, // 9
    0.22, // 10
    0.40, // 11
    0.40, // 12
];

pub const EQUITY_CORR_NON_RES: [[f64; 12]; 12] = [
    [1.00, 0.14, 0.15, 0.16, 0.13, 0.15, 0.14, 0.15, 0.14, 0.12, 0.17, 0.17],
    [0.14, 1.00, 0.18, 0.18, 0.14, 0.17, 0.17, 0.18, 0.16, 0.14, 0.19, 0.19],
    [0.15, 0.18, 1.00, 0.19, 0.14, 0.18, 0.21, 0.19, 0.18, 0.14, 0.21, 0.21],
    [0.16, 0.18, 0.19, 1.00, 0.17, 0.22, 0.21, 0.23, 0.18, 0.17, 0.24, 0.24],
    [0.13, 0.14, 0.14, 0.17, 1.00, 0.25, 0.23, 0.26, 0.13, 0.20, 0.28, 0.28],
    [0.15, 0.17, 0.18, 0.22, 0.25, 1.00, 0.29, 0.33, 0.16, 0.26, 0.34, 0.34],
    [0.14, 0.17, 0.21, 0.21, 0.23, 0.29, 1.00, 0.30, 0.15, 0.24, 0.33, 0.33],
    [0.15, 0.18, 0.19, 0.23, 0.26, 0.33, 0.30, 1.00, 0.16, 0.26, 0.37, 0.37],
    [0.14, 0.16, 0.18, 0.18, 0.13, 0.16, 0.15, 0.16, 1.00, 0.12, 0.19, 0.19],
    [0.12, 0.14, 0.14, 0.17, 0.20, 0.26, 0.24, 0.26, 0.12, 1.00, 0.26, 0.26],
    [0.17, 0.19, 0.21, 0.24, 0.28, 0.34, 0.33, 0.37, 0.19, 0.26, 1.00, 0.40],
    [0.17, 0.19, 0.21, 0.24, 0.28, 0.34, 0.33, 0.37, 0.19, 0.26, 0.40, 1.00],
];

pub const COMMODITY_RW: [f64; 18] = [
    0.0,  // 0 = unused (or could be Residual if you have one)
    48.0, // 1
    21.0, // 2
    23.0, // 3
    20.0, // 4
    24.0, // 5
    33.0, // 6
    61.0, // 7
    45.0, // 8
    65.0, // 9
    45.0, // 10
    21.0, // 11
    19.0, // 12
    16.0, // 13
    16.0, // 14
    11.0, // 15
    65.0, // 16
    16.0, // 17
];

pub const COMMODITY_HVR : f64 = 0.85;
pub const COMMODITY_VRW : f64 = 0.34;

pub const COMMODITY_CORR: [f64; 18] = [
    0.0,  // dummy for zero element
    0.84, // 1
    0.98, // 2
    0.98, // 3
    0.98, // 4
    0.98, // 5
    0.93, // 6
    0.93, // 7
    0.51, // 8
    0.59, // 9
    0.44, // 10
    0.58, // 11
    0.60, // 12
    0.60, // 13
    0.21, // 14
    0.17, // 15
    0.00, // 16
    0.43, // 17
];

pub const COMMODITY_CORR_NON_RES: [[f64; 17]; 17] = [
    [1.00, 0.23, 0.19, 0.28, 0.24, 0.32,  0.62,  0.29,  0.50, 0.15, 0.13,  0.08, 0.19, 0.12,  0.04, 0.00, 0.22],
    [0.23, 1.00, 0.94, 0.92, 0.89, 0.36,  0.15,  0.23,  0.15, 0.20, 0.42,  0.31, 0.38, 0.28,  0.16, 0.00, 0.67],
    [0.19, 0.94, 1.00, 0.91, 0.86, 0.32,  0.11,  0.19,  0.12, 0.22, 0.41,  0.31, 0.37, 0.25,  0.15, 0.00, 0.64],
    [0.28, 0.92, 0.91, 1.00, 0.81, 0.40,  0.17,  0.26,  0.18, 0.20, 0.41,  0.26, 0.34, 0.25,  0.14, 0.00, 0.64],
    [0.24, 0.89, 0.86, 0.81, 1.00, 0.29,  0.17,  0.26,  0.23, 0.26, 0.42,  0.34, 0.23, 0.32,  0.14, 0.00, 0.62],
    [0.32, 0.36, 0.32, 0.40, 0.29, 1.00,  0.30,  0.66,  0.23, 0.07, 0.12,  0.07, 0.23, 0.09,  0.11, 0.00, 0.39],
    [0.62, 0.15, 0.11, 0.17, 0.17, 0.30,  1.00,  0.19,  0.78, 0.12, 0.12,  0.02, 0.11, 0.07,  0.00, 0.00, 0.21],
    [0.29, 0.23, 0.19, 0.26, 0.21, 0.66,  0.19,  1.00,  0.19, 0.04, 0.10, -0.01, 0.11, 0.04,  0.03, 0.00, 0.21],
    [0.50, 0.15, 0.12, 0.18, 0.23, 0.23,  0.78,  0.19,  1.00, 0.07, 0.06, -0.08, 0.13, 0.12,  0.10, 0.00, 0.18],
    [0.15, 0.20, 0.22, 0.20, 0.26, 0.07,  0.12,  0.04,  0.07, 1.00, 0.19,  0.10, 0.13, 0.10,  0.10, 0.00, 0.12],
    [0.13, 0.42, 0.41, 0.41, 0.42, 0.21,  0.12,  0.10,  0.06, 0.19, 1.00,  0.39, 0.31, 0.24,  0.14, 0.00, 0.39],
    [0.08, 0.31, 0.31, 0.26, 0.34, 0.07,  0.02, -0.01, -0.08, 0.10, 0.39,  1.00, 0.22, 0.20,  0.12, 0.00, 0.28],
    [0.19, 0.38, 0.37, 0.34, 0.23, 0.19,  0.11,  0.11,  0.13, 0.13, 0.31,  0.22, 1.00, 0.28,  0.19, 0.00, 0.41],
    [0.12, 0.28, 0.25, 0.27, 0.32, 0.09,  0.07,  0.04,  0.12, 0.10, 0.24,  0.20, 0.28, 1.00,  0.09, 0.00, 0.22],
    [0.04, 0.16, 0.15, 0.14, 0.14, 0.11,  0.00,  0.03,  0.10, 0.10, 0.14,  0.12, 0.19, 0.09,  1.00, 0.00, 0.21],
    [0.00, 0.00, 0.00, 0.00, 0.00, 0.00,  0.00,  0.00,  0.00, 0.00, 0.00,  0.00, 0.00, 0.00,  0.00, 1.00, 0.00],
    [0.22, 0.67, 0.64, 0.64, 0.62, 0.39,  0.21,  0.21,  0.18, 0.12, 0.39,  0.28, 0.41, 0.22,  0.21, 0.00, 1.00],
];

pub const HIGH_VOL_CURRENCY_GROUP : &[&str] = &["ARS", "RUB", "TRY"];

#[derive(Debug, Clone, Copy)]
pub enum RiskLevel {
    Regular,
    High,
}

pub const fn fx_rw(outer: RiskLevel, inner: RiskLevel) -> f64 {
    match (outer, inner) {
        (RiskLevel::Regular, RiskLevel::Regular) => 7.3,
        (RiskLevel::Regular, RiskLevel::High) => 21.4,
        (RiskLevel::High, RiskLevel::Regular) => 21.4,
        (RiskLevel::High, RiskLevel::High) => 35.9,
    }
}

pub const FX_HVR : f64 = 0.62;
pub const FX_VRW : f64 = 0.35;

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
    [0.50, 0.17],  // Regular -> [Regular, High]
    [0.17, -0.41], // High -> [Regular, High]
];

// High Volatility FX Correlations
pub const FX_HIGH_VOL_CORR: [[f64; 2]; 2] = [
    [0.94, 0.84],  // Regular -> [Regular, High]
    [0.84, 0.50],  // High -> [Regular, High]
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
            Currency::USD | Currency::EUR | Currency::GBP => 340,

            // Regular volatility, less well-traded
            Currency::AUD | Currency::CAD | Currency::CHF |
            Currency::DKK | Currency::HKD | Currency::KRW |
            Currency::NOK | Currency::NZD | Currency::SEK |
            Currency::SGD | Currency::TWD => 61,

            // Low volatility
            Currency::JPY => 150,

            // All other currencies
            Currency::Others => 29,
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
            Currency::SGD | Currency::TWD => 550,

            // Low volatility
            Currency::JPY => 890,

            // All other currencies
            Currency::Others => 76,
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
            (CreditQuality::Qualifying, 1) => 0.98,  // Sovereigns including central banks
            (CreditQuality::Qualifying, 7) => 0.98,  // Sovereigns including central banks
            (CreditQuality::Qualifying, 2..=6) => 0.18,  // Corporate entities
            (CreditQuality::Qualifying, 8..=12) => 0.18, // Corporate entities
            (CreditQuality::Qualifying, 0) => 0.18,  // Residual

            // Non-Qualifying
            (CreditQuality::NonQualifying, 1) => 3.3,  // Investment Grades (RMBS & CMBS)
            (CreditQuality::NonQualifying, 2) => 0.18,  // High Yield/Non-rated (RMBS & CMBS)
            (CreditQuality::NonQualifying, 0) => 0.18,  // Residual

            // Default case for any other bucket numbers
            _ => 0.18,
        }
    }

    const fn vega_ct(self) -> u32 {
        match self {
            CreditQuality::Qualifying => 290,
            CreditQuality::NonQualifying => 21,
        }
    }
}

pub const CREDIT_DELTA_CT_QUALIFYING: [f64; 13] = [
    0.18, // 0: Residual
    0.98, // 1: Sovereigns including central banks
    0.18, // 2: Corporate entities
    0.18, // 3: Corporate entities
    0.18, // 4: Corporate entities
    0.18, // 5: Corporate entities
    0.18, // 6: Corporate entities
    0.98, // 7: Sovereigns including central banks
    0.18, // 8: Corporate entities
    0.18, // 9: Corporate entities
    0.18, // 10: Corporate entities
    0.18, // 11: Corporate entities
    0.18, // 12: Corporate entities
];

pub const CREDIT_DELTA_CT_NON_QUALIFYING: [f64; 3] = [
    0.18, // 0: Residual
    3.3,  // 1: Investment Grades (RMBS & CMBS)
    0.18, // 2: High Yield/Non-rated (RMBS & CMBS)
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
    0.3,   // 0: Residual - Not classified
    2.5,   // 1: Emerging Markets - Large Cap
    2.5,   // 2: Emerging Markets - Large Cap
    2.5,   // 3: Emerging Markets - Large Cap
    2.5,   // 4: Emerging Markets - Large Cap
    10.0,  // 5: Developed Markets - Large Cap
    10.0,  // 6: Developed Markets - Large Cap
    10.0,  // 7: Developed Markets - Large Cap
    10.0,  // 8: Developed Markets - Large Cap
    0.61,  // 9: Emerging Markets - Small Cap
    0.3,   // 10: Developed Markets - Small Cap
    710.0, // 11: Indexes, Funds, ETFs, Volatility Indexes
    710.0, // 12: Indexes, Funds, ETFs, Volatility Indexes
];

pub const COMMODITY_DELTA_CT: [f64; 18] = [
    52.0,   // 0: dummy (using "Other" value as default)
    310.0,  // 1: Coal
    2500.0, // 2: Crude Oil
    1700.0, // 3: Oil Fractions
    1700.0, // 4: Oil Fractions
    1700.0, // 5: Oil Fractions
    2400.0, // 6: Natural Gas
    2400.0, // 7: Natural Gas
    1800.0, // 8: Power
    1800.0, // 9: Power
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
            FxCategory::Category1 => 2000, // Significantly material
            FxCategory::Category2 => 630,  // Frequently traded
            FxCategory::Others => 120,     // All other currencies
        }
    }

    const fn vega_ct(self, other: FxCategory) -> u32 {
        use FxCategory::*;

        match (self, other) {
            (Category1, Category1) => 3000,
            (Category1, Category2) | (Category2, Category1) => 1500,
            (Category1, Others) | (Others, Category1) => 670,
            (Category2, Category2) => 600,
            (Category2, Others) | (Others, Category2) => 390,
            (Others, Others) => 240,
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
    74,   // 0: Residual - Not classified
    300,  // 1: Emerging Markets - Large Cap
    300,  // 2: Emerging Markets - Large Cap
    300,  // 3: Emerging Markets - Large Cap
    300,  // 4: Emerging Markets - Large Cap
    1500, // 5: Developed Markets - Large Cap
    1500, // 6: Developed Markets - Large Cap
    1500, // 7: Developed Markets - Large Cap
    1500, // 8: Developed Markets - Large Cap
    74,   // 9: Emerging Markets - Small Cap
    280,  // 10: Developed Markets - Small Cap
    4300, // 11: Indexes, Funds, ETFs, Volatility Indexes
    4300, // 12: Indexes, Funds, ETFs, Volatility Indexes
];

pub const COMMODITY_VEGA_CT: [u32; 18] = [
    59,   // 0: dummy (using "Other" value as default)
    450,  // 1: Coal
    2300, // 2: Crude Oil
    240,  // 3: Oil Fractions
    240,  // 4: Oil Fractions
    240,  // 5: Oil Fractions
    6400, // 6: Natural Gas
    6400, // 7: Natural Gas
    1300, // 8: Power
    1300, // 9: Power
    94,   // 10: Freight, Dry or Wet
    490,  // 11: Base metals
    810,  // 12: Precious Metals
    730,  // 13: Agricultural
    730,  // 14: Agricultural
    730,  // 15: Agricultural
    59,   // 16: Other
    59,   // 17: Indices
];

pub const CORR_PARAMS: [[f64; 6]; 6] = [
    [1.00, 0.15, 0.09, 0.08, 0.33, 0.09],
    [0.15, 1.00, 0.52, 0.67, 0.23, 0.20],
    [0.09, 0.52, 1.00, 0.36, 0.16, 0.12],
    [0.08, 0.67, 0.36, 1.00, 0.34, 0.24],
    [0.33, 0.23, 0.16, 0.34, 1.00, 0.28],
    [0.09, 0.20, 0.12, 0.24, 0.28, 1.00],
];

static RISK_CLASSES: &[&str] = &["Rates", "CreditQ", "CreditNonQ", "Equity", "Commodity", "FX"];

static BUCKET_LIST_12: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"];
static BUCKET_LIST_17: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17"];

/* ===========================
   === TRAIT IMPLEMENTATION ==
   =========================== */

impl WeightsAndCorr for V2_7 {
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
