use std::clone::Clone;

/// Supplier data structure representing a supplier in the supply chain
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Supplier {
    pub id: usize,
    pub name: String,
    pub fixed_capacity: u32,
    pub lead_time_months: usize,
    pub unit_cost: f64,
    pub setup_cost: f64,
}

/// Pair of suppliers: one for base orders, one for surge orders
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SupplierPair {
    pub base_supplier: Supplier,      // Longer lead time, planned orders
    pub surge_supplier: Supplier,     // Shorter lead time, emergency orders
}

/// Simulation parameters for configuring demand, costs, and pricing
#[allow(dead_code)]
#[derive(Clone)]
pub struct SimulationParams {
    /// Expected demand (used for decision-making)
    pub mean_demand: f64,
    pub std_dev_demand: f64,
    /// Actual demand (realized during simulation - may differ from expected)
    pub actual_mean_demand: f64,
    pub actual_std_dev_demand: f64,
    pub selling_price: f64,
    pub monthly_holding_cost: f64,
    pub liquidation_price: f64,
    pub order_change_fee: f64,
}

/// Monthly order decision between base and surge suppliers
#[derive(Clone, Debug)]
pub struct MonthlyOrder {
    pub base_quantity: u32,
    pub surge_quantity: u32,
}

/// Result of a single month's simulation
#[allow(dead_code)]
#[derive(Debug)]
pub struct MonthlyResult {
    pub month: String,
    pub inventory_start: u32,
    pub incoming: u32,
    pub inventory_after_incoming: u32,
    pub demand: u32,
    pub units_sold: u32,
    pub inventory_end: u32,
    pub revenue: f64,
    pub production_cost: f64,
    pub holding_cost: f64,
    pub liquidation_revenue: f64,
    pub order_change_cost: f64,
    pub monthly_profit: f64,
}

/// Complete simulation result for a supplier combination
#[allow(dead_code)]
#[derive(Debug)]
pub struct SimulationResult {
    pub base_supplier: String,
    pub surge_supplier: String,
    pub optimal_quantity: u32,
    pub monthly_results: Vec<MonthlyResult>,
    pub total_profit: f64,
}

/// Monte Carlo statistics for a supplier combination
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MonteCarloStats {
    pub base_supplier: String,
    pub base_supplier_lead_time: usize,
    pub surge_supplier: String,
    pub surge_supplier_lead_time: usize,
    pub optimal_quantity: u32,
    pub num_simulations: usize,
    pub mean_profit: f64,
    pub std_dev_profit: f64,
    pub min_profit: f64,
    pub max_profit: f64,
    pub percentile_10: f64,
    pub percentile_25: f64,
    pub percentile_50: f64,  // Median
    pub percentile_75: f64,
    pub percentile_90: f64,
}

/// Option valuation state for binomial tree
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct OptionState {
    pub month: usize,
    pub inventory: u32,
    pub cumulative_uplifts: i32, // Net up/down movements in binomial tree
}
