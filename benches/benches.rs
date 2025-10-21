use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, criterion_group, criterion_main,
};
use std::hint::black_box;
use top_few::Top16;
use topset::TopSet;

/// Generate random data with seeded RNG for reproducibility
fn generate_random_data(size: usize, seed: u64) -> Vec<u32> {
    let mut data = Vec::with_capacity(size);
    let mut rng = seed;
    for _ in 0..size {
        // Simple LCG (Linear Congruential Generator)
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let value = ((rng >> 16) % 1_000_000_000) as u32;
        data.push(value);
    }
    data
}

/// Generate worst-case data: sequential values starting at 1
fn generate_worst_case_data(size: usize) -> Vec<u32> {
    (1..=size as u32).collect()
}

fn benchmark_random_data(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("random_data");
    group.sample_size(10);
    group.plot_config(plot_config);

    for size in [10_000, 100_000, 1_000_000].iter() {
        let data = black_box(generate_random_data(*size, 42));

        group.bench_with_input(BenchmarkId::new("top16", size), size, |b, _| {
            b.iter(|| {
                let mut top = Top16::new(0);
                for &value in &data {
                    top.see(black_box(value));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("topset", size), size, |b, _| {
            b.iter(|| {
                let mut top = TopSet::new(16, |a: &u32, b: &u32| b < a);
                for &value in &data {
                    top.insert(black_box(value));
                }
            });
        });
    }

    group.finish();
}

fn benchmark_worst_case(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("worst_case");
    group.sample_size(10);
    group.plot_config(plot_config);

    for size in [10_000, 100_000, 1_000_000].iter() {
        let data = black_box(generate_worst_case_data(*size));

        group.bench_with_input(BenchmarkId::new("top16", size), size, |b, _| {
            b.iter(|| {
                let mut top = Top16::new(0);
                for &value in &data {
                    top.see(black_box(value));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("topset", size), size, |b, _| {
            b.iter(|| {
                let mut top = TopSet::new(16, |a: &u32, b: &u32| b < a);
                for &value in &data {
                    top.insert(black_box(value));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_random_data, benchmark_worst_case);
criterion_main!(benches);
