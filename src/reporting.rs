/// Reporting and output formatting module
/// Handles all console output and result presentation

use crate::models::MonteCarloStats;

/// Display Monte Carlo results for all supplier combinations
pub fn display_all_results(mut results: Vec<MonteCarloStats>) {
    println!("\n╔═══════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    MONTE CARLO RESULTS - ALL COMBINATIONS                        ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════════╝\n");

    // Sort results by mean profit
    results.sort_by(|a, b| b.mean_profit.partial_cmp(&a.mean_profit).unwrap());

    for (rank, result) in results.iter().enumerate() {
        println!(
            "{}. {} ({}mo lead) + {} ({}mo lead) (Qty: {})",
            rank + 1,
            result.base_supplier, result.base_supplier_lead_time,
            result.surge_supplier, result.surge_supplier_lead_time,
            result.optimal_quantity
        );
        println!(
            "   Mean: ${:.2} ± ${:.2} | Median: ${:.2} | Range: [${:.2}, ${:.2}]",
            result.mean_profit,
            result.std_dev_profit,
            result.percentile_50,
            result.min_profit,
            result.max_profit
        );
        println!(
            "   10th-90th Percentile: [${:.2}, ${:.2}]\n",
            result.percentile_10, result.percentile_90
        );
    }
}

/// Display the best supplier combination with detailed statistics
pub fn display_best_result(result: &MonteCarloStats) {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║              BEST SUPPLIER COMBINATION (HIGHEST MEAN PROFIT)                ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝\n");

    println!(
        "Base Supplier: {} ({} month lead time)\nSurge Supplier: {} ({} month lead time)\nOptimal Quantity: {}\n",
        result.base_supplier, result.base_supplier_lead_time,
        result.surge_supplier, result.surge_supplier_lead_time,
        result.optimal_quantity
    );
    println!(
        "Expected Profit: ${:.2} ± ${:.2} (std dev)",
        result.mean_profit, result.std_dev_profit
    );
    println!("\nProfit Distribution:");
    println!("  Minimum:          ${:.2}", result.min_profit);
    println!("  10th Percentile:  ${:.2}", result.percentile_10);
    println!("  25th Percentile:  ${:.2}", result.percentile_25);
    println!("  Median (50th):    ${:.2}", result.percentile_50);
    println!("  75th Percentile:  ${:.2}", result.percentile_75);
    println!("  90th Percentile:  ${:.2}", result.percentile_90);
    println!("  Maximum:          ${:.2}", result.max_profit);
}

/// Display optimization progress message
pub fn display_optimization_start(base_name: &str, surge_name: &str) {
    println!(
        "\n=== Evaluating: {} (base) + {} (surge) ===",
        base_name, surge_name
    );
}

/// Display finding optimal supply level message
pub fn display_finding_optimal() {
    print!("  Finding optimal supply level... ");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

/// Display found optimal quantity
pub fn display_found_quantity(quantity: u32) {
    println!("Found: {} units", quantity);
}

/// Display Monte Carlo results for current combination
pub fn display_combination_results(mean_profit: f64, std_dev: f64, min_profit: f64, max_profit: f64) {
    println!(
        "Mean Profit: ${:.2} ± ${:.2} | Min: ${:.2} | Max: ${:.2}",
        mean_profit, std_dev, min_profit, max_profit
    );
}
