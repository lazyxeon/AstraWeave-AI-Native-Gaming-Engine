#![allow(dead_code)]

//! Adversarial Observability Benchmarks
//!
//! Stress testing for tracing, metrics collection, and crash reporting.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;
use std::time::{Duration, Instant};

// ============================================================================
// LOCAL TYPES (Mirror astraweave-observability API)
// ============================================================================

#[derive(Clone, Debug)]
struct Span {
    name: String,
    start: Instant,
    end: Option<Instant>,
    parent_id: Option<u64>,
    span_id: u64,
    attributes: HashMap<String, AttributeValue>,
}

impl Span {
    fn new(name: &str, span_id: u64, parent_id: Option<u64>) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            end: None,
            parent_id,
            span_id,
            attributes: HashMap::new(),
        }
    }
    
    fn end(&mut self) {
        self.end = Some(Instant::now());
    }
    
    fn duration(&self) -> Option<Duration> {
        self.end.map(|e| e.duration_since(self.start))
    }
    
    fn set_attribute(&mut self, key: &str, value: AttributeValue) {
        self.attributes.insert(key.to_string(), value);
    }
}

#[derive(Clone, Debug)]
enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Clone, Debug)]
struct Metric {
    name: String,
    metric_type: MetricType,
    value: f64,
    tags: HashMap<String, String>,
    timestamp: u64,
}

#[derive(Clone, Copy, Debug)]
enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Clone, Debug)]
struct HistogramBucket {
    le: f64, // Less than or equal
    count: u64,
}

#[derive(Clone, Debug)]
struct Histogram {
    name: String,
    buckets: Vec<HistogramBucket>,
    sum: f64,
    count: u64,
}

impl Histogram {
    fn new(name: &str, boundaries: &[f64]) -> Self {
        Self {
            name: name.to_string(),
            buckets: boundaries.iter().map(|&le| HistogramBucket { le, count: 0 }).collect(),
            sum: 0.0,
            count: 0,
        }
    }
    
    fn observe(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
        
        for bucket in &mut self.buckets {
            if value <= bucket.le {
                bucket.count += 1;
            }
        }
    }
}

#[derive(Clone, Debug)]
struct CrashReport {
    error_type: String,
    message: String,
    backtrace: Vec<StackFrame>,
    context: HashMap<String, String>,
    timestamp: u64,
}

#[derive(Clone, Debug)]
struct StackFrame {
    function: String,
    file: Option<String>,
    line: Option<u32>,
    module: String,
}

#[derive(Clone, Debug)]
struct LogEntry {
    level: LogLevel,
    message: String,
    target: String,
    timestamp: u64,
    fields: HashMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Ord, PartialOrd, Eq)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// ============================================================================
// CATEGORY 1: SPAN OPERATIONS
// ============================================================================

