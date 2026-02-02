/// Optimization module for finding optimal production quantities
/// This module handles the grid search optimization to find the best supply levels for multiple products

use crate::models::{MonthlyOrder, ProductOrder, SimulationParams, SupplierPair};
use crate::simulation::run_monthly_simulation_internal;

/// Find optimal production quantities for all products using grid search
/// Tests multiple combinations within shared capacity constraints
/// When called from within options valuation, options_enabled should be false to avoid infinite recursion
pub fn find_optimal_production_quantities(
    params: &SimulationParams,
    pair: &SupplierPair,
) -> Vec<(usize, u32)> {
    find_optimal_production_quantities_internal(params, pair, false)
}

fn find_optimal_production_quantities_internal(
    params: &SimulationParams,
    pair: &SupplierPair,
    enable_options: bool,
) -> Vec<(usize, u32)> {
    let total_capacity = pair.base_supplier.fixed_capacity + pair.surge_supplier.fixed_capacity;
    
    // Get product IDs and their expected demands
    let products: Vec<(usize, f64)> = params.products.iter()
        .map(|p| {
            let mean = params.get_demand_params(p.id)
                .map(|dp| dp.mean_demand)
                .unwrap_or(0.0);
            (p.id, mean)
        })
        .collect();

    if products.len() == 1 {
        // Single product: use original approach
        return find_optimal_single_product(params, pair, enable_options, 15);
    }

    if products.len() != 2 {
        // For more than 2 products, use proportional allocation as fallback
        return allocate_proportionally(&products, total_capacity);
    }

    // Two-product coarse-to-fine grid search
    coarse_to_fine_grid_search(params, pair, &products, enable_options)
}

/// Coarse-to-fine grid search for two products
fn coarse_to_fine_grid_search(
    params: &SimulationParams,
    pair: &SupplierPair,
    products: &[(usize, f64)],
    enable_options: bool,
) -> Vec<(usize, u32)> {
    let (product_a_id, demand_a) = products[0];
    let (product_b_id, demand_b) = products[1];
    let total_capacity = pair.base_supplier.fixed_capacity + pair.surge_supplier.fixed_capacity;

    // Tighter bounds based on newsvendor theory
    let min_factor = 0.7;
    let max_factor = 1.2;

    // COARSE PASS: 6x6 grid with 30 simulations
    let coarse_steps = 5; // 6 points (0..=5)
    let coarse_sims = 30;
    let mut best_profit_coarse = f64::NEG_INFINITY;
    let mut best_a_coarse = demand_a as u32;
    let mut best_b_coarse = demand_b as u32;

    for i in 0..=coarse_steps {
        let factor_a = min_factor + (max_factor - min_factor) * (i as f64 / coarse_steps as f64);
        let qty_a = (demand_a * factor_a) as u32;

        for j in 0..=coarse_steps {
            let factor_b = min_factor + (max_factor - min_factor) * (j as f64 / coarse_steps as f64);
            let qty_b = (demand_b * factor_b) as u32;

            if qty_a + qty_b > total_capacity {
                continue;
            }

            let monthly_order = MonthlyOrder {
                base_orders: vec![
                    ProductOrder { product_id: product_a_id, quantity: qty_a },
                    ProductOrder { product_id: product_b_id, quantity: qty_b },
                ],
                surge_orders: vec![
                    ProductOrder { product_id: product_a_id, quantity: 0 },
                    ProductOrder { product_id: product_b_id, quantity: 0 },
                ],
            };

            let mut profits = Vec::new();
            for _ in 0..coarse_sims {
                let (_, total_profit) = run_monthly_simulation_internal(
                    params,
                    pair,
                    &monthly_order,
                    enable_options,
                    false,
                );
                profits.push(total_profit);
            }

            let mean_profit = profits.iter().sum::<f64>() / profits.len() as f64;

            if mean_profit > best_profit_coarse {
                best_profit_coarse = mean_profit;
                best_a_coarse = qty_a;
                best_b_coarse = qty_b;
            }
        }
    }

    // FINE PASS: 5x5 grid around best coarse point with 50 simulations
    let fine_steps = 4; // 5 points (0..=4)
    let fine_sims = 50;
    let mut best_profit_fine = best_profit_coarse;
    let mut best_allocation = vec![(product_a_id, best_a_coarse), (product_b_id, best_b_coarse)];

    // Define fine search window (±15% around best coarse point)
    let a_min = (best_a_coarse as f64 * 0.85) as u32;
    let a_max = (best_a_coarse as f64 * 1.15) as u32;
    let b_min = (best_b_coarse as f64 * 0.85) as u32;
    let b_max = (best_b_coarse as f64 * 1.15) as u32;

    for i in 0..=fine_steps {
        let qty_a = a_min + ((a_max - a_min) as f64 * (i as f64 / fine_steps as f64)) as u32;

        for j in 0..=fine_steps {
            let qty_b = b_min + ((b_max - b_min) as f64 * (j as f64 / fine_steps as f64)) as u32;

            if qty_a + qty_b > total_capacity {
                continue;
            }

            let monthly_order = MonthlyOrder {
                base_orders: vec![
                    ProductOrder { product_id: product_a_id, quantity: qty_a },
                    ProductOrder { product_id: product_b_id, quantity: qty_b },
                ],
                surge_orders: vec![
                    ProductOrder { product_id: product_a_id, quantity: 0 },
                    ProductOrder { product_id: product_b_id, quantity: 0 },
                ],
            };

            let mut profits = Vec::new();
            for _ in 0..fine_sims {
                let (_, total_profit) = run_monthly_simulation_internal(
                    params,
                    pair,
                    &monthly_order,
                    enable_options,
                    false,
                );
                profits.push(total_profit);
            }

            let mean_profit = profits.iter().sum::<f64>() / profits.len() as f64;

            if mean_profit > best_profit_fine {
                best_profit_fine = mean_profit;
                best_allocation = vec![(product_a_id, qty_a), (product_b_id, qty_b)];
            }
        }
    }

    best_allocation
}

