/// Monthly simulation logic for inventory management and ordering decisions

use std::cmp;
use crate::models::{MonthlyOrder, MonthlyResult, SimulationParams, SupplierPair};
use crate::demand::simulation_demand;
use crate::options::OptionValuation;
use crate::optimizer::find_optimal_production_quantity;

const MONTHS: &[&str] = &[
    "May", "June", "July", "August", "September", "October", "November", "December",
];
const TOTAL_MONTHS: usize = 8;

/// Run monthly simulation for May through December (8 months)
pub fn run_monthly_simulation(
    params: &SimulationParams,
    pair: &SupplierPair,
    initial_order: &MonthlyOrder,
) -> (Vec<MonthlyResult>, f64) {
    // Top-level simulation for final evaluation always uses actual demand
    run_monthly_simulation_internal(params, pair, initial_order, true, true)
}

/// Internal monthly simulation with optional options valuation
pub fn run_monthly_simulation_internal(
    params: &SimulationParams,
    pair: &SupplierPair,
    initial_order: &MonthlyOrder,
    enable_options: bool,
    use_actual_demand: bool,
) -> (Vec<MonthlyResult>, f64) {
    let mut inventory: u32 = 0;
    let mut total_profit: f64 = 0.0;
    let mut monthly_results: Vec<MonthlyResult> = Vec::new();

    // Track order changes with lead time
    let mut current_order = initial_order.clone();
    let mut pending_order: Option<(usize, MonthlyOrder)> = None; // (effective_month, new_order)
    let mut base_setup_cost_deducted = false;
    let mut surge_setup_cost_deducted = false;

    for (month_idx, month_name) in MONTHS.iter().enumerate() {
        let inventory_start = inventory;
        let mut order_change_cost_this_month = 0.0;
        let mut setup_cost_this_month = 0.0;

        // Check if a pending order should take effect this month
        if let Some((effective_month, new_order)) = &pending_order {
            if month_idx >= *effective_month {
                current_order = new_order.clone();
                pending_order = None;
                order_change_cost_this_month = params.order_change_fee;
            }
        }

        // Step 3a: Monthly inventory comes in
        let incoming_inventory = current_order.base_quantity + current_order.surge_quantity;
        inventory += incoming_inventory;
        let inventory_after_incoming = inventory;

        // Deduct setup costs on first order from each supplier
        if current_order.base_quantity > 0 && !base_setup_cost_deducted {
            setup_cost_this_month += pair.base_supplier.setup_cost;
            base_setup_cost_deducted = true;
        }
        if current_order.surge_quantity > 0 && !surge_setup_cost_deducted {
            setup_cost_this_month += pair.surge_supplier.setup_cost;
            surge_setup_cost_deducted = true;
        }

        // Step 3b: Monthly demand is deducted
        let monthly_demand = simulation_demand(params, use_actual_demand);
        let units_sold = cmp::min(inventory, monthly_demand);
        inventory -= units_sold;

        // Step 3c: Options valuation - evaluate whether to change monthly order
        // Only evaluate if we haven't already committed to a pending order change
        if enable_options {
            if pending_order.is_none() && month_idx < 7 {
                let option_valuer = OptionValuation::new(
                    current_order.base_quantity + current_order.surge_quantity,
                    inventory,
                    month_idx,
                    params.clone(),
                    pair.clone(),
                );

                let option_value = option_valuer.value_option();

                // Exercise the option if the value exceeds the fixed fee (threshold for profitability)
                // We need the option value to be greater than the fee to justify the exercise
                if option_value > params.order_change_fee {
                    // Recalculate optimal based on FORECAST parameters (passed in params)
                    // The optimizer internally will now use forecast demand
                    let new_optimal_quantity =
                        find_optimal_production_quantity(params, pair);
                    
                    // Decouple Base from Surge:
                    // Option exercise only changes the SURGE supplier's supply.
                    // Base quantity remains fixed from the initial plan.
                    
                    // Create new order keeping base quantity fixed
                    let current_base = current_order.base_quantity;
                    
                    // The new total target is new_optimal_quantity.
                    // Surge fills the gap between base and optimal, up to capacity.
                    let desired_surge = new_optimal_quantity.saturating_sub(current_base);
                    let final_surge = cmp::min(desired_surge, pair.surge_supplier.fixed_capacity);
                    
                    let new_order = MonthlyOrder {
                        base_quantity: current_base,
                        surge_quantity: final_surge,
                    };

                    // Schedule the order change to take effect after SURGE supplier's lead time
                    let effective_month = month_idx + pair.surge_supplier.lead_time_months;
                    if effective_month < TOTAL_MONTHS {
                        pending_order = Some((effective_month, new_order));
                    }
                }
            }
        }

        // Step 3d: Calculate monthly costs and revenue
        let revenue = (units_sold as f64) * params.selling_price;
        let production_cost = (current_order.base_quantity as f64) * pair.base_supplier.unit_cost
            + (current_order.surge_quantity as f64) * pair.surge_supplier.unit_cost;
        let holding_cost = (inventory as f64) * params.monthly_holding_cost;

        let mut liquidation_revenue = 0.0;
        let mut monthly_profit = revenue - production_cost - holding_cost - order_change_cost_this_month - setup_cost_this_month;

        // Step 3e: If December, liquidate remaining inventory
        if month_idx == TOTAL_MONTHS - 1 {
            liquidation_revenue = (inventory as f64) * params.liquidation_price;
            monthly_profit += liquidation_revenue;
            inventory = 0;
        }

        total_profit += monthly_profit;

        monthly_results.push(MonthlyResult {
            month: month_name.to_string(),
            inventory_start,
            incoming: incoming_inventory,
            inventory_after_incoming,
            demand: monthly_demand,
            units_sold,
            inventory_end: inventory,
            revenue,
            production_cost,
            holding_cost,
            liquidation_revenue,
            order_change_cost: order_change_cost_this_month,
            monthly_profit,
        });
    }

    (monthly_results, total_profit)
}

