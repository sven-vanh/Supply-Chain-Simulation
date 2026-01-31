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
        let u = 1.05_f64; // Up factor (5% higher demand)
        let d = 0.95_f64; // Down factor (5% lower demand)

        // Demand at this node in the binomial tree
        let base_demand = self.params.mean_demand;
        let demand_up = (base_demand * u.powi(cumulative_uplifts + 1)) as u32;
        let demand_down = (base_demand * d.powi(-(cumulative_uplifts + 1))) as u32;

        // Calculate payoff if we exercise the option NOW (change orders)
        let exercise_payoff = self.calculate_exercise_payoff(inventory, cumulative_uplifts);

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

        // Risk-neutral probability (assuming 50-50)
        let continuation_value = 0.5 * continuation_up + 0.5 * continuation_down;

        // American option: take maximum of exercising now or waiting
        exercise_payoff.max(continuation_value)
    }

    /// Calculate the payoff from exercising the option (changing the order)
    fn calculate_exercise_payoff(&self, _current_inventory: u32, cumulative_uplifts: i32) -> f64 {
        // Calculate what the expected future demand path looks like
        let u = 1.05_f64;
        let mean_future_demand = self.params.mean_demand * u.powi(cumulative_uplifts);

        // Calculate what the new optimal order would be based on future demand
        // Use the updated mean demand directly (no inflated multiplier)
        let new_order_qty = mean_future_demand as u32;
        let current_order = self.current_order_quantity as f64;

        // Divergence between current and new optimal order
        let order_diff = (new_order_qty as f64 - current_order).abs();

        // Calculate the benefit from adjusting inventory to meet future demand
        // Benefit = margin * (units sold due to better matching)
        let margin = self.params.selling_price -
            (self.pair.base_supplier.unit_cost + self.pair.surge_supplier.unit_cost) / 2.0;

        // Estimate benefit: if we change order by order_diff, we expect to improve
        // inventory matching over remaining months. Each unit of additional inventory
        // that matches demand generates the margin profit.
        // Expected improved units = order_diff (the adjustment amount)
        let expected_improved_units = order_diff;
        let benefit = expected_improved_units * margin;

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
