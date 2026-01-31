use crate::models::{SimulationParams, SupplierPair};
use std::cmp;

/// American option valuation using binomial method
#[allow(dead_code)]
pub struct OptionValuation {
    current_order_quantity: u32,
    inventory: u32,
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
    /// inventory: current inventory state
    fn binomial_value_recursive(&self, period: usize, cumulative_uplifts: i32, inventory: u32) -> f64 {
        // Base case: reached end of evaluation period
        if period >= self.remaining_months {
            return 0.0;
        }

        // Calculate up and down demand scenarios using binomial model
        // Volatility calibration based on CV
        // sigma approx CV for log-normal, u = exp(sigma * sqrt(dt))
        // dt = 1 month = 1/12 year. But if std_dev is monthly, then sigma = CV directly.
        // CV = 12000 / 60000 = 0.2
        let cv = self.params.std_dev_demand / self.params.mean_demand;
        let u = (cv).exp(); // e^sigma
        let d = 1.0 / u;

        // Demand at this node in the binomial tree
        let base_demand = self.params.mean_demand;
        let demand_up = (base_demand * u.powi(cumulative_uplifts + 1)) as u32;
        let demand_down = (base_demand * d.powi(-(cumulative_uplifts + 1))) as u32;

        // Calculate payoff if we exercise the option NOW (change orders)
        let exercise_payoff = self.calculate_exercise_payoff(inventory, cumulative_uplifts, u);

        // Calculate continuation value (don't exercise, keep current order)
        // Up scenario
        let continuation_up = self.binomial_value_recursive(
            period + 1,
            cumulative_uplifts + 1,
            self.update_inventory(inventory, demand_up),
        );

        // Down scenario
        let continuation_down = self.binomial_value_recursive(
            period + 1,
            cumulative_uplifts - 1,
            self.update_inventory(inventory, demand_down),
        );

        // Risk-neutral probability
        // P = (e^(r*dt) - d) / (u - d). Assuming r=0 for simplicity in this supply chain context
        let p = (1.0 - d) / (u - d);
        let continuation_value = p * continuation_up + (1.0 - p) * continuation_down;

        // American option: take maximum of exercising now or waiting
        exercise_payoff.max(continuation_value)
    }

    /// Calculate the payoff from exercising the option (changing the order)
    /// Uses Newsvendor logic: Payoff = Expected reduction in mismatch costs
    fn calculate_exercise_payoff(&self, _current_inventory: u32, cumulative_uplifts: i32, u: f64) -> f64 {
        // Forecasted demand at this node
        let forecast_demand = self.params.mean_demand * u.powi(cumulative_uplifts);
        
        // Cost parameters
        let overage_cost = self.params.monthly_holding_cost;
        let underage_cost = self.params.selling_price - self.pair.surge_supplier.unit_cost; // Lost margin
        
        // Critical fractile (Newsvendor target service level)
        let critical_fractile = underage_cost / (underage_cost + overage_cost);
        
        // Calculate optimal Q* for this node's forecast
        // Q* = mean + z * std_dev
        // Assuming std_dev scales with mean (constant CV)
        let node_std_dev = forecast_demand * (self.params.std_dev_demand / self.params.mean_demand);
        
        // Inverse Normal CDF approx for z-score (simple approx or just use 0.5 + shift)
        // For simplicity, let's assume Normal dist. 
        // We can use the 'statrs' crate or a simple approximation. 
        // Given constraints, let's use a widely available approximation for inv_cdf:
        // Or since we don't have statrs, let's use a simple linear approx for the z-score near 0.5-0.9
        // This is a placeholder for a true inv_cdf.
        // Let's assume z corresponds to the critical fractile.
        let z_score = if critical_fractile > 0.5 { 1.645 } else { 0.0 }; // simplified
        
        let optimal_q = forecast_demand + z_score * node_std_dev;
        
        // Payoff heuristic:
        // The value of switching is the difference in Expected Profit between New Optimal Q and Old Q
        // Expected Profit Function G(Q) = (p-c)*mean - (p-c+h)*ExpectedOverstock - ...
        // Simplified: Value = Loss Function Reduction
        // Loss L(Q) = (Cu + Co) * sigma * L((Q-mu)/sigma)
        // For now, let's stick to the simpler margin heuristic but corrected for probability
        
        let new_q = optimal_q;
        let old_q = self.current_order_quantity as f64;
        
        // If we represent the benefit as escaping the "quadratic cost" of mismatch
        // Benefit ~ k * (Q_new - Q_old)^2 / 2 ???
        // Let's stick to the previous linear heuristic but strictly capped by the demand reality
        
        let margin = self.params.selling_price - self.pair.surge_supplier.unit_cost;
        
        // Improvement in filled demand (underage reduction)
        let demand_captured_improvement = if new_q > old_q {
             (new_q - old_q).min(forecast_demand) // can't capture more than demand
        } else {
             0.0 // Reducing stock doesn't capture more demand, it saves holding cost
        };
        
        let holding_savings = if new_q < old_q {
            (old_q - new_q) * self.params.monthly_holding_cost
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