/// Split order quantity between base and surge suppliers
/// Uses a heuristic based on available capacity and demand variability
pub fn split_order_quantity(
    total_quantity: u32,
    pair: &SupplierPair,
    params: &SimulationParams,
) -> MonthlyOrder {
    // Heuristic: allocate based on capacity AND demand variability
    // Higher demand variability (higher coefficient of variation) suggests
    // allocating more to surge for flexibility

    let coefficient_of_variation = params.std_dev_demand / params.mean_demand;

    // Allocation weights based on variability:
    // Low variability: favor stable base supplier
    // High variability: favor flexible surge supplier
    let base_weight = 1.0 / (1.0 + coefficient_of_variation);

    // Desired allocation
    let ideal_base = (total_quantity as f64 * base_weight) as u32;

    // Constrain to available capacity
    let base_quantity = cmp::min(ideal_base, pair.base_supplier.fixed_capacity);
    let remaining = total_quantity - base_quantity;
    let surge_quantity = cmp::min(remaining, pair.surge_supplier.fixed_capacity);

    // If we couldn't fulfill the entire order due to capacity constraints,
    // try to reallocate to the other supplier
    if base_quantity + surge_quantity < total_quantity {
        let shortfall = total_quantity - (base_quantity + surge_quantity);
        // Attempt to get more base if possible (since we already maxed surge)
        let additional_base = cmp::min(shortfall, pair.base_supplier.fixed_capacity - base_quantity);
        let final_base_quantity = base_quantity + additional_base;
        let final_surge_quantity = surge_quantity;

        return MonthlyOrder {
            base_quantity: final_base_quantity,
            surge_quantity: final_surge_quantity,
        };
    }

    MonthlyOrder {
        base_quantity,
        surge_quantity,
    }
}
