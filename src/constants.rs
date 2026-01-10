
use std::str::FromStr;

/// Constants and configuration values for ISDA SIMM calculations

#[allow(dead_code)]
pub const LIST_VEGA: &[&str] = &[
    "Risk_IRVol",
    "Risk_InflationVol",
    "Risk_CreditVol",
    "Risk_CreditVolNonQ",
    "Risk_EquityVol",
    "Risk_CommodityVol",
    "Risk_FXVol",
];

#[allow(dead_code)]
pub const FULL_BUCKET_LIST: &[&str] = &[
    "1","2","3","4","5","6","7","8","9","10","11","12","13","14","15","16","Residual"
];

pub const SIMM_TENOR_LIST: &[&str] = &[
    "2w","1m","3m","6m","1y","2y","3y","5y","10y","15y","20y","30y"
];

pub const LIST_RATES: &[&str] = &[
    "Risk_IRCurve",
    "Risk_Inflation",
    "Risk_XCcyBasis",
    "Risk_IRVol",
    "Risk_InflationVol",
];

pub const LIST_FX: &[&str] = &[
    "Risk_FX",
    "Risk_FXVol",
];

pub const LIST_CREDIT_Q: &[&str] = &[
    "Risk_CreditQ",
    "Risk_CreditVol",
    "Risk_BaseCorr",
];

pub const LIST_CREDIT_NON_Q: &[&str] = &[
    "Risk_CreditNonQ",
    "Risk_CreditVolNonQ",
];

pub const LIST_EQUITY: &[&str] = &[
    "Risk_Equity",
    "Risk_EquityVol",
];

pub const LIST_COMMODITY: &[&str] = &[
    "Risk_Commodity",
    "Risk_CommodityVol",
];

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RiskClass {
    Rates,
    FX,
    CreditQ,
    CreditNonQ,
    Equity,
    Commodity,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RiskType {
    Delta,
    Vega,
    Curvature,
    BaseCorr,
}

impl FromStr for RiskClass {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Rates" => Ok(RiskClass::Rates),
            "FX" => Ok(RiskClass::FX),
            "CreditQ" => Ok(RiskClass::CreditQ),
            "CreditNonQ" => Ok(RiskClass::CreditNonQ),
            "Equity" => Ok(RiskClass::Equity),
            "Commodity" => Ok(RiskClass::Commodity),
            _ => Err(()),
        }
    }
}

impl FromStr for RiskType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Delta" => Ok(RiskType::Delta),
            "Vega" => Ok(RiskType::Vega),
            "Curvature" => Ok(RiskType::Curvature),
            "BaseCorr" => Ok(RiskType::BaseCorr),
            _ => Err(()),
        }
    }
}

impl RiskClass {
    #[allow(dead_code)]
    pub const fn as_str(self) -> &'static str {
        match self {
            RiskClass::Rates => "Rates",
            RiskClass::FX => "FX",
            RiskClass::CreditQ => "CreditQ",
            RiskClass::CreditNonQ => "CreditNonQ",
            RiskClass::Equity => "Equity",
            RiskClass::Commodity => "Commodity",
        }
    }
}

impl RiskType {
    #[allow(dead_code)]
    pub const fn as_str(self) -> &'static str {
        match self {
            RiskType::Delta => "Delta",
            RiskType::Vega => "Vega",
            RiskType::Curvature => "Curvature",
            RiskType::BaseCorr => "BaseCorr",
        }
    }
}

impl RiskClass {
    /// Which risk types are valid for this risk class (SIMM spec)
    #[allow(dead_code)]
    pub const fn allowed_risk_types(self) -> &'static [RiskType] {
        match self {
            RiskClass::Rates
            | RiskClass::FX
            | RiskClass::CreditQ
            | RiskClass::CreditNonQ
            | RiskClass::Equity
            | RiskClass::Commodity => &[
                RiskType::Delta,
                RiskType::Vega,
                RiskType::Curvature,
                RiskType::BaseCorr,
            ],
        }
    }

    /// Fast check (no allocation, no hashmap)
    #[allow(dead_code)]
    pub const fn supports(self, rt: RiskType) -> bool {
        match (self, rt) {
            (
                RiskClass::Rates
                | RiskClass::FX
                | RiskClass::CreditQ
                | RiskClass::CreditNonQ
                | RiskClass::Equity
                | RiskClass::Commodity,
                RiskType::Delta | RiskType::Vega | RiskType::Curvature | RiskType::BaseCorr,
            ) => true,
        }
    }
}

impl RiskClass {
    #[allow(dead_code)]
    pub const ALL: [RiskClass; 6] = [
        RiskClass::Rates,
        RiskClass::FX,
        RiskClass::CreditQ,
        RiskClass::CreditNonQ,
        RiskClass::Equity,
        RiskClass::Commodity,
    ];
}

impl RiskType {
    #[allow(dead_code)]
    pub const ALL: [RiskType; 4] = [
        RiskType::Delta,
        RiskType::Vega,
        RiskType::Curvature,
        RiskType::BaseCorr,
    ];
}

/// Dict mapping each risk class to its allowed risk measures
pub fn margin_by_risk_class() -> std::collections::HashMap<&'static str, Vec<&'static str>> {
    let mut map = std::collections::HashMap::new();
    map.insert("Rates", vec!["Delta", "Vega", "Curvature", "BaseCorr"]);
    map.insert("FX", vec!["Delta", "Vega", "Curvature", "BaseCorr"]);
    map.insert("CreditQ", vec!["Delta", "Vega", "Curvature", "BaseCorr"]);
    map.insert("CreditNonQ", vec!["Delta", "Vega", "Curvature", "BaseCorr"]);
    map.insert("Equity", vec!["Delta", "Vega", "Curvature", "BaseCorr"]);
    map.insert("Commodity", vec!["Delta", "Vega", "Curvature", "BaseCorr"]);
    map
}

/* Usage
fn main() {
    let rc: RiskClass = "Rates".parse().unwrap();
    let rt: RiskType = "Vega".parse().unwrap();

    assert!(rc.supports(rt));

    for rc in RiskClass::ALL {
        println!("{} supports:", rc.as_str());
        for rt in rc.allowed_risk_types() {
            println!("  - {}", rt.as_str());
        }
    }
}
 */