/// Capacity allocation module
/// Handles shared capacity allocation between products for suppliers

use crate::models::{MonthlyOrder, ProductOrder, SupplierPair};

/// Error type for capacity constraint violations
#[allow(dead_code)]
#[derive(Debug)]
pub struct CapacityError {
    pub message: String,
    pub supplier_name: String,
    pub capacity: u32,
    pub requested: u32,
}

/// Validates that total product orders don't exceed supplier capacity
#[allow(dead_code)]
pub fn validate_capacity_constraint(
    order: &MonthlyOrder,
    pair: &SupplierPair,
) -> Result<(), CapacityError> {
    let total_base = order.total_base_quantity();
    let total_surge = order.total_surge_quantity();
    
    if total_base > pair.base_supplier.fixed_capacity {
        return Err(CapacityError {
            message: "Base supplier capacity exceeded".to_string(),
            supplier_name: pair.base_supplier.name.clone(),
            capacity: pair.base_supplier.fixed_capacity,
            requested: total_base,
        });
    }
    
    if total_surge > pair.surge_supplier.fixed_capacity {
        return Err(CapacityError {
            message: "Surge supplier capacity exceeded".to_string(),
            supplier_name: pair.surge_supplier.name.clone(),
            capacity: pair.surge_supplier.fixed_capacity,
            requested: total_surge,
        });
    }
    
    Ok(())
}

/// Create an empty order for the given product IDs
#[allow(dead_code)]
pub fn create_empty_order(product_ids: &[usize]) -> MonthlyOrder {
    MonthlyOrder {
        base_orders: product_ids.iter().map(|&id| ProductOrder { product_id: id, quantity: 0 }).collect(),
        surge_orders: product_ids.iter().map(|&id| ProductOrder { product_id: id, quantity: 0 }).collect(),
    }
}

/// Create an order with specified quantities for each product
#[allow(dead_code)]
pub fn create_order(
    base_quantities: Vec<(usize, u32)>,  // (product_id, quantity)
    surge_quantities: Vec<(usize, u32)>, // (product_id, quantity)
) -> MonthlyOrder {
    MonthlyOrder {
        base_orders: base_quantities.into_iter()
            .map(|(id, qty)| ProductOrder { product_id: id, quantity: qty })
            .collect(),
        surge_orders: surge_quantities.into_iter()
            .map(|(id, qty)| ProductOrder { product_id: id, quantity: qty })
            .collect(),
    }
}

/// Calculate remaining capacity after order allocation
#[allow(dead_code)]
pub fn remaining_capacity(order: &MonthlyOrder, pair: &SupplierPair) -> (u32, u32) {
    let base_remaining = pair.base_supplier.fixed_capacity.saturating_sub(order.total_base_quantity());
    let surge_remaining = pair.surge_supplier.fixed_capacity.saturating_sub(order.total_surge_quantity());
    (base_remaining, surge_remaining)
}
