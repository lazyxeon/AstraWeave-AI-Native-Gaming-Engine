//! Benchmark for LLM evaluation harness performance

use astraweave_llm::MockLlm;
use astraweave_llm_eval::EvaluationSuite;
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("🔥 Benchmarking LLM Evaluation Harness");
    println!("======================================\n");

    let suite = EvaluationSuite::default();
    let client = Arc::new(MockLlm);

    println!("Scenarios: {}", suite.scenarios.len());
    println!("Running warmup...");

    // Warmup run
    let _ = suite.evaluate(client.clone()).await;

    println!("Running benchmark (3 iterations)...\n");

    let mut times = Vec::new();
    for i in 1..=3 {
        let start = Instant::now();
        let results = suite.evaluate(client.clone()).await;
        let elapsed = start.elapsed();

        times.push(elapsed.as_millis());

        println!(
            "Iteration {}: {} scenarios in {:?} ({:.1}% overall score)",
            i,
            results.total_scenarios,
            elapsed,
            results.overall_score * 100.0
        );
    }

    let avg_ms = times.iter().sum::<u128>() / times.len() as u128;
    let min_ms = times.iter().min().unwrap();
    let max_ms = times.iter().max().unwrap();

    println!("\n📊 Benchmark Results:");
    println!("--------------------");
    println!("Average: {}ms", avg_ms);
    println!("Min:     {}ms", min_ms);
    println!("Max:     {}ms", max_ms);
    println!();

    if avg_ms > 30_000 {
        println!("⚠️  Warning: Average time exceeds 30s target");
    } else {
        println!("✅ Performance target met (<30s)");
    }
}
