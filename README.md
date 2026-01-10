# SIMM-RS: Standard Initial Margin Model Calculator

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](https://github.com/yourusername/simm-rs)

A high-performance Rust implementation of the ISDA SIMM (Standard Initial Margin Model) for calculating initial margin requirements on non-cleared derivatives.

## Table of Contents

- [What is SIMM?](#what-is-simm)
- [Features](#features)
- [Current Scope](#current-scope)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Rust API](#core-rust-api)
- [Examples](#examples)
- [CRIF Format](#crif-format)
- [Testing](#testing)
- [Roadmap](#roadmap)
- [Credits](#credits)
- [References](#references)
- [License](#license)

## What is SIMM?

SIMM (Standard Initial Margin Model) is a standardized methodology developed by ISDA (International Swaps and Derivatives Association) in 2013 for calculating the appropriate level of initial margin (IM) for non-cleared derivatives. It emerged as a response to lessons learned from the Global Financial Crisis, particularly the defaults of Lehman Brothers and AIG, which highlighted the need for proper margining of derivative exposures.

### Core Principles

The SIMM methodology is built on six fundamental principles:

1. **Shock Application**: Stress scenarios applied to derivative portfolios
2. **Aggregation Methodology**: Systematic approach to combining risks
3. **99% Confidence Interval**: Loss estimation at 99th percentile
4. **10-Day Time Horizon**: Risk measurement over a 10-day closeout period
5. **Sensitivities-Based Approach**: Uses Greeks (Delta, Vega, Curvature) rather than full revaluation
6. **Collateral Haircuts**: Includes adjustments for collateral valuation

### Risk Classification

SIMM segments derivative instruments into six primary risk classes:

- **GIRR (General Interest Rate Risk)**: Interest rate derivatives
- **FX (Foreign Exchange)**: Currency derivatives
- **Equity**: Equity and equity index derivatives
- **Commodity**: Commodity derivatives (energy, metals, agriculture, etc.)
- **Credit Qualifying**: Investment-grade credit derivatives
- **Credit Non-Qualifying**: Non-investment-grade credit derivatives

### Calculation Methodology

The SIMM calculation combines three distinct margin components:

#### 1. Delta Margin
First-order sensitivity to underlying price changes. Captures the linear relationship between derivative value and the underlying asset price.

#### 2. Vega Margin
Sensitivity to changes in implied volatility. Measures exposure to volatility shifts in option-based derivatives.

#### 3. Curvature Margin
Second-order convexity effects (Gamma). Captures non-linear price relationships, particularly important for deep out-of-the-money options.

The methodology employs a delta-normal approximation:
```
VaR = |Delta₀| × (α × σ × S₀)
```
where α represents the standard normal deviate for the 99% confidence level.

## Features

- **Multiple SIMM Versions**: Support for SIMM 2.5, 2.6, and 2.7
- **Complete Risk Classes**: All six risk classes (GIRR, FX, Equity, Commodity, Credit Qualifying, Credit Non-Qualifying)
- **Three Margin Types**: Delta, Vega, and Curvature margin calculations
- **High Performance**: Optimized Rust implementation for fast calculations
- **CRIF Support**: Standard Common Risk Interchange Format (CSV/JSON)
- **Comprehensive Testing**: 481 test cases per SIMM version (1,443 total)
- **Well Documented**: Extensive API documentation and examples

## Current Scope

### Implemented

- **SIMM Versions**: 2.5, 2.6, and 2.7 with complete risk weights and correlations
- **Risk Classes**: All six primary risk classes with bucket subdivisions
- **Margin Components**: Delta, Vega, and Curvature margin calculations
- **Aggregation**: Product class and risk class aggregation with proper correlations
- **Add-On Handling**: Support for non-sensitivity-based add-ons
- **Input Formats**: CSV and JSON CRIF formats
- **Output Formats**: JSON output with summary and detailed breakdowns
- **Testing Framework**: Comprehensive test suite with validation against reference implementations

### Calculation Details

- Interest rate risk across multiple currencies and tenors
- FX risk with currency pair handling
- Credit risk with qualifier and bucket-based aggregation
- Equity risk with sector-based bucketing
- Commodity risk with subtype classifications
- Concentration thresholds for all risk types
- Cross-currency basis risk for interest rates
- Risk factor correlation matrices

## **WARNING** !!!!

**If you intend to use this library for commercial purposes, you need to reach out to ISDA SIMM (isdalegal@isda.org) to obtain a license. 
This library is provided AS IS without warranty of any kind.**

This work is solely intended for educational purposes.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
simm-rs = "0.1.0"
```

Or install via cargo:

```bash
cargo add simm-rs
```

## Quick Start

```rust,no_run
use simm_rs::{SIMM, EngineConfig, V2_5, parse_csv_from_string};

fn main() -> anyhow::Result<()> {
    // Create configuration for SIMM 2.5
    let cfg = EngineConfig {
        weights_and_corr_version: "2_5".to_string(),
        calculation_currency: "USD".to_string(),
        exchange_rate: 1.0,
    };

    // Load CRIF data (Common Risk Interchange Format)
    let crif_csv = std::fs::read_to_string("portfolio.csv")?;
    let crif = parse_csv_from_string(&crif_csv)?;

    // Calculate SIMM
    let wnc = V2_5;  // Weights and correlations for version 2.5
    let simm = SIMM::from_crif(crif, &cfg, &wnc)?;

    println!("Total SIMM: {:.2}", simm.simm);

    // Access detailed breakdown
    for row in simm.simm_break_down.iter() {
        println!("{:?}", row);
    }

    Ok(())
}
```

## Core Rust API

### Main Types

#### `SIMM`

The main calculator struct that holds the calculation results:

```rust,ignore
pub struct SIMM<'a> {
    pub simm: f64,                      // Total SIMM amount
    pub simm_break_down: Vec<Vec<String>>, // Detailed breakdown by risk class
    // ... internal fields
}
```

**Methods:**
- `SIMM::from_crif(crif: Crif, cfg: &EngineConfig, wnc: &dyn WeightsAndCorr) -> Result<Self>`
  - Creates a new SIMM calculator from CRIF data

#### `EngineConfig`

Configuration for SIMM calculations:

```rust,ignore
pub struct EngineConfig {
    pub weights_and_corr_version: String,  // "2_5", "2_6", or "2_7"
    pub calculation_currency: String,      // ISO currency code (e.g., "USD")
    pub exchange_rate: f64,                // Exchange rate to calculation currency
}
```

#### Version-Specific Weights and Correlations

```rust,ignore
pub struct V2_5;  // SIMM 2.5
pub struct V2_6;  // SIMM 2.6
pub struct V2_7;  // SIMM 2.7
```

All implement the `WeightsAndCorr` trait providing risk weights and correlation matrices.

### Utility Functions

#### File Processing

```rust,ignore
// Parse CSV string into CRIF format
pub fn parse_csv_from_string(csv_content: &str) -> anyhow::Result<Vec<Vec<String>>>;

// Read CSV file
pub fn read_csv_to_list(filepath: impl AsRef<Path>) -> Result<Crif>;

// Read JSON file
pub fn read_json_to_list(filepath: impl AsRef<Path>) -> Result<Crif>;

// Process CRIF file and generate output
pub fn process_crif_file(
    crif_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    cfg: &EngineConfig,
) -> Result<HashMap<String, String>>;
```

#### Calculation Functions

```rust,ignore
// Calculate K factors
pub fn k_delta(simm_s: f64) -> f64;
pub fn k_vega(simm_s: f64) -> f64;
pub fn k_curvature(simm_s: f64) -> f64;

// Calculate SIMM by measure
pub fn calculate_simm_by_measure(
    breakdown_list: &Crif,
    portfolio_crif: &Crif,
    measure_name: &str,
    wnc: &dyn WeightsAndCorr,
) -> f64;
```

### Type Aliases

```rust,ignore
pub type Crif = Vec<Vec<String>>;  // Common Risk Interchange Format
```

## Examples

### Example 1: Calculate SIMM from CSV

```rust,no_run
use simm_rs::{SIMM, EngineConfig, V2_5};
use simm_rs::file_utils::read_csv_to_list;

fn main() -> anyhow::Result<()> {
    let cfg = EngineConfig {
        weights_and_corr_version: "2_5".to_string(),
        calculation_currency: "USD".to_string(),
        exchange_rate: 1.0,
    };

    let crif = read_csv_to_list("tests_2_5/C1_crif.csv")?;
    let wnc = V2_5;
    let simm = SIMM::from_crif(crif, &cfg, &wnc)?;

    println!("SIMM Total: {:.0}", simm.simm);
    Ok(())
}
```

### Example 2: Using Different SIMM Versions

```rust,no_run
use simm_rs::{SIMM, EngineConfig, V2_5, V2_6, V2_7, parse_csv_from_string};

fn compare_versions(crif_csv: &str) -> anyhow::Result<()> {
    let crif = parse_csv_from_string(crif_csv)?;

    // Calculate with SIMM 2.5
    let cfg_2_5 = EngineConfig {
        weights_and_corr_version: "2_5".to_string(),
        calculation_currency: "USD".to_string(),
        exchange_rate: 1.0,
    };
    let simm_2_5 = SIMM::from_crif(crif.clone(), &cfg_2_5, &V2_5)?;

    // Calculate with SIMM 2.6
    let cfg_2_6 = EngineConfig {
        weights_and_corr_version: "2_6".to_string(),
        calculation_currency: "USD".to_string(),
        exchange_rate: 1.0,
    };
    let simm_2_6 = SIMM::from_crif(crif.clone(), &cfg_2_6, &V2_6)?;

    // Calculate with SIMM 2.7
    let cfg_2_7 = EngineConfig {
        weights_and_corr_version: "2_7".to_string(),
        calculation_currency: "USD".to_string(),
        exchange_rate: 1.0,
    };
    let simm_2_7 = SIMM::from_crif(crif, &cfg_2_7, &V2_7)?;

    println!("SIMM 2.5: {:.0}", simm_2_5.simm);
    println!("SIMM 2.6: {:.0}", simm_2_6.simm);
    println!("SIMM 2.7: {:.0}", simm_2_7.simm);

    Ok(())
}
```

### Example 3: Process and Compare Multiple Portfolios

```rust,no_run
use simm_rs::{EngineConfig, V2_5};
use simm_rs::file_utils::{process_crif_file, compare_csv_files};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let cfg = EngineConfig {
        weights_and_corr_version: "2_5".to_string(),
        calculation_currency: "USD".to_string(),
        exchange_rate: 1.0,
    };

    let tests_dir = PathBuf::from("tests_2_5");

    // Process a CRIF file
    let crif_path = tests_dir.join("C1_crif.csv");
    let output_path = tests_dir.join("C1_calc_output.csv");
    let expected_path = tests_dir.join("C1_expected_output.csv");

    let summary = process_crif_file(&crif_path, &output_path, &cfg)?;
    println!("Calculated SIMM: {}", summary.get("SIMM Benchmark").unwrap());

    // Compare with expected output
    let (matches, differences) = compare_csv_files(&output_path, &expected_path)?;
    if matches {
        println!("✓ Output matches expected results");
    } else {
        println!("✗ Differences found:");
        for diff in differences {
            println!("{}", diff);
        }
    }

    Ok(())
}
```

### Example 4: Custom JSON Output

```rust,no_run
use simm_rs::calc_simm;

fn main() -> anyhow::Result<()> {
    let crif_csv = std::fs::read_to_string("portfolio.csv")?;

    let result_json = calc_simm("2_5", "USD", 1.0, &crif_csv)?;

    // Parse JSON output
    let parsed: serde_json::Value = serde_json::from_str(&result_json)?;

    println!("Summary:");
    if let Some(summary) = parsed.get("summary") {
        println!("  Delta: {}", summary["SIMM Delta"]);
        println!("  Vega: {}", summary["SIMM Vega"]);
        println!("  Curvature: {}", summary["SIMM Curvature"]);
        println!("  Base Corr: {}", summary["SIMM Base Corr"]);
        println!("  AddOn: {}", summary["SIMM AddOn"]);
        println!("  Total: {}", summary["SIMM Benchmark"]);
    }

    Ok(())
}
```

## CRIF Format

The Common Risk Interchange Format (CRIF) is a standardized CSV format for representing derivative sensitivities:

| Column | Description | Example |
|--------|-------------|---------|
| ProductClass | Product classification | RatesFX, Rates, Credit, Equity, Commodity |
| RiskType | Type of risk | Risk_IRCurve, Risk_FX, Risk_CreditQ, Risk_Equity, Risk_Commodity |
| Qualifier | Risk identifier | USD, EUR, AAPL, WTI |
| Bucket | Risk bucketing | Currency code, sector number, rating category |
| Label1 | Primary label (usually tenor) | 1y, 5y, 10y, 2w, 1m |
| Label2 | Secondary label | Sub-curve identifier or empty |
| Amount | Sensitivity amount | 1000000.00 |
| AmountCurrency | Currency of amount | USD |
| AmountUSD | Amount in USD | 1000000.00 |

### Example CRIF CSV

```csv
ProductClass,RiskType,Qualifier,Bucket,Label1,Label2,Amount,AmountCurrency,AmountUSD
RatesFX,Risk_IRCurve,USD,1,5y,,1000000,USD,1000000
RatesFX,Risk_FX,EURUSD,,,,-500000,EUR,550000
Equity,Risk_Equity,AAPL,1,,,2000000,USD,2000000
Credit,Risk_CreditQ,CORP123,3,5y,,100000,USD,100000
Commodity,Risk_Commodity,WTI,1,,,50000,USD,50000
```

## Testing

The library includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run tests for a specific version
cargo test test_all_simm_v2_5_calculations
cargo test test_all_simm_v2_6_calculations
cargo test test_all_simm_v2_7_calculations

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_calc_simm_with_c298_crif
```

### Test Coverage

- **SIMM 2.5**: 481 test cases with expected output validation
- **SIMM 2.6**: 481 test cases
- **SIMM 2.7**: 481 test cases
- **Total**: 1,443 test cases covering all risk classes and scenarios

All v2.5 tests validate against ISDA reference implementations with rounding tolerance for large numbers (>1 billion).

## Roadmap

### Version 0.2.0 (Planned)

- **SIMM 2.8 Support**: Add latest ISDA SIMM version
- **Parallel Processing**: Multi-threaded calculation for large portfolios
- **Caching**: Optimize repeated calculations with intelligent caching
- **Additional Input Formats**: Support for Parquet and Arrow
- **Python Bindings**: PyO3-based Python API

### Version 0.3.0 (Planned)

- **Interactive CLI**: Command-line tool for ad-hoc calculations
- **Batch Processing**: Process multiple portfolios efficiently
- **Incremental Calculations**: Update margins for portfolio changes
- **Risk Attribution**: Detailed risk factor contribution analysis
- **What-If Analysis**: Scenario testing and stress testing capabilities

### Version 1.0.0 (Future)

- **WebAssembly Support**: Browser-based calculations
- **REST API Server**: Microservice for SIMM calculations
- **Real-time Monitoring**: Live margin monitoring and alerts
- **Historical Analysis**: Time-series margin analytics
- **Optimization Tools**: Portfolio optimization for margin efficiency
- **Regulatory Reporting**: Automated report generation

### Potential Enhancements

- [ ] Support for additional margin models (e.g., ISDA GRID)
- [ ] Integration with market data providers
- [ ] Machine learning for margin prediction
- [ ] Blockchain integration for margin settlement
- [ ] Cloud-native deployment options

### Author

**Suhas Ghorpadkar**
- Email: suhasghorp@gmail.com
- GitHub: [@suhasghorp](https://github.com/suhasghorp)

### Acknowledgments

- **ISDA** - For developing and maintaining the SIMM methodology
- **Rust Community** - For the excellent ecosystem of libraries that made this project possible

### Technology Stack

This project is built with:

- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[serde](https://serde.rs/)** - Serialization framework
- **[csv](https://docs.rs/csv/)** - CSV parsing
- **[statrs](https://docs.rs/statrs/)** - Statistical distributions
- **[anyhow](https://docs.rs/anyhow/)** - Error handling
- **[nalgebra](https://nalgebra.org/)** - Linear algebra operations

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

Please make sure to update tests as appropriate and follow the existing code style.

## References

- **ISDA SIMM Methodology**: [https://www.isda.org/margin-reform/](https://www.isda.org/margin-reform/)
- **Introduction to SIMM**: [https://benjaminwhiteside.com/2020/03/08/introduction-to-simm/](https://benjaminwhiteside.com/2020/03/08/introduction-to-simm/)
- **BCBS-IOSCO Framework**: Basel Committee on Banking Supervision and International Organization of Securities Commissions framework for margin requirements
- **CRIF Specification**: Common Risk Interchange Format documentation

## License
```
MIT License

Copyright (c) 2024 Suhas Ghorpadkar

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

**Note**: This is an open-source implementation for educational and development purposes. For production use in regulated environments, please ensure compliance with ISDA and your organization's risk management policies and regulatory requirements. This library is not officially endorsed by ISDA.

## Support

If you find this project helpful, please consider:

- Starring the repository
- Reporting bugs and issues
- Suggesting new features
- Improving documentation
- Contributing code

For questions and discussions, please open an issue on GitHub.
