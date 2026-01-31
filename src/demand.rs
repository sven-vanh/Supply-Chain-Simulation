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

/// Actual monthly demand - what occurs during the simulation
/// Drawn from a normal distribution that may differ from the expected distribution
#[allow(dead_code)]
pub fn actual_demand(params: &SimulationParams) -> u32 {
    let mut rng = thread_rng();
    let normal = Normal::new(params.actual_mean_demand, params.actual_std_dev_demand)
        .expect("Invalid normal distribution parameters");

    // Sample from the distribution and ensure non-negative
    let demand = rng.sample(normal);
    // Cap at 3 standard deviations above mean to prevent extreme outliers
    let max_reasonable_demand = params.actual_mean_demand + (3.0 * params.actual_std_dev_demand);
    (demand.max(0.0) as u32).min(max_reasonable_demand as u32)
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