/// Single product optimization (backward compatible)
fn find_optimal_single_product(
    params: &SimulationParams,
    pair: &SupplierPair,
    enable_options: bool,
    simulations_per_candidate: usize,
) -> Vec<(usize, u32)> {
    let product = &params.products[0];
    let base_demand = params.get_demand_params(product.id)
        .map(|dp| dp.mean_demand)
        .unwrap_or(60_000.0);

    let num_candidates = 12;
    let min_factor = 0.7;
    let max_factor = 1.2;

    let mut best_profit = f64::NEG_INFINITY;
    let mut best_quantity = base_demand as u32;

    for i in 0..num_candidates {
        let factor = min_factor + (max_factor - min_factor) * (i as f64 / (num_candidates - 1) as f64);
        let candidate_quantity = (base_demand * factor) as u32;

        let monthly_order = MonthlyOrder {
            base_orders: vec![ProductOrder { product_id: product.id, quantity: candidate_quantity }],
            surge_orders: vec![ProductOrder { product_id: product.id, quantity: 0 }],
        };

        let mut profits = Vec::new();
        for _ in 0..simulations_per_candidate {
            let (_, total_profit) = run_monthly_simulation_internal(
                params,
                pair,
                &monthly_order,
                enable_options,
                false,
            );
            profits.push(total_profit);
        }

        let mean_profit = profits.iter().sum::<f64>() / profits.len() as f64;

        if mean_profit > best_profit {
            best_profit = mean_profit;
            best_quantity = candidate_quantity;
        }
    }

    vec![(product.id, best_quantity)]
}

/// Allocate capacity proportionally to expected demand (fallback for 3+ products)
fn allocate_proportionally(products: &[(usize, f64)], total_capacity: u32) -> Vec<(usize, u32)> {
    let total_demand: f64 = products.iter().map(|(_, d)| d).sum();
    
    products.iter()
        .map(|(id, demand)| {
            let proportion = demand / total_demand;
            let qty = (total_capacity as f64 * proportion) as u32;
            (*id, qty)
        })
        .collect()
}

/// Find optimal production quantities with diagnostic output
/// Uses coarse-to-fine approach for efficiency
pub fn find_optimal_production_quantities_with_diagnostics(
    params: &SimulationParams,
    pair: &SupplierPair,
) -> Vec<(usize, u32)> {
    // Use the same coarse-to-fine approach
    find_optimal_production_quantities_internal(params, pair, false)
}
