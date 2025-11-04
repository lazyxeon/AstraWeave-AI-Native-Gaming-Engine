//! Evaluation binary - Run LLM evaluation suite from command line

use astraweave_llm::MockLlm;
use astraweave_llm_eval::EvaluationSuite;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let json_output = std::env::args().any(|arg| arg == "--json");

    println!("🧪 AstraWeave LLM Evaluation Harness");
    println!("====================================\n");

    // Create suite with default scenarios
    let suite = EvaluationSuite::default();

    println!("Running {} scenarios...\n", suite.scenarios.len());

    // Use MockLlm for testing
    let client = Arc::new(MockLlm);

    // Run evaluation
    let results = suite.evaluate(client).await;

    if json_output {
        // JSON output for CI
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        // Human-readable output
        println!("📊 Results Summary");
        println!("------------------");
        println!("Total scenarios: {}", results.total_scenarios);
        println!("Passed: {} ✅", results.passed);
        println!("Failed: {} ❌", results.failed);
        println!();
        println!("Average Scores:");
        println!("  Validity:         {:.1}%", results.avg_validity * 100.0);
        println!(
            "  Goal Achievement: {:.1}%",
            results.avg_goal_achievement * 100.0
        );
        println!("  Safety:           {:.1}%", results.avg_safety * 100.0);
        println!("  Coherence:        {:.1}%", results.avg_coherence * 100.0);
        println!();
        println!("🎯 Overall Score: {:.1}%", results.overall_score * 100.0);
        println!("⏱️  Total time: {}ms", results.total_elapsed_ms);
        println!();

        // Per-type breakdown
        println!("📈 Breakdown by Scenario Type:");
        println!("------------------------------");
        for (scenario_type, stats) in &results.results_by_type {
            println!(
                "{:?}: {} scenarios, {:.1}% validity, {:.1}% goal, {:.1}% overall",
                scenario_type,
                stats.count,
                stats.avg_validity * 100.0,
                stats.avg_goal * 100.0,
                stats.avg_overall * 100.0
            );
        }
        println!();

        // Individual results
        println!("📝 Individual Scenario Results:");
        println!("--------------------------------");
        for result in &results.scenario_results {
            let pass_fail = if result.overall_score >= suite.passing_threshold {
                "✅ PASS"
            } else {
                "❌ FAIL"
            };
            println!(
                "{} {} - {:.1}% overall ({}ms)",
                pass_fail,
                result.scenario_id,
                result.overall_score * 100.0,
                result.elapsed_ms
            );

            if !result.errors.is_empty() {
                for error in &result.errors {
                    println!("    ⚠️  {}", error);
                }
            }
        }
    }

    if results.overall_score < 0.95 {
        println!("\n⚠️  Warning: Overall score below 95% target");
        std::process::exit(1);
    }

    Ok(())
}
