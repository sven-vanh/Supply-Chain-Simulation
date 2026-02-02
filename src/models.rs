use std::clone::Clone;
use std::collections::HashMap;

/// Product data structure representing a product in the supply chain
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Product {
    pub id: usize,
    pub name: String,
    pub selling_price: f64,
    pub liquidation_price: f64,
    pub monthly_holding_cost: f64,
}

/// Demand parameters for a specific product
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ProductDemandParams {
    pub product_id: usize,
    pub mean_demand: f64,
    pub std_dev_demand: f64,
    pub actual_mean_demand: f64,
    pub actual_std_dev_demand: f64,
}

/// Supplier data structure representing a supplier in the supply chain
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Supplier {
    pub id: usize,
    pub name: String,
    pub fixed_capacity: u32,
    pub lead_time_months: usize,
    /// Unit cost per product (product_id -> cost)
    pub unit_costs: HashMap<usize, f64>,
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
    /// Products in the simulation
    pub products: Vec<Product>,
    /// Demand parameters per product
    pub demand_params: Vec<ProductDemandParams>,
    /// Order change fee (paid once per exercise, covers all products)
    pub order_change_fee: f64,
}

impl SimulationParams {
    /// Get demand params for a specific product
    pub fn get_demand_params(&self, product_id: usize) -> Option<&ProductDemandParams> {
        self.demand_params.iter().find(|p| p.product_id == product_id)
    }
    
    /// Get product by ID
    pub fn get_product(&self, product_id: usize) -> Option<&Product> {
        self.products.iter().find(|p| p.id == product_id)
    }
}

/// Order quantity for a specific product
#[derive(Clone, Debug)]
pub struct ProductOrder {
    pub product_id: usize,
    pub quantity: u32,
}

/// Monthly order decision between base and surge suppliers for all products
#[derive(Clone, Debug)]
pub struct MonthlyOrder {
    pub base_orders: Vec<ProductOrder>,
    pub surge_orders: Vec<ProductOrder>,
}

impl MonthlyOrder {
    /// Get total base quantity across all products
    pub fn total_base_quantity(&self) -> u32 {
        self.base_orders.iter().map(|o| o.quantity).sum()
    }
    
    /// Get total surge quantity across all products
    pub fn total_surge_quantity(&self) -> u32 {
        self.surge_orders.iter().map(|o| o.quantity).sum()
    }
    
    /// Get base quantity for a specific product
    pub fn base_quantity_for(&self, product_id: usize) -> u32 {
        self.base_orders.iter()
            .find(|o| o.product_id == product_id)
            .map(|o| o.quantity)
            .unwrap_or(0)
    }
    
    /// Get surge quantity for a specific product
    pub fn surge_quantity_for(&self, product_id: usize) -> u32 {
        self.surge_orders.iter()
            .find(|o| o.product_id == product_id)
            .map(|o| o.quantity)
            .unwrap_or(0)
    }
}

/// Result for a single product in a month
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProductMonthlyResult {
    pub product_id: usize,
    pub product_name: String,
    pub inventory_start: u32,
    pub incoming: u32,
    pub demand: u32,
    pub units_sold: u32,
    pub inventory_end: u32,
    pub revenue: f64,
    pub production_cost: f64,
    pub holding_cost: f64,
    pub liquidation_revenue: f64,
}

/// Result of a single month's simulation (aggregated across products)
#[allow(dead_code)]
#[derive(Debug)]
pub struct MonthlyResult {
    pub month: String,
    pub product_results: Vec<ProductMonthlyResult>,
    pub order_change_cost: f64,
    pub setup_cost: f64,
    pub monthly_profit: f64,
}

/// Complete simulation result for a supplier combination
#[allow(dead_code)]
#[derive(Debug)]
pub struct SimulationResult {
    pub base_supplier: String,
    pub surge_supplier: String,
    pub product_allocations: Vec<ProductAllocation>,
    pub monthly_results: Vec<MonthlyResult>,
    pub total_profit: f64,
}

/// Allocation of quantity for a specific product
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProductAllocation {
    pub product_id: usize,
    pub product_name: String,
    pub base_quantity: u32,
    pub surge_quantity: u32,
}

/// Monte Carlo statistics for a supplier combination
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MonteCarloStats {
    pub base_supplier: String,
    pub base_supplier_lead_time: usize,
    pub surge_supplier: String,
    pub surge_supplier_lead_time: usize,
    pub product_allocations: Vec<ProductAllocation>,
    pub total_capacity_used: u32,
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
