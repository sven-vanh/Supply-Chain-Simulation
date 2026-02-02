/// Monthly simulation logic for inventory management and ordering decisions
/// Supports multiple products with shared supplier capacity

use std::cmp;
use std::collections::HashMap;
use crate::models::{MonthlyOrder, MonthlyResult, ProductMonthlyResult, ProductOrder, SimulationParams, SupplierPair};
use crate::demand::simulation_demand;
use crate::options::OptionValuation;
use crate::optimizer::find_optimal_production_quantities;

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
    // Track inventory per product
    let mut inventories: HashMap<usize, u32> = HashMap::new();
    for product in &params.products {
        inventories.insert(product.id, 0);
    }
    
    let mut total_profit: f64 = 0.0;
    let mut monthly_results: Vec<MonthlyResult> = Vec::new();

    // Track order changes with lead time
    let mut current_order = initial_order.clone();
    let mut pending_order: Option<(usize, MonthlyOrder)> = None; // (effective_month, new_order)
    let mut base_setup_cost_deducted = false;
    let mut surge_setup_cost_deducted = false;

    for (month_idx, month_name) in MONTHS.iter().enumerate() {
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

        // Deduct setup costs on first order from each supplier (once per supplier, not per product)
        if current_order.total_base_quantity() > 0 && !base_setup_cost_deducted {
            setup_cost_this_month += pair.base_supplier.setup_cost;
            base_setup_cost_deducted = true;
        }
        if current_order.total_surge_quantity() > 0 && !surge_setup_cost_deducted {
            setup_cost_this_month += pair.surge_supplier.setup_cost;
            surge_setup_cost_deducted = true;
        }

        let mut product_results: Vec<ProductMonthlyResult> = Vec::new();
        let mut monthly_revenue = 0.0;
        let mut monthly_production_cost = 0.0;
        let mut monthly_holding_cost = 0.0;
        let mut monthly_liquidation_revenue = 0.0;

        // Process each product
        for product in &params.products {
            let product_id = product.id;
            let inventory_start = *inventories.get(&product_id).unwrap_or(&0);

            // Get incoming inventory for this product
            let base_incoming = current_order.base_quantity_for(product_id);
            let surge_incoming = current_order.surge_quantity_for(product_id);
            let incoming = base_incoming + surge_incoming;
            
            let inventory_after_incoming = inventory_start + incoming;

            // Generate demand for this product
            let demand_params = params.get_demand_params(product_id);
            let monthly_demand = demand_params
                .map(|dp| simulation_demand(dp, use_actual_demand))
                .unwrap_or(0);

            // Calculate sales
            let units_sold = cmp::min(inventory_after_incoming, monthly_demand);
            let mut inventory_end = inventory_after_incoming - units_sold;

            // Calculate revenue and costs for this product
            let revenue = (units_sold as f64) * product.selling_price;
            
            // Production cost uses supplier-specific unit costs for this product
            let base_unit_cost = pair.base_supplier.unit_costs.get(&product_id).copied().unwrap_or(0.0);
            let surge_unit_cost = pair.surge_supplier.unit_costs.get(&product_id).copied().unwrap_or(0.0);
            let production_cost = (base_incoming as f64) * base_unit_cost 
                + (surge_incoming as f64) * surge_unit_cost;
            
            let holding_cost = (inventory_end as f64) * product.monthly_holding_cost;

            let mut liquidation_revenue = 0.0;

            // If December, liquidate remaining inventory
            if month_idx == TOTAL_MONTHS - 1 {
                liquidation_revenue = (inventory_end as f64) * product.liquidation_price;
                inventory_end = 0;
            }

            // Update inventory for next month
            inventories.insert(product_id, inventory_end);

            // Accumulate totals
            monthly_revenue += revenue;
            monthly_production_cost += production_cost;
            monthly_holding_cost += holding_cost;
            monthly_liquidation_revenue += liquidation_revenue;

            product_results.push(ProductMonthlyResult {
                product_id,
                product_name: product.name.clone(),
                inventory_start,
                incoming,
                demand: monthly_demand,
                units_sold,
                inventory_end,
                revenue,
                production_cost,
                holding_cost,
                liquidation_revenue,
            });
        }

        // Options valuation - evaluate whether to change monthly order
        // Only evaluate if we haven't already committed to a pending order change
        if enable_options && pending_order.is_none() && month_idx < 7 {
            // Get total current inventory
            let total_inventory: u32 = inventories.values().sum();
            let total_current_order = current_order.total_base_quantity() + current_order.total_surge_quantity();
            
            let option_valuer = OptionValuation::new(
                total_current_order,
                total_inventory,
                month_idx,
                params.clone(),
                pair.clone(),
            );

            let option_value = option_valuer.value_option();

            // Exercise the option if the value exceeds the fixed fee
            if option_value > params.order_change_fee {
                // Recalculate optimal based on FORECAST parameters
                let new_allocations = find_optimal_production_quantities(params, pair);
                
                // Create new surge order based on optimal allocations
                // Base quantity remains fixed from the initial plan
                let new_surge_orders: Vec<ProductOrder> = new_allocations.iter()
                    .map(|(product_id, optimal_qty)| {
                        let current_base = current_order.base_quantity_for(*product_id);
                        let desired_surge = optimal_qty.saturating_sub(current_base);
                        ProductOrder {
                            product_id: *product_id,
                            quantity: desired_surge,
                        }
                    })
                    .collect();

                // Ensure surge doesn't exceed capacity
                let total_new_surge: u32 = new_surge_orders.iter().map(|o| o.quantity).sum();
                let surge_capacity = pair.surge_supplier.fixed_capacity;
                
                let final_surge_orders = if total_new_surge > surge_capacity {
                    // Scale down proportionally
                    let scale = surge_capacity as f64 / total_new_surge as f64;
                    new_surge_orders.iter()
                        .map(|o| ProductOrder {
                            product_id: o.product_id,
                            quantity: (o.quantity as f64 * scale) as u32,
                        })
                        .collect()
                } else {
                    new_surge_orders
                };

                let new_order = MonthlyOrder {
                    base_orders: current_order.base_orders.clone(),
                    surge_orders: final_surge_orders,
                };

                // Schedule the order change to take effect after SURGE supplier's lead time
                let effective_month = month_idx + pair.surge_supplier.lead_time_months;
                if effective_month < TOTAL_MONTHS {
                    pending_order = Some((effective_month, new_order));
                }
            }
        }

        // Calculate monthly profit
        let monthly_profit = monthly_revenue - monthly_production_cost - monthly_holding_cost 
            + monthly_liquidation_revenue - order_change_cost_this_month - setup_cost_this_month;

        total_profit += monthly_profit;

        monthly_results.push(MonthlyResult {
            month: month_name.to_string(),
            product_results,
            order_change_cost: order_change_cost_this_month,
            setup_cost: setup_cost_this_month,
            monthly_profit,
        });
    }

    (monthly_results, total_profit)
}

