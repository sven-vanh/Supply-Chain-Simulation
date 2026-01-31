/// Monte Carlo simulation and statistical analysis module

use crate::models::{MonteCarloStats, MonthlyOrder, SimulationParams, SupplierPair};
use crate::simulation::run_monthly_simulation;

/// Run Monte Carlo simulation for a supplier combination
/// Executes the simulation many times to gather statistics
pub fn run_monte_carlo_simulation(
    params: &SimulationParams,
    pair: &SupplierPair,
    monthly_order: &MonthlyOrder,
    num_simulations: usize,
) -> MonteCarloStats {
    let mut profits = Vec::with_capacity(num_simulations);

    // Run simulation multiple times
    for _ in 0..num_simulations {
        let (_, total_profit) = run_monthly_simulation(params, pair, monthly_order);
        profits.push(total_profit);
    }

    // Calculate statistics
    profits.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mean_profit = profits.iter().sum::<f64>() / profits.len() as f64;
    let variance = profits
        .iter()
        .map(|p| (p - mean_profit).powi(2))
        .sum::<f64>()
        / profits.len() as f64;
    let std_dev_profit = variance.sqrt();

    let min_profit = profits.first().copied().unwrap_or(0.0);
    let max_profit = profits.last().copied().unwrap_or(0.0);

    // Calculate percentiles
    let percentile = |p: f64| {
        let index = ((p / 100.0) * (profits.len() as f64 - 1.0)).round() as usize;
        profits[index.min(profits.len() - 1)]
    };

    MonteCarloStats {
        base_supplier: pair.base_supplier.name.clone(),
        base_supplier_lead_time: pair.base_supplier.lead_time_months,
        surge_supplier: pair.surge_supplier.name.clone(),
        surge_supplier_lead_time: pair.surge_supplier.lead_time_months,
        optimal_quantity: monthly_order.base_quantity + monthly_order.surge_quantity,
        num_simulations,
        mean_profit,
        std_dev_profit,
        min_profit,
        max_profit,
        percentile_10: percentile(10.0),
        percentile_25: percentile(25.0),
        percentile_50: percentile(50.0),
        percentile_75: percentile(75.0),
        percentile_90: percentile(90.0),
    }
}
