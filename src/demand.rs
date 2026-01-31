/// Demand calculation module
/// Handles both expected demand (used for planning) and actual demand (realized during simulation)

use rand::{thread_rng, Rng};
use rand_distr::Normal;
use crate::models::SimulationParams;

/// Expected monthly demand - used by the model for decision-making
/// Returns the mean of the expected demand distribution
#[allow(dead_code)]
pub fn expected_demand(params: &SimulationParams) -> u32 {
    params.mean_demand as u32
}

/// Simulation monthly demand - can be based on forecast (expected) or actuals
#[allow(dead_code)]
pub fn simulation_demand(params: &SimulationParams, use_actual: bool) -> u32 {
    let mut rng = thread_rng();
    
    let (mean, std_dev) = if use_actual {
        (params.actual_mean_demand, params.actual_std_dev_demand)
    } else {
        (params.mean_demand, params.std_dev_demand)
    };

    let normal = Normal::new(mean, std_dev)
        .expect("Invalid normal distribution parameters");

    // Sample from the distribution and ensure non-negative
    let demand = rng.sample(normal);
    // Cap at 3 standard deviations above mean to prevent extreme outliers
    let max_reasonable_demand = mean + (3.0 * std_dev);
    (demand.max(0.0) as u32).min(max_reasonable_demand as u32)
}

/// Legacy actual demand wrapper for compatibility (uses actuals)
#[allow(dead_code)]
pub fn actual_demand(params: &SimulationParams) -> u32 {
    simulation_demand(params, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_demand() {
        let params = SimulationParams {
            mean_demand: 100.0,
            std_dev_demand: 20.0,
            actual_mean_demand: 100.0,
            actual_std_dev_demand: 25.0,
            selling_price: 50.0,
            monthly_holding_cost: 5.0,
            liquidation_price: 20.0,
            unit_cost_base: 30.0,
            lead_time: 1,
            order_change_fee: 100.0,
        };

        assert_eq!(expected_demand(&params), 100);
    }

    #[test]
    fn test_actual_demand_is_non_negative() {
        let params = SimulationParams {
            mean_demand: 100.0,
            std_dev_demand: 20.0,
            actual_mean_demand: 100.0,
            actual_std_dev_demand: 25.0,
            selling_price: 50.0,
            monthly_holding_cost: 5.0,
            liquidation_price: 20.0,
            unit_cost_base: 30.0,
            lead_time: 1,
            order_change_fee: 100.0,
        };

        for _ in 0..100 {
            let demand = actual_demand(&params);
            assert!(demand >= 0);
            assert!(demand <= 1000);
        }
    }
}
