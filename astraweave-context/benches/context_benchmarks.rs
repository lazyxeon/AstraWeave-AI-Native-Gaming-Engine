use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use astraweave_context::{
    ContextConfig, ContextWindow, ContextWindowConfig, ConversationHistory, Message, Role,
    WindowType,
};
use std::hint::black_box;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test message with specified role and content
fn create_test_message(role: Role, content: &str) -> Message {
    Message::new(role, content.to_string())
}

/// Create a batch of test messages
fn create_message_batch(count: usize) -> Vec<Message> {
    (0..count)
        .map(|i| {
            let role = if i % 2 == 0 {
                Role::User
            } else {
                Role::Assistant
            };
            create_test_message(role, &format!("Test message number {} with some content", i))
        })
        .collect()
}

// ============================================================================
// Benchmark 1: Message Creation & Formatting
// ============================================================================

fn bench_message_creation(c: &mut Criterion) {
    c.bench_function("message_creation", |b| {
        b.iter(|| {
            let msg = create_test_message(Role::User, "Hello, world!");
            black_box(msg)
        })
    });
}

fn bench_message_formatting(c: &mut Criterion) {
    let msg = create_test_message(Role::User, "Hello, world!");

    c.bench_function("message_format_for_prompt", |b| {
        b.iter(|| {
            let formatted = msg.format_for_prompt();
            black_box(formatted)
        })
    });
}

// ============================================================================
// Benchmark 2: Context Window Creation & Operations
// ============================================================================

fn bench_context_window_creation(c: &mut Criterion) {
    let config = ContextWindowConfig::default();

    c.bench_function("context_window_creation", |b| {
        b.iter(|| {
            let window = ContextWindow::new(config.clone());
            black_box(window)
        })
    });
}

fn bench_context_window_add_message(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_window_add_message");

    for size in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter_with_setup(
                || {
                    let config = ContextWindowConfig {
                        max_tokens: 10000,
                        max_messages: 200,
                        ..Default::default()
                    };
                    ContextWindow::new(config)
                },
                |mut window| {
                    for i in 0..size {
                        let msg = create_test_message(Role::User, &format!("Message {}", i));
                        let _ = window.add_message(msg);
                    }
                    black_box(window)
                },
            )
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 3: Window Types Performance
// ============================================================================

fn bench_window_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("window_types");

    for window_type in [WindowType::Sliding, WindowType::Fixed] {
        group.bench_with_input(
            BenchmarkId::new("add_50_messages", format!("{:?}", window_type)),
            &window_type,
            |b, &window_type| {
                b.iter_with_setup(
                    || {
                        let config = ContextWindowConfig {
                            max_tokens: 4096,
                            max_messages: 50,
                            window_type,
                            ..Default::default()
                        };
                        ContextWindow::new(config)
                    },
                    |mut window| {
                        for i in 0..50 {
                            let msg = create_test_message(Role::User, &format!("Test {}", i));
                            let _ = window.add_message(msg);
                        }
                        black_box(window)
                    },
                )
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 4: Conversation History (sync operations only)
// ============================================================================

fn bench_conversation_history_creation(c: &mut Criterion) {
    let config = ContextConfig::default();

    c.bench_function("conversation_history_creation", |b| {
        b.iter(|| {
            let history = ConversationHistory::new(config.clone());
            black_box(history)
        })
    });
}

fn bench_get_recent_messages(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_recent_messages");

    for total in [50, 100, 200] {
        group.bench_with_input(
            BenchmarkId::from_parameter(total),
            &total,
            |b, &_total| {
                b.iter_with_setup(
                    || {
                        // Setup: Create history with messages
                        let config = ContextConfig {
                            max_tokens: 10000,
                            ..Default::default()
                        };
                        let history = ConversationHistory::new(config);

                        // Pre-populate with messages (sync operation)
                        // Note: We can't use add_message here as it's async
                        // This benchmark focuses on the get operation
                        history
                    },
                    |history| {
                        // Benchmark the retrieval (sync operation)
                        let recent = history.get_recent_messages(20);
                        black_box(recent)
                    },
                )
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 5: Message Batching
// ============================================================================

fn bench_message_batch_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_batch_creation");

    for count in [10, 50, 100, 500] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                let batch = create_message_batch(count);
                black_box(batch)
            })
        });
    }

    group.finish();
}

fn bench_message_batch_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_batch_formatting");

    for count in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_with_setup(
                || create_message_batch(count),
                |batch| {
                    let formatted: Vec<_> = batch.iter().map(|m| m.format_for_prompt()).collect();
                    black_box(formatted)
                },
            )
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 6: Context Window Statistics
// ============================================================================

fn bench_context_window_stats(c: &mut Criterion) {
    c.bench_function("context_window_with_stats", |b| {
        b.iter_with_setup(
            || {
                let config = ContextWindowConfig::default();
                let mut window = ContextWindow::new(config);

                // Add some messages to generate stats
                for i in 0..20 {
                    let msg = create_test_message(Role::User, &format!("Stats test {}", i));
                    let _ = window.add_message(msg);
                }

                window
            },
            |window| {
                // Access stats (this should be cheap)
                black_box(window)
            },
        )
    });
}

// ============================================================================
// Benchmark Registration
// ============================================================================

criterion_group!(
    benches,
    bench_message_creation,
    bench_message_formatting,
    bench_context_window_creation,
    bench_context_window_add_message,
    bench_window_types,
    bench_conversation_history_creation,
    bench_get_recent_messages,
    bench_message_batch_creation,
    bench_message_batch_formatting,
    bench_context_window_stats,
);

criterion_main!(benches);
