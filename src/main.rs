mod models;
mod options;
mod demand;
mod simulation;
mod optimizer;
mod monte_carlo;
mod reporting;
mod pairing;

use std::io;
use models::{Supplier, SimulationParams};
use optimizer::find_optimal_production_quantity_with_diagnostics;
use simulation::split_order_quantity;
use monte_carlo::run_monte_carlo_simulation;
use reporting::{display_optimization_start, display_finding_optimal, display_found_quantity,
                display_combination_results, display_all_results, display_best_result};
use pairing::generate_intelligent_pairs;

fn main() {
    // Initialize simulation parameters
    // Expected demand is what the model uses for decisions
    // Actual demand is what happens during the season (may differ)
    let params = SimulationParams {
        mean_demand: 60_000.0,
        std_dev_demand: 12_000.0,
        // Actual demand has different mean than expected
        actual_mean_demand: 53_000.0,
        actual_std_dev_demand: 12_000.0,
        selling_price: 230.0,
        monthly_holding_cost: 4.60,
        liquidation_price: 144.0,
        order_change_fee: 2_000_000.0,
    };

    // Initialize suppliers with proper lead times and setup costs
    let suppliers = vec![
        Supplier {
            id: 0,
            name: "FarFarAway".to_string(),
            fixed_capacity: 60_000,
            lead_time_months: 4,
            unit_cost: 160.0,
            setup_cost: 1_000_000.0,
        },
        Supplier {
            id: 1,
            name: "FarAway".to_string(),
            fixed_capacity: 60_000,
            lead_time_months: 3,
            unit_cost: 160.0,
            setup_cost: 2_000_000.0,
        },
        Supplier {
            id: 2,
            name: "PrettyClose".to_string(),
            fixed_capacity: 35_000,
            lead_time_months: 0,
            unit_cost: 170.0,
            setup_cost: 1_000_000.0,
        },
        Supplier {
            id: 3,
            name: "VeryClose".to_string(),
            fixed_capacity: 40_000,
            lead_time_months: 0,
            unit_cost: 170.0,
            setup_cost: 2_000_000.0,
        },
    ];

    // Generate intelligent supplier pairs (long lead time + short lead time)
    let pairs = generate_intelligent_pairs(&suppliers);

    // Number of Monte Carlo simulations per supplier combination
    let num_simulations = 500;

    let mut all_results = Vec::new();
    let mut best_mean_profit = f64::NEG_INFINITY;

    // Iterate over all intelligent supplier pairs
    for pair in &pairs {
        // Display evaluation progress
        display_optimization_start(&pair.base_supplier.name, &pair.surge_supplier.name);

        // Step 1: Find optimal production quantity with diagnostics
        display_finding_optimal();
        let _ = io::Write::flush(&mut io::stdout());
        let optimal_quantity =
            find_optimal_production_quantity_with_diagnostics(&params, &pair);
        display_found_quantity(optimal_quantity);

        // Step 2: Split order quantity between base and surge
        let monthly_order = split_order_quantity(
            optimal_quantity,
            &pair,
            &params,
        );

        // Step 3: Run Monte Carlo simulation
        let stats = run_monte_carlo_simulation(
            &params,
            &pair,
            &monthly_order,
            num_simulations,
        );

        display_combination_results(
            stats.mean_profit,
            stats.std_dev_profit,
            stats.min_profit,
            stats.max_profit,
        );

        if stats.mean_profit > best_mean_profit {
            best_mean_profit = stats.mean_profit;
        }

        all_results.push(stats);
    }

    // Present Monte Carlo results for all combinations
    let mut sorted_results = all_results.clone();
    sorted_results.sort_by(|a, b| b.mean_profit.partial_cmp(&a.mean_profit).unwrap());
    display_all_results(sorted_results.clone());

    // Present best combination results
    if !sorted_results.is_empty() {
        let best_result = &sorted_results[0]; // First after sorting by mean profit
        display_best_result(best_result);
    }
}