fn bench_span_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("observability_adversarial/span_operations");
    
    // Test 1: Span creation
    group.bench_function("span_creation_10000", |bencher| {
        let mut span_id = 0u64;
        
        bencher.iter(|| {
            let spans: Vec<Span> = (0..10000)
                .map(|i| {
                    span_id += 1;
                    let parent = if i > 0 && i % 10 != 0 {
                        Some(span_id - 1)
                    } else {
                        None
                    };
                    Span::new(&format!("span_{}", i), span_id, parent)
                })
                .collect();
            
            std_black_box(spans.len())
        });
    });
    
    // Test 2: Span lifecycle
    group.bench_function("span_lifecycle_5000", |bencher| {
        bencher.iter(|| {
            let mut spans: Vec<Span> = Vec::with_capacity(5000);
            
            for i in 0..5000 {
                let mut span = Span::new(&format!("op_{}", i % 100), i as u64, None);
                span.set_attribute("iteration", AttributeValue::Int(i as i64));
                span.set_attribute("type", AttributeValue::String("benchmark".to_string()));
                
                // Simulate some work
                let _ = (0..10).sum::<u32>();
                
                span.end();
                spans.push(span);
            }
            
            let total_duration: Duration = spans
                .iter()
                .filter_map(|s| s.duration())
                .sum();
            
            std_black_box(total_duration.as_nanos())
        });
    });
    
    // Test 3: Nested span tree
    group.bench_function("nested_span_tree_depth_20", |bencher| {
        bencher.iter(|| {
            let mut spans: Vec<Span> = Vec::new();
            let mut current_id = 0u64;
            
            // Create 100 trees with depth 20
            for tree in 0..100 {
                let mut parent_id: Option<u64> = None;
                
                for depth in 0..20 {
                    current_id += 1;
                    let mut span = Span::new(
                        &format!("tree_{}_depth_{}", tree, depth),
                        current_id,
                        parent_id,
                    );
                    span.set_attribute("depth", AttributeValue::Int(depth as i64));
                    span.end();
                    
                    parent_id = Some(current_id);
                    spans.push(span);
                }
            }
            
            std_black_box(spans.len())
        });
    });
    
    // Test 4: Span attribute operations
    group.bench_function("span_attributes_20000", |bencher| {
        let mut span = Span::new("test_span", 1, None);
        
        bencher.iter(|| {
            for i in 0..20000 {
                let key = format!("attr_{}", i % 100);
                let value = match i % 4 {
                    0 => AttributeValue::String(format!("value_{}", i)),
                    1 => AttributeValue::Int(i as i64),
                    2 => AttributeValue::Float(i as f64 * 0.1),
                    _ => AttributeValue::Bool(i % 2 == 0),
                };
                span.set_attribute(&key, value);
            }
            
            std_black_box(span.attributes.len())
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 2: METRICS COLLECTION
// ============================================================================

fn bench_metrics_collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("observability_adversarial/metrics_collection");
    
    // Test 1: Counter increments
    group.bench_function("counter_increments_100000", |bencher| {
        let mut counters: HashMap<String, u64> = HashMap::new();
        
        for i in 0..100 {
            counters.insert(format!("counter_{}", i), 0);
        }
        
        bencher.iter(|| {
            for i in 0..100000 {
                let key = format!("counter_{}", i % 100);
                if let Some(count) = counters.get_mut(&key) {
                    *count += 1;
                }
            }
            
            let total: u64 = counters.values().sum();
            std_black_box(total)
        });
    });
    
    // Test 2: Gauge updates
    group.bench_function("gauge_updates_50000", |bencher| {
        let mut gauges: HashMap<String, f64> = HashMap::new();
        
        for i in 0..50 {
            gauges.insert(format!("gauge_{}", i), 0.0);
        }
        
        let updates: Vec<(String, f64)> = (0..50000)
            .map(|i| (format!("gauge_{}", i % 50), (i % 1000) as f64))
            .collect();
        
        bencher.iter(|| {
            for (key, value) in &updates {
                if let Some(gauge) = gauges.get_mut(key) {
                    *gauge = *value;
                }
            }
            
            let sum: f64 = gauges.values().sum();
            std_black_box(sum)
        });
    });
    
    // Test 3: Histogram observations
    group.bench_function("histogram_observations_10000", |bencher| {
        let boundaries = vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];
        let mut histogram = Histogram::new("request_duration", &boundaries);
        
        let observations: Vec<f64> = (0..10000)
            .map(|i| {
                // Simulate response times
                let base = (i % 100) as f64 * 0.01;
                base + (i % 10) as f64 * 0.001
            })
            .collect();
        
        bencher.iter(|| {
            for &value in &observations {
                histogram.observe(value);
            }
            
            std_black_box((histogram.count, histogram.sum))
        });
    });
    
    // Test 4: Metric tagging
    group.bench_function("metric_tagging_10000", |bencher| {
        bencher.iter(|| {
            let metrics: Vec<Metric> = (0..10000)
                .map(|i| {
                    let mut tags = HashMap::new();
                    tags.insert("service".to_string(), format!("svc_{}", i % 10));
                    tags.insert("region".to_string(), format!("region_{}", i % 5));
                    tags.insert("instance".to_string(), format!("inst_{}", i % 100));
                    
                    Metric {
                        name: format!("metric_{}", i % 50),
                        metric_type: match i % 4 {
                            0 => MetricType::Counter,
                            1 => MetricType::Gauge,
                            2 => MetricType::Histogram,
                            _ => MetricType::Summary,
                        },
                        value: i as f64 * 0.1,
                        tags,
                        timestamp: i as u64,
                    }
                })
                .collect();
            
            std_black_box(metrics.len())
        });
    });
    
    // Test 5: Metric aggregation
    group.bench_function("metric_aggregation_5000", |bencher| {
        let metrics: Vec<Metric> = (0..5000)
            .map(|i| {
                let mut tags = HashMap::new();
                tags.insert("service".to_string(), format!("svc_{}", i % 10));
                
                Metric {
                    name: format!("metric_{}", i % 20),
                    metric_type: MetricType::Gauge,
                    value: i as f64 * 0.1,
                    tags,
                    timestamp: i as u64,
                }
            })
            .collect();
        
        bencher.iter(|| {
            let mut aggregated: HashMap<(String, String), (f64, usize)> = HashMap::new();
            
            for metric in &metrics {
                let service = metric.tags.get("service").cloned().unwrap_or_default();
                let key = (metric.name.clone(), service);
                
                let entry = aggregated.entry(key).or_insert((0.0, 0));
                entry.0 += metric.value;
                entry.1 += 1;
            }
            
            // Calculate averages
            let averages: Vec<f64> = aggregated
                .values()
                .map(|(sum, count)| sum / *count as f64)
                .collect();
            
            std_black_box(averages.len())
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 3: LOGGING
// ============================================================================

fn bench_logging(c: &mut Criterion) {
    let mut group = c.benchmark_group("observability_adversarial/logging");
    
    // Test 1: Log entry creation
    for count in [1000, 5000, 10000] {
        group.throughput(Throughput::Elements(count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("log_entry_creation", count),
            &count,
            |bencher, &count| {
                bencher.iter(|| {
                    let entries: Vec<LogEntry> = (0..count)
                        .map(|i| {
                            let mut fields = HashMap::new();
                            fields.insert("request_id".to_string(), format!("req_{}", i));
                            fields.insert("user_id".to_string(), format!("user_{}", i % 100));
                            
                            LogEntry {
                                level: match i % 5 {
                                    0 => LogLevel::Trace,
                                    1 => LogLevel::Debug,
                                    2 => LogLevel::Info,
                                    3 => LogLevel::Warn,
                                    _ => LogLevel::Error,
                                },
                                message: format!("Processing request {} for operation {}", i, i % 10),
                                target: format!("astraweave::module_{}", i % 20),
                                timestamp: i as u64,
                                fields,
                            }
                        })
                        .collect();
                    
                    std_black_box(entries.len())
                });
            },
        );
    }
    
    // Test 2: Log level filtering
    group.bench_function("log_level_filtering_10000", |bencher| {
        let entries: Vec<LogEntry> = (0..10000)
            .map(|i| LogEntry {
                level: match i % 5 {
                    0 => LogLevel::Trace,
                    1 => LogLevel::Debug,
                    2 => LogLevel::Info,
                    3 => LogLevel::Warn,
                    _ => LogLevel::Error,
                },
                message: format!("Log message {}", i),
                target: "test".to_string(),
                timestamp: i as u64,
                fields: HashMap::new(),
            })
            .collect();
        
        let min_level = LogLevel::Info;
        
        bencher.iter(|| {
            let filtered: Vec<&LogEntry> = entries
                .iter()
                .filter(|e| e.level >= min_level)
                .collect();
            
            std_black_box(filtered.len())
        });
    });
    
    // Test 3: Log formatting
    group.bench_function("log_formatting_5000", |bencher| {
        let entries: Vec<LogEntry> = (0..5000)
            .map(|i| {
                let mut fields = HashMap::new();
                fields.insert("request_id".to_string(), format!("req_{}", i));
                fields.insert("duration_ms".to_string(), format!("{}", i % 1000));
                
                LogEntry {
                    level: LogLevel::Info,
                    message: "Request completed successfully".to_string(),
                    target: "astraweave::http::handler".to_string(),
                    timestamp: 1700000000 + i as u64,
                    fields,
                }
            })
            .collect();
        
        bencher.iter(|| {
            let formatted: Vec<String> = entries
                .iter()
                .map(|e| {
                    let mut output = String::with_capacity(200);
                    output.push_str(&format!("[{}] ", e.timestamp));
                    output.push_str(&format!("{:?} ", e.level));
                    output.push_str(&e.target);
                    output.push_str(": ");
                    output.push_str(&e.message);
                    
                    if !e.fields.is_empty() {
                        output.push_str(" {");
                        for (k, v) in &e.fields {
                            output.push_str(&format!(" {}={}", k, v));
                        }
                        output.push_str(" }");
                    }
                    
                    output
                })
                .collect();
            
            let total_len: usize = formatted.iter().map(|s| s.len()).sum();
            std_black_box(total_len)
        });
    });
    
    // Test 4: Target matching
    group.bench_function("target_matching_10000", |bencher| {
        let entries: Vec<LogEntry> = (0..10000)
            .map(|i| LogEntry {
                level: LogLevel::Debug,
                message: "test".to_string(),
                target: format!(
                    "astraweave::{}::{}",
                    ["ai", "render", "physics", "ecs", "audio"][i % 5],
                    ["system", "component", "resource", "plugin"][i % 4]
                ),
                timestamp: i as u64,
                fields: HashMap::new(),
            })
            .collect();
        
        let patterns = ["astraweave::ai", "astraweave::render", "astraweave::physics"];
        
        bencher.iter(|| {
            let matched: Vec<&LogEntry> = entries
                .iter()
                .filter(|e| patterns.iter().any(|p| e.target.starts_with(p)))
                .collect();
            
            std_black_box(matched.len())
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 4: CRASH REPORTING
// ============================================================================

fn bench_crash_reporting(c: &mut Criterion) {
    let mut group = c.benchmark_group("observability_adversarial/crash_reporting");
    
    // Test 1: Backtrace generation
    group.bench_function("backtrace_generation_100", |bencher| {
        bencher.iter(|| {
            let backtraces: Vec<Vec<StackFrame>> = (0..100)
                .map(|i| {
                    (0..50)
                        .map(|j| StackFrame {
                            function: format!("astraweave::module{}::function{}", i % 10, j),
                            file: Some(format!("src/module{}/file{}.rs", i % 10, j % 5)),
                            line: Some((j * 10 + i) as u32),
                            module: format!("astraweave::module{}", i % 10),
                        })
                        .collect()
                })
                .collect();
            
            let total_frames: usize = backtraces.iter().map(|b| b.len()).sum();
            std_black_box(total_frames)
        });
    });
    
    // Test 2: Crash report creation
    group.bench_function("crash_report_creation_500", |bencher| {
        bencher.iter(|| {
            let reports: Vec<CrashReport> = (0..500)
                .map(|i| {
                    let mut context = HashMap::new();
                    context.insert("version".to_string(), "1.0.0".to_string());
                    context.insert("platform".to_string(), "windows".to_string());
                    context.insert("gpu".to_string(), format!("GPU_{}", i % 10));
                    context.insert("ram_mb".to_string(), format!("{}", 8192 + i * 100));
                    
                    CrashReport {
                        error_type: format!("{}Error", ["Panic", "OutOfMemory", "Gpu", "Io", "Assert"][i % 5]),
                        message: format!("Error occurred during operation {}", i),
                        backtrace: (0..30)
                            .map(|j| StackFrame {
                                function: format!("func_{}", j),
                                file: Some(format!("file_{}.rs", j % 10)),
                                line: Some(j as u32 * 10),
                                module: format!("module_{}", j % 5),
                            })
                            .collect(),
                        context,
                        timestamp: 1700000000 + i as u64,
                    }
                })
                .collect();
            
            std_black_box(reports.len())
        });
    });
    
    // Test 3: Symbolication (simulated)
    group.bench_function("symbolication_1000", |bencher| {
        let addresses: Vec<u64> = (0..1000).map(|i| 0x1000_0000 + i * 0x100).collect();
        
        // Symbol table
        let symbols: HashMap<u64, String> = (0..100)
            .map(|i| {
                let addr = 0x1000_0000 + i * 0x1000;
                let name = format!("astraweave::module{}::function{}", i / 10, i % 10);
                (addr, name)
            })
            .collect();
        
        bencher.iter(|| {
            let resolved: Vec<Option<&String>> = addresses
                .iter()
                .map(|addr| {
                    // Find closest symbol (simple linear search)
                    symbols
                        .iter()
                        .filter(|(&sym_addr, _)| *addr >= sym_addr && *addr < sym_addr + 0x1000)
                        .map(|(_, name)| name)
                        .next()
                })
                .collect();
            
            let found = resolved.iter().filter(|r| r.is_some()).count();
            std_black_box(found)
        });
    });
    
    // Test 4: Report serialization
    group.bench_function("report_serialization_200", |bencher| {
        let reports: Vec<CrashReport> = (0..200)
            .map(|i| {
                let mut context = HashMap::new();
                context.insert("version".to_string(), "1.0.0".to_string());
                
                CrashReport {
                    error_type: "PanicError".to_string(),
                    message: format!("Panic at {}", i),
                    backtrace: (0..20)
                        .map(|j| StackFrame {
                            function: format!("func_{}", j),
                            file: Some("file.rs".to_string()),
                            line: Some(j as u32),
                            module: "test".to_string(),
                        })
                        .collect(),
                    context,
                    timestamp: i as u64,
                }
            })
            .collect();
        
        bencher.iter(|| {
            let serialized: Vec<String> = reports
                .iter()
                .map(|r| {
                    let mut json = String::with_capacity(2000);
                    json.push_str("{\"error_type\":\"");
                    json.push_str(&r.error_type);
                    json.push_str("\",\"message\":\"");
                    json.push_str(&r.message);
                    json.push_str("\",\"timestamp\":");
                    json.push_str(&r.timestamp.to_string());
                    json.push_str(",\"backtrace\":[");
                    
                    for (i, frame) in r.backtrace.iter().enumerate() {
                        if i > 0 {
                            json.push(',');
                        }
                        json.push_str("{\"function\":\"");
                        json.push_str(&frame.function);
                        json.push_str("\"}");
                    }
                    
                    json.push_str("]}");
                    json
                })
                .collect();
            
            let total_len: usize = serialized.iter().map(|s| s.len()).sum();
            std_black_box(total_len)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 5: TRACE CONTEXT PROPAGATION
// ============================================================================

fn bench_trace_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("observability_adversarial/trace_context");
    
    // Test 1: Context creation
    group.bench_function("context_creation_10000", |bencher| {
        bencher.iter(|| {
            let contexts: Vec<(u128, u64, u8)> = (0..10000)
                .map(|i| {
                    let trace_id = (i as u128) << 64 | (i as u128);
                    let span_id = i as u64;
                    let flags = (i % 2) as u8;
                    (trace_id, span_id, flags)
                })
                .collect();
            
            std_black_box(contexts.len())
        });
    });
    
    // Test 2: Header encoding
    group.bench_function("header_encoding_5000", |bencher| {
        let contexts: Vec<(u128, u64)> = (0..5000)
            .map(|i| ((i as u128) << 64 | (i as u128 + 1), i as u64))
            .collect();
        
        bencher.iter(|| {
            let headers: Vec<String> = contexts
                .iter()
                .map(|(trace_id, span_id)| {
                    format!("00-{:032x}-{:016x}-01", trace_id, span_id)
                })
                .collect();
            
            let total_len: usize = headers.iter().map(|h| h.len()).sum();
            std_black_box(total_len)
        });
    });
    
    // Test 3: Header parsing
    group.bench_function("header_parsing_5000", |bencher| {
        let headers: Vec<String> = (0..5000)
            .map(|i| {
                let trace_id = (i as u128) << 64 | (i as u128 + 1);
                let span_id = i as u64;
                format!("00-{:032x}-{:016x}-01", trace_id, span_id)
            })
            .collect();
        
        bencher.iter(|| {
            let parsed: Vec<Option<(u128, u64, u8)>> = headers
                .iter()
                .map(|h| {
                    let parts: Vec<&str> = h.split('-').collect();
                    if parts.len() != 4 {
                        return None;
                    }
                    
                    let trace_id = u128::from_str_radix(parts[1], 16).ok()?;
                    let span_id = u64::from_str_radix(parts[2], 16).ok()?;
                    let flags = u8::from_str_radix(parts[3], 16).ok()?;
                    
                    Some((trace_id, span_id, flags))
                })
                .collect();
            
            let valid = parsed.iter().filter(|p| p.is_some()).count();
            std_black_box(valid)
        });
    });
    
    // Test 4: Baggage propagation
    group.bench_function("baggage_propagation_2000", |bencher| {
        let baggage: Vec<HashMap<String, String>> = (0..2000)
            .map(|i| {
                let mut b = HashMap::new();
                b.insert("user_id".to_string(), format!("user_{}", i % 100));
                b.insert("session_id".to_string(), format!("sess_{}", i));
                b.insert("request_id".to_string(), format!("req_{}", i));
                b.insert("region".to_string(), format!("region_{}", i % 5));
                b
            })
            .collect();
        
        bencher.iter(|| {
            let encoded: Vec<String> = baggage
                .iter()
                .map(|b| {
                    b.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .collect();
            
            let total_len: usize = encoded.iter().map(|s| s.len()).sum();
            std_black_box(total_len)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 6: PERFORMANCE MONITORING
// ============================================================================

fn bench_performance_monitoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("observability_adversarial/performance_monitoring");
    
    // Test 1: FPS calculation
    group.bench_function("fps_calculation_600", |bencher| {
        let frame_times: Vec<f64> = (0..600)
            .map(|i| {
                // Simulate varying frame times (16.67ms ± variation)
                16.67 + (i % 10) as f64 * 0.5 - 2.5
            })
            .collect();
        
        bencher.iter(|| {
            // Calculate rolling average FPS
            let window_size = 60;
            let fps_values: Vec<f64> = frame_times
                .windows(window_size)
                .map(|window| {
                    let avg_frame_time: f64 = window.iter().sum::<f64>() / window.len() as f64;
                    1000.0 / avg_frame_time
                })
                .collect();
            
            let avg_fps: f64 = fps_values.iter().sum::<f64>() / fps_values.len() as f64;
            std_black_box(avg_fps)
        });
    });
    
    // Test 2: Memory tracking
    group.bench_function("memory_tracking_1000", |bencher| {
        let allocations: Vec<(u64, usize, &str)> = (0..1000)
            .map(|i| {
                let ptr = 0x1000_0000 + (i * 0x1000) as u64;
                let size = 64 + (i % 100) * 16;
                let category = ["texture", "mesh", "audio", "ai", "physics"][i % 5];
                (ptr, size, category)
            })
            .collect();
        
        bencher.iter(|| {
            let mut by_category: HashMap<&str, (usize, usize)> = HashMap::new();
            
            for (_, size, category) in &allocations {
                let entry = by_category.entry(category).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += size;
            }
            
            let total_size: usize = by_category.values().map(|(_, s)| s).sum();
            std_black_box(total_size)
        });
    });
    
    // Test 3: CPU profiling simulation
    group.bench_function("cpu_profile_aggregation_5000", |bencher| {
        let samples: Vec<(String, u64)> = (0..5000)
            .map(|i| {
                let function = format!(
                    "astraweave::{}::{}",
                    ["ai", "render", "physics", "ecs", "audio"][i % 5],
                    ["update", "tick", "process", "compute", "run"][i % 5]
                );
                let cycles = 1000 + (i % 500) as u64;
                (function, cycles)
            })
            .collect();
        
        bencher.iter(|| {
            let mut profile: HashMap<String, (u64, usize)> = HashMap::new();
            
            for (func, cycles) in &samples {
                let entry = profile.entry(func.clone()).or_insert((0, 0));
                entry.0 += cycles;
                entry.1 += 1;
            }
            
            // Sort by total cycles
            let mut sorted: Vec<_> = profile.into_iter().collect();
            sorted.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
            
            std_black_box(sorted.len())
        });
    });
    
    // Test 4: GPU timing
    group.bench_function("gpu_timing_aggregation_1000", |bencher| {
        let timings: Vec<(&str, f64, f64)> = (0..1000)
            .map(|i| {
                let pass = ["shadow", "gbuffer", "lighting", "post", "ui"][i % 5];
                let start = i as f64 * 0.1;
                let duration = 0.5 + (i % 10) as f64 * 0.1;
                (pass, start, duration)
            })
            .collect();
        
        bencher.iter(|| {
            let mut by_pass: HashMap<&str, Vec<f64>> = HashMap::new();
            
            for (pass, _, duration) in &timings {
                by_pass.entry(pass).or_default().push(*duration);
            }
            
            let stats: HashMap<&str, (f64, f64, f64)> = by_pass
                .iter()
                .map(|(pass, durations)| {
                    let count = durations.len() as f64;
                    let sum: f64 = durations.iter().sum();
                    let avg = sum / count;
                    let max = durations.iter().cloned().fold(0.0f64, f64::max);
                    (*pass, (avg, max, sum))
                })
                .collect();
            
            std_black_box(stats.len())
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_span_operations,
    bench_metrics_collection,
    bench_logging,
    bench_crash_reporting,
    bench_trace_context,
    bench_performance_monitoring,
);

criterion_main!(benches);
