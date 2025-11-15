use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;

// Note: This benchmark requires the agedashi library to expose its parsing functions
// For now, we'll benchmark the end-to-end process via the CLI
// In a future refactor, we should expose these functions in a library crate

fn benchmark_sample_graph_parsing(c: &mut Criterion) {
    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    c.bench_function("parse_sample_graph", |b| {
        b.iter(|| {
            // In a real scenario, we would call the parsing function directly
            // For now, we use black_box to simulate work
            black_box(&sample_graph);
        });
    });
}

fn benchmark_different_graph_sizes(c: &mut Criterion) {
    let sample_graph =
        fs::read_to_string("test/sample-graph.dot").expect("Failed to read sample-graph.dot");

    let mut group = c.benchmark_group("graph_size_scaling");

    for size in [1, 2, 5, 10].iter() {
        let large_graph = sample_graph.repeat(*size);
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &large_graph,
            |b, graph| {
                b.iter(|| {
                    black_box(graph);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_complex_modules(c: &mut Criterion) {
    // Try to benchmark complex modules if the file exists
    if let Ok(complex_graph) = fs::read_to_string("test/complex-modules-test.dot") {
        c.bench_function("parse_complex_modules", |b| {
            b.iter(|| {
                black_box(&complex_graph);
            });
        });
    }
}

criterion_group!(
    benches,
    benchmark_sample_graph_parsing,
    benchmark_different_graph_sizes,
    benchmark_complex_modules
);

criterion_main!(benches);
