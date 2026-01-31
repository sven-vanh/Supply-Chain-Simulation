use crate::models::{Supplier, SupplierPair};

/// Generate intelligent supplier pairs based on lead times.
/// Pairs longer lead-time suppliers with shorter lead-time suppliers.
/// Only suppliers with lead_time >= 1 can be used as base suppliers.
/// Only suppliers with lead_time < 2 can be used as surge suppliers.
pub fn generate_intelligent_pairs(suppliers: &[Supplier]) -> Vec<SupplierPair> {
    let mut pairs = Vec::new();

    // Identify eligible base suppliers (lead_time >= 1) and surge suppliers (lead_time < 2)
    let base_eligible: Vec<&Supplier> = suppliers.iter()
        .filter(|s| s.lead_time_months >= 1)
        .collect();

    let surge_eligible: Vec<&Supplier> = suppliers.iter()
        .filter(|s| s.lead_time_months < 2)
        .collect();

    // Create pairs: each base-eligible supplier with each surge-eligible supplier
    for base in &base_eligible {
        for surge in &surge_eligible {
            if base.id != surge.id {
                pairs.push(SupplierPair {
                    base_supplier: (*base).clone(),
                    surge_supplier: (*surge).clone(),
                });
            }
        }
    }

    pairs
}
