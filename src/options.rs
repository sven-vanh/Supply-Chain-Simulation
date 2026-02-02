use crate::models::{SimulationParams, SupplierPair};
use std::cmp;

/// American option valuation using binomial method
/// Updated to work with multi-product simulation using aggregate values
#[allow(dead_code)]
pub struct OptionValuation {
    current_order_quantity: u32,  // Total across all products
    inventory: u32,               // Total across all products
    current_month: usize,
    remaining_months: usize,
    params: SimulationParams,
    pair: SupplierPair,
}

impl OptionValuation {
    /// Create a new option valuation instance
    pub fn new(
        current_order_quantity: u32,
        inventory: u32,
        current_month: usize,
        params: SimulationParams,
        pair: SupplierPair,
    ) -> Self {
        let remaining_months = 8 - current_month; // 8 months total (May=0 to December=7)
        OptionValuation {
            current_order_quantity,
            inventory,
            current_month,
            remaining_months,
            params,
            pair,
        }
    }

    /// Get aggregate demand parameters (sum across products)
    fn get_aggregate_demand_params(&self) -> (f64, f64) {
        let total_mean: f64 = self.params.demand_params.iter()
            .map(|dp| dp.mean_demand)
            .sum();
        let total_std_dev: f64 = self.params.demand_params.iter()
            .map(|dp| dp.std_dev_demand)
            .sum();
        (total_mean, total_std_dev)
    }

    /// Get weighted average selling price
    fn get_avg_selling_price(&self) -> f64 {
        if self.params.products.is_empty() {
            return 0.0;
        }
        let total: f64 = self.params.products.iter()
            .map(|p| p.selling_price)
            .sum();
        total / self.params.products.len() as f64
    }

    /// Get weighted average holding cost
    fn get_avg_holding_cost(&self) -> f64 {
        if self.params.products.is_empty() {
            return 0.0;
        }
        let total: f64 = self.params.products.iter()
            .map(|p| p.monthly_holding_cost)
            .sum();
        total / self.params.products.len() as f64
    }

    /// Get average surge unit cost across products
    fn get_avg_surge_unit_cost(&self) -> f64 {
        if self.params.products.is_empty() {
            return 0.0;
        }
        let total: f64 = self.params.products.iter()
            .map(|p| self.pair.surge_supplier.unit_costs.get(&p.id).copied().unwrap_or(0.0))
            .sum();
        total / self.params.products.len() as f64
    }

    /// Calculate the value of the option to change orders using recursive binomial method
    pub fn value_option(&self) -> f64 {
        if self.remaining_months == 0 {
            return 0.0; // No time value left
        }

        self.binomial_value_recursive(0, 0, self.inventory)
    }

    /// Recursive binomial tree valuation
    /// period: current period (0 = current month)
    /// cumulative_uplifts: net up movements minus down movements
    /// inventory: current inventory state (aggregate across products)
    fn binomial_value_recursive(&self, period: usize, cumulative_uplifts: i32, inventory: u32) -> f64 {
        // Base case: reached end of evaluation period
        if period >= self.remaining_months {
            return 0.0;
        }

        let (mean_demand, std_dev_demand) = self.get_aggregate_demand_params();

        // Calculate up and down demand scenarios using binomial model
        let cv = std_dev_demand / mean_demand;
        let u = cv.exp();
        let d = 1.0 / u;

        // Demand at this node in the binomial tree
        let demand_up = (mean_demand * u.powi(cumulative_uplifts + 1)) as u32;
        let demand_down = (mean_demand * d.powi(-(cumulative_uplifts + 1))) as u32;

        // Calculate payoff if we exercise the option NOW (change orders)
        let exercise_payoff = self.calculate_exercise_payoff(inventory, cumulative_uplifts, u);

        // Calculate continuation value (don't exercise, keep current order)
        let continuation_up = self.binomial_value_recursive(
            period + 1,
            cumulative_uplifts + 1,
            self.update_inventory(inventory, demand_up),
        );

        let continuation_down = self.binomial_value_recursive(
            period + 1,
            cumulative_uplifts - 1,
            self.update_inventory(inventory, demand_down),
        );

        // Risk-neutral probability
        let p = (1.0 - d) / (u - d);
        let continuation_value = p * continuation_up + (1.0 - p) * continuation_down;

        // American option: take maximum of exercising now or waiting
        exercise_payoff.max(continuation_value)
    }

    /// Calculate the payoff from exercising the option (changing the order)
    fn calculate_exercise_payoff(&self, _current_inventory: u32, cumulative_uplifts: i32, u: f64) -> f64 {
        let (mean_demand, std_dev_demand) = self.get_aggregate_demand_params();
        
        // Forecasted demand at this node
        let forecast_demand = mean_demand * u.powi(cumulative_uplifts);
        
        // Cost parameters (using aggregate values)
        let overage_cost = self.get_avg_holding_cost();
        let underage_cost = self.get_avg_selling_price() - self.get_avg_surge_unit_cost();
        
        // Critical fractile (Newsvendor target service level)
        let critical_fractile = underage_cost / (underage_cost + overage_cost);
        
        // Calculate optimal Q* for this node's forecast
        let node_std_dev = forecast_demand * (std_dev_demand / mean_demand);
        
        let z_score = if critical_fractile > 0.5 { 1.645 } else { 0.0 };
        
        let optimal_q = forecast_demand + z_score * node_std_dev;
        
        let new_q = optimal_q;
        let old_q = self.current_order_quantity as f64;
        
        let margin = self.get_avg_selling_price() - self.get_avg_surge_unit_cost();
        
        // Improvement in filled demand (underage reduction)
        let demand_captured_improvement = if new_q > old_q {
             (new_q - old_q).min(forecast_demand)
        } else {
             0.0
        };
        
        let holding_savings = if new_q < old_q {
            (old_q - new_q) * self.get_avg_holding_cost()
        } else {
            0.0
        };

        // Benefit = (Extra Margin from more sales) + (Holding Cost Saved)
        let benefit = (demand_captured_improvement * margin) + holding_savings;

        // Net benefit after paying the fixed fee
        benefit - self.params.order_change_fee
    }

    /// Update inventory after demand realization
    fn update_inventory(&self, inventory: u32, demand: u32) -> u32 {
        let incoming = self.current_order_quantity;
        let new_inventory = inventory + incoming;
        let sold = cmp::min(new_inventory, demand);
        (new_inventory - sold).max(0)
    }
}
