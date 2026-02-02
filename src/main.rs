mod models;
mod options;
mod demand;
mod simulation;
mod optimizer;
mod monte_carlo;
mod reporting;
mod pairing;
mod pairing_utils;
mod capacity;

use std::collections::HashMap;
use std::io;
use models::{Product, ProductDemandParams, Supplier, SimulationParams};
use optimizer::find_optimal_production_quantities_with_diagnostics;
use simulation::split_order_quantities;
use monte_carlo::run_monte_carlo_simulation;
use reporting::{display_optimization_start, display_finding_optimal, display_found_quantities,
                display_combination_results, display_all_results, display_best_result};
use pairing::generate_intelligent_pairs;
use pairing_utils::{quick_profit_estimate, is_pair_promising};

fn main() {
    // Initialize products
    let products = vec![
        Product {
            id: 0,
            name: "Model A".to_string(),
            selling_price: 230.0,
            liquidation_price: 144.0,
            monthly_holding_cost: 4.60,
        },
        Product {
            id: 1,
            name: "Model B".to_string(),
            selling_price: 280.0,
            liquidation_price: 175.0,
            monthly_holding_cost: 5.60,
        },
    ];

    // Initialize demand parameters per product
    // Expected demand is what the model uses for decisions
    // Actual demand is what happens during the season (may differ)
    let demand_params = vec![
        ProductDemandParams {
            product_id: 0,
            mean_demand: 35_000.0,
            std_dev_demand: 7_000.0,
            actual_mean_demand: 32_000.0,
            actual_std_dev_demand: 7_000.0,
        },
        ProductDemandParams {
            product_id: 1,
            mean_demand: 25_000.0,
            std_dev_demand: 8_000.0,
            actual_mean_demand: 28_000.0,
            actual_std_dev_demand: 8_000.0,
        },
    ];

    let params = SimulationParams {
        products,
        demand_params,
        order_change_fee: 2_000_000.0,
    };

    // Initialize suppliers with unit costs per product
    // Product 0 = Model A, Product 1 = Model B
    let suppliers = vec![
        Supplier {
            id: 0,
            name: "FarFarAway".to_string(),
            fixed_capacity: 60_000,
            lead_time_months: 4,
            unit_costs: HashMap::from([
                (0, 160.0),  // Model A
                (1, 170.0),  // Model B (more complex to produce)
            ]),
            setup_cost: 1_000_000.0,
        },
        Supplier {
            id: 1,
            name: "FarAway".to_string(),
            fixed_capacity: 60_000,
            lead_time_months: 3,
            unit_costs: HashMap::from([
                (0, 160.0),  // Model A
                (1, 170.0),  // Model B
            ]),
            setup_cost: 2_000_000.0,
        },
        Supplier {
            id: 2,
            name: "PrettyClose".to_string(),
            fixed_capacity: 35_000,
            lead_time_months: 0,
            unit_costs: HashMap::from([
                (0, 170.0),  // Model A (premium for shorter lead time)
                (1, 180.0),  // Model B
            ]),
            setup_cost: 1_000_000.0,
        },
        Supplier {
            id: 3,
            name: "VeryClose".to_string(),
            fixed_capacity: 40_000,
            lead_time_months: 0,
            unit_costs: HashMap::from([
                (0, 170.0),  // Model A
                (1, 180.0),  // Model B
            ]),
            setup_cost: 2_000_000.0,
        },
    ];

    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║               MULTI-PRODUCT SUPPLY CHAIN SIMULATION                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");

    println!("Products:");
    for product in &params.products {
        let dp = params.get_demand_params(product.id).unwrap();
        println!("  {}: Price=${:.0}, Holding=${:.2}/mo, Liquidation=${:.0}",
                 product.name, product.selling_price, product.monthly_holding_cost, product.liquidation_price);
        println!("      Expected Demand: {:.0} ± {:.0}, Actual: {:.0} ± {:.0}",
                 dp.mean_demand, dp.std_dev_demand, dp.actual_mean_demand, dp.actual_std_dev_demand);
    }
    println!();

    // Generate intelligent supplier pairs (long lead time + short lead time)
    let pairs = generate_intelligent_pairs(&suppliers);

    println!("\n{} supplier pairs generated. Filtering for promising combinations...", pairs.len());

    // Quick profitability filter to skip obviously poor pairs
    let min_profit_threshold = 0.0; // Set to 0 to keep all pairs, or higher to filter aggressively
    let promising_pairs: Vec<_> = pairs.iter()
        .filter(|pair| is_pair_promising(&params, pair, min_profit_threshold))
        .collect();

    println!("{} pairs passed initial profitability screening.\n", promising_pairs.len());

    // Number of Monte Carlo simulations per supplier combination
    // 250 provides good statistical confidence; 500+ for publication-quality
    let num_simulations = 500;

    let mut all_results = Vec::new();
    let mut best_mean_profit = f64::NEG_INFINITY;

    // Iterate over promising supplier pairs
    for pair in &promising_pairs {
        // Quick profit estimate to show potential
        let quick_estimate = quick_profit_estimate(&params, pair);
        
        // Display evaluation progress
        display_optimization_start(&pair.base_supplier.name, &pair.surge_supplier.name);
        println!("  Quick estimate: ${:.2}", quick_estimate);

        // Step 1: Find optimal production quantities with diagnostics
        display_finding_optimal();
        let _ = io::Write::flush(&mut io::stdout());
        let optimal_quantities = find_optimal_production_quantities_with_diagnostics(&params, &pair);
        
        // Build display quantities with names
        let display_quantities: Vec<(usize, String, u32)> = optimal_quantities.iter()
            .map(|(id, qty)| {
                let name = params.get_product(*id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| format!("Product {}", id));
                (*id, name, *qty)
            })
            .collect();
        display_found_quantities(&display_quantities);

        // Step 2: Split order quantity between base and surge
        let monthly_order = split_order_quantities(
            &optimal_quantities,
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
        let best_result = &sorted_results[0];
        display_best_result(best_result);
    }
}
