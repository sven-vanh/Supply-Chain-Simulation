/// Optimization module for finding optimal production quantities
/// This module handles the Monte Carlo optimization to find the best supply level

use crate::models::{MonthlyOrder, SimulationParams, SupplierPair};
use crate::simulation::run_monthly_simulation_internal;


/// Find optimal production quantity using Monte Carlo
/// Tests multiple supply candidates with real supplier costs
/// When called from within options valuation, options_enabled should be false to avoid infinite recursion
pub fn find_optimal_production_quantity(
    params: &SimulationParams,
    pair: &SupplierPair,
) -> u32 {
    find_optimal_production_quantity_internal(params, pair, false)
}

fn find_optimal_production_quantity_internal(
    params: &SimulationParams,
    pair: &SupplierPair,
    enable_options: bool,
) -> u32 {
    let num_candidates = 10;
    let simulations_per_candidate = 10;

    let base_demand = params.mean_demand;
    let min_factor = 0.7;
    let max_factor = 1.3;

    let mut best_profit = f64::NEG_INFINITY;
    let mut best_quantity = base_demand as u32;

    for i in 0..num_candidates {
        let factor = min_factor + (max_factor - min_factor) * (i as f64 / (num_candidates - 1) as f64);
        let candidate_quantity = (base_demand * factor) as u32;

        let monthly_order = MonthlyOrder {
            base_quantity: candidate_quantity,
            surge_quantity: 0,
        };

        // Run quick Monte Carlo to estimate profit for this candidate
        let mut profits = Vec::new();
        for _ in 0..simulations_per_candidate {
            let (_, total_profit) = run_monthly_simulation_internal(
                params,
                pair,
                &monthly_order,
                enable_options,
            );
            profits.push(total_profit);
        }

        let mean_profit = profits.iter().sum::<f64>() / profits.len() as f64;

        if mean_profit > best_profit {
            best_profit = mean_profit;
            best_quantity = candidate_quantity;
        }
    }

    best_quantity
}

/// Find optimal production quantity with diagnostic output
/// Shows detailed results during the optimization
pub fn find_optimal_production_quantity_with_diagnostics(
    params: &SimulationParams,
    pair: &SupplierPair,
) -> u32 {
    let num_candidates = 10;
    let simulations_per_candidate = 10;

    let base_demand = params.mean_demand;
    let min_factor = 0.7;
    let max_factor = 1.3;

    let mut candidates: Vec<(u32, f64)> = Vec::new();
    let mut best_profit = f64::NEG_INFINITY;
    let mut best_quantity = base_demand as u32;

    for i in 0..num_candidates {
        let factor = min_factor + (max_factor - min_factor) * (i as f64 / (num_candidates - 1) as f64);
        let candidate_quantity = (base_demand * factor) as u32;

        let monthly_order = MonthlyOrder {
            base_quantity: candidate_quantity,
            surge_quantity: 0,
        };

        // Run quick Monte Carlo to estimate profit for this candidate (with options valuation)
        let mut profits = Vec::new();
        for _ in 0..simulations_per_candidate {
            let (_, total_profit) = run_monthly_simulation_internal(
                params,
                pair,
                &monthly_order,
                true,  // Enable options during initial optimization
            );
            profits.push(total_profit);
        }

        let mean_profit = profits.iter().sum::<f64>() / profits.len() as f64;
        candidates.push((candidate_quantity, mean_profit));

        if mean_profit > best_profit {
            best_profit = mean_profit;
            best_quantity = candidate_quantity;
        }
    }

    // Display optimization results
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("\n    Supply Level Optimization Results:");
    for (rank, (qty, profit)) in candidates.iter().take(3).enumerate() {
        println!("    {}. {} units: ${:.2}", rank + 1, qty, profit);
    }

    best_quantity
}
