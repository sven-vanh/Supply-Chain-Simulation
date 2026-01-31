# Supply Chain Simulation

A Monte Carlo simulation for optimizing multi-supplier supply chain strategies with demand uncertainty.

## Overview

This Rust-based simulator evaluates supplier combinations to find optimal order quantities under uncertain demand. It uses intelligent pairing of long-lead-time (base) and short-lead-time (surge) suppliers to balance cost and flexibility.

## Features

- **Monte Carlo Simulation**: Runs multiple scenarios to assess profit distributions
- **Supplier Pairing**: Automatically generates intelligent base/surge supplier combinations
- **Optimization**: Finds optimal production quantities using gradient descent
- **Real Options**: Models the value of order flexibility and change decisions
- **Comprehensive Reporting**: Displays profit statistics and scenario analysis

## Build & Run

### Prerequisites
- Rust toolchain (1.56 or later)

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run --release
```

## Simulation Parameters

All parameters are configured in [src/main.rs](src/main.rs):

### Demand Parameters
- `mean_demand`: Expected demand used for decision-making (e.g., 60,000)
- `std_dev_demand`: Standard deviation of expected demand (e.g., 12,000)
- `actual_mean_demand`: Realized demand during simulation (e.g., 53,000)
- `actual_std_dev_demand`: Standard deviation of actual demand (e.g., 12,000)

### Financial Parameters
- `selling_price`: Revenue per unit sold (e.g., $230)
- `monthly_holding_cost`: Cost to hold one unit for one month (e.g., $4.60)
- `liquidation_price`: Salvage value for unsold inventory (e.g., $144)
- `order_change_fee`: Fixed cost to modify orders (e.g., $2,000,000)

### Supplier Configuration
Edit the `suppliers` vector to modify:
- `name`: Supplier identifier
- `fixed_capacity`: Maximum units per order
- `lead_time_months`: Delivery time (0-4 months)
- `unit_cost`: Production cost per unit
- `setup_cost`: Fixed cost per order

### Simulation Settings
- `num_simulations`: Number of Monte Carlo runs per supplier pair (default: 500)

## Project Structure

- `main.rs`: Entry point and parameter configuration
- `models.rs`: Core data structures
- `optimizer.rs`: Production quantity optimization
- `simulation.rs`: Order splitting and simulation logic
- `monte_carlo.rs`: Monte Carlo runner
- `options.rs`: Real options valuation
- `demand.rs`: Demand generation
- `pairing.rs`: Supplier pair generation
- `reporting.rs`: Output formatting