/// Split order quantity between base and surge suppliers for multiple products
/// Uses a heuristic based on available capacity and demand variability
pub fn split_order_quantities(
    product_quantities: &[(usize, u32)],  // (product_id, desired_quantity)
    pair: &SupplierPair,
    params: &SimulationParams,
) -> MonthlyOrder {
    let mut base_orders: Vec<ProductOrder> = Vec::new();
    let mut surge_orders: Vec<ProductOrder> = Vec::new();
    
    let mut base_capacity_remaining = pair.base_supplier.fixed_capacity;
    let mut surge_capacity_remaining = pair.surge_supplier.fixed_capacity;

    for (product_id, total_quantity) in product_quantities {
        // Get demand params for this product to calculate CV
        let cv = params.get_demand_params(*product_id)
            .map(|dp| dp.std_dev_demand / dp.mean_demand)
            .unwrap_or(0.2);

        // Allocation weights based on variability:
        // Low variability: favor stable base supplier
        // High variability: favor flexible surge supplier
        let base_weight = 1.0 / (1.0 + cv);

        // Desired allocation
        let ideal_base = (*total_quantity as f64 * base_weight) as u32;

        // Constrain to available capacity
        let base_quantity = cmp::min(ideal_base, base_capacity_remaining);
        let remaining = total_quantity.saturating_sub(base_quantity);
        let surge_quantity = cmp::min(remaining, surge_capacity_remaining);

        // Update remaining capacities
        base_capacity_remaining = base_capacity_remaining.saturating_sub(base_quantity);
        surge_capacity_remaining = surge_capacity_remaining.saturating_sub(surge_quantity);

        base_orders.push(ProductOrder {
            product_id: *product_id,
            quantity: base_quantity,
        });
        surge_orders.push(ProductOrder {
            product_id: *product_id,
            quantity: surge_quantity,
        });
    }

    MonthlyOrder {
        base_orders,
        surge_orders,
    }
}
