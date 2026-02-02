/// Utility functions for supplier pairing and quick profitability checks

use crate::models::{SimulationParams, SupplierPair};

/// Quick profitability check for a supplier pair
/// Returns estimated profit potential without full optimization
/// Used to filter out obviously poor supplier combinations early
pub fn quick_profit_estimate(
    params: &SimulationParams,
    pair: &SupplierPair,
) -> f64 {
    // Calculate weighted average unit cost
    let mut total_weighted_cost = 0.0;
    let mut total_demand = 0.0;
    
    for product in &params.products {
        let demand_params = params.get_demand_params(product.id);
        let demand = demand_params.map(|dp| dp.mean_demand).unwrap_or(0.0);
        
        // Use base supplier cost as representative
        let unit_cost = pair.base_supplier.unit_costs.get(&product.id).copied().unwrap_or(0.0);
        
        total_weighted_cost += demand * unit_cost;
        total_demand += demand;
    }
    
    let avg_cost = if total_demand > 0.0 {
        total_weighted_cost / total_demand
    } else {
        0.0
    };
    
    // Calculate weighted average selling price
    let mut total_revenue_potential = 0.0;
    for product in &params.products {
        let demand_params = params.get_demand_params(product.id);
        let demand = demand_params.map(|dp| dp.mean_demand).unwrap_or(0.0);
        total_revenue_potential += demand * product.selling_price;
    }
    
    let avg_price = if total_demand > 0.0 {
        total_revenue_potential / total_demand
    } else {
        0.0
    };
    
    // Quick newsvendor-style estimate
    // Assume we order ~90% of expected demand (conservative)
    let order_quantity = total_demand * 0.9;
    
    // Estimated revenue (assuming we can sell what we ordered)
    let revenue = order_quantity * avg_price;
    
    // Estimated costs
    let production_cost = order_quantity * avg_cost;
    let setup_cost = pair.base_supplier.setup_cost;
    
    // Rough holding cost estimate (assume 20% inventory carryover)
    let avg_holding = params.products.iter()
        .map(|p| p.monthly_holding_cost)
        .sum::<f64>() / params.products.len() as f64;
    let holding_cost = order_quantity * 0.2 * avg_holding * 8.0; // 8 months
    
    // Estimated profit
    revenue - production_cost - setup_cost - holding_cost
}

/// Check if a supplier pair is worth fully evaluating
/// Returns true if the pair passes basic profitability thresholds
pub fn is_pair_promising(
    params: &SimulationParams,
    pair: &SupplierPair,
    min_profit_threshold: f64,
) -> bool {
    let estimate = quick_profit_estimate(params, pair);
    
    // Also check if pair has sufficient capacity
    let total_capacity = pair.base_supplier.fixed_capacity + pair.surge_supplier.fixed_capacity;
    let total_demand: f64 = params.demand_params.iter()
        .map(|dp| dp.mean_demand)
        .sum();
    
    // Require capacity to be at least 70% of expected demand
    let has_adequate_capacity = (total_capacity as f64) >= (total_demand * 0.7);
    
    estimate >= min_profit_threshold && has_adequate_capacity
}
