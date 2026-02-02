/// Demand calculation module
/// Handles both expected demand (used for planning) and actual demand (realized during simulation)

use rand::{thread_rng, Rng};
use rand_distr::Normal;
use crate::models::{SimulationParams, ProductDemandParams};

/// Expected monthly demand for a specific product - used by the model for decision-making
/// Returns the mean of the expected demand distribution
#[allow(dead_code)]
pub fn expected_demand(demand_params: &ProductDemandParams) -> u32 {
    demand_params.mean_demand as u32
}

/// Expected monthly demand using SimulationParams - for backwards compatibility
#[allow(dead_code)]
pub fn expected_demand_for_product(params: &SimulationParams, product_id: usize) -> u32 {
    params.get_demand_params(product_id)
        .map(|dp| dp.mean_demand as u32)
        .unwrap_or(0)
}

/// Simulation monthly demand for a specific product - can be based on forecast (expected) or actuals
#[allow(dead_code)]
pub fn simulation_demand(demand_params: &ProductDemandParams, use_actual: bool) -> u32 {
    let mut rng = thread_rng();
    
    let (mean, std_dev) = if use_actual {
        (demand_params.actual_mean_demand, demand_params.actual_std_dev_demand)
    } else {
        (demand_params.mean_demand, demand_params.std_dev_demand)
    };

    let normal = Normal::new(mean, std_dev)
        .expect("Invalid normal distribution parameters");

    // Sample from the distribution and ensure non-negative
    let demand = rng.sample(normal);
    // Cap at 3 standard deviations above mean to prevent extreme outliers
    let max_reasonable_demand = mean + (3.0 * std_dev);
    (demand.max(0.0) as u32).min(max_reasonable_demand as u32)
}

/// Simulation monthly demand using SimulationParams
#[allow(dead_code)]
pub fn simulation_demand_for_product(params: &SimulationParams, product_id: usize, use_actual: bool) -> u32 {
    params.get_demand_params(product_id)
        .map(|dp| simulation_demand(dp, use_actual))
        .unwrap_or(0)
}

/// Generate demands for all products (independent demands)
#[allow(dead_code)]
pub fn simulation_demand_all_products(params: &SimulationParams, use_actual: bool) -> Vec<(usize, u32)> {
    params.demand_params.iter()
        .map(|dp| (dp.product_id, simulation_demand(dp, use_actual)))
        .collect()
}

/// Legacy actual demand wrapper for compatibility (uses actuals)
#[allow(dead_code)]
pub fn actual_demand(demand_params: &ProductDemandParams) -> u32 {
    simulation_demand(demand_params, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_demand() {
        let demand_params = ProductDemandParams {
            product_id: 0,
            mean_demand: 100.0,
            std_dev_demand: 20.0,
            actual_mean_demand: 100.0,
            actual_std_dev_demand: 25.0,
        };

        assert_eq!(expected_demand(&demand_params), 100);
    }

    #[test]
    fn test_actual_demand_is_non_negative() {
        let demand_params = ProductDemandParams {
            product_id: 0,
            mean_demand: 100.0,
            std_dev_demand: 20.0,
            actual_mean_demand: 100.0,
            actual_std_dev_demand: 25.0,
        };

        for _ in 0..100 {
            let demand = actual_demand(&demand_params);
            assert!(demand >= 0);
            assert!(demand <= 1000);
        }
    }
}
