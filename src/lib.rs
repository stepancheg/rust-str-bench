mod aligned_writer;

use std::hint;
use std::time::Instant;

use crate::aligned_writer::AlignedWriter;

pub struct Benchmark<'a> {
    name: String,
    run: Box<dyn Fn() + 'a>,
}

impl<'a> Benchmark<'a> {
    pub fn new<R>(name: &str, f: impl Fn() -> R + 'a) -> Benchmark<'a> {
        Benchmark {
            name: name.to_owned(),
            run: Box::new(move || {
                hint::black_box(f());
            }),
        }
    }
}

fn batch_size(benchmarks: &[Benchmark]) -> usize {
    let mut count = 1;
    loop {
        let start = Instant::now();
        for b in benchmarks {
            for _ in 0..count {
                (b.run)();
            }
        }
        let duration = start.elapsed();
        if duration.as_millis() > 500 {
            return count;
        }
        count *= 2;
    }
}

#[derive(Clone, Default)]
struct Stats {
    value_nanos: Vec<u64>,
}

impl Stats {
    fn sum_nanos(&self) -> u64 {
        self.value_nanos.iter().sum()
    }

    fn sum_seconds(&self) -> f64 {
        self.sum_nanos() as f64 / 1_000_000_000.0
    }

    fn mean_seconds(&self) -> f64 {
        self.sum_seconds() / self.value_nanos.len() as f64
    }

    fn mean_nanos(&self) -> f64 {
        (self.sum_nanos() as f64) / (self.value_nanos.len() as f64)
    }

    fn std_seconds(&self) -> f64 {
        assert!(self.value_nanos.len() > 1);
        let mean_seconds = self.mean_seconds();
        let mut sum = 0.0;
        for &v in &self.value_nanos {
            let diff = (v as f64 / 1_000_000_000.0) - mean_seconds;
            sum += diff * diff;
        }
        (sum / (self.value_nanos.len() - 1) as f64).sqrt()
    }

    fn std_nanos(&self) -> f64 {
        self.std_seconds() * 1_000_000_000.0
    }

    fn standard_error_seconds(&self) -> f64 {
        self.std_seconds() / (self.value_nanos.len() as f64).sqrt()
    }
}

pub fn benchmark(iterations_in_benchmark: usize, benchmarks: &[Benchmark]) -> Vec<f64> {
    println!("Calculating batch size...");
    let batch_size = batch_size(benchmarks);
    println!("batch_size: {}", batch_size);
    let mut stats = vec![Stats::default(); benchmarks.len()];
    let mut n = 0;
    loop {
        for (i, b) in benchmarks.iter().enumerate() {
            let start = Instant::now();
            for _ in 0..batch_size {
                (b.run)();
            }
            let duration = start.elapsed();
            stats[i].value_nanos.push(duration.as_nanos() as u64);
        }
        n += 1;
        if n >= 2 {
            let max_se_mean = stats
                .iter()
                .map(|s| s.standard_error_seconds() / s.mean_seconds())
                .max_by_key(|&s| (s * 1_000_000_000.0) as i64)
                .unwrap();
            let mut w = AlignedWriter::new(benchmarks.len());
            w.write_n_l(benchmarks.iter().map(|b| format!("{}:", b.name.as_str())));
            w.write(" avg=");
            w.write_n_r(stats.iter().map(|s| {
                format!(
                    "{:.3}ns",
                    s.mean_nanos() / (batch_size * iterations_in_benchmark) as f64
                )
            }));
            w.write(" std=");
            w.write_n_r(stats.iter().map(|s| {
                format!(
                    "{:.3}ns",
                    s.std_nanos() / (batch_size * iterations_in_benchmark) as f64
                )
            }));
            println!("N={}", n);
            w.print();
            println!(
                "se/mean={:.3}, stop at 0.01 or after 10 iterations, whichever is later",
                max_se_mean
            );
            if n >= 10 && max_se_mean < 0.01 {
                return stats
                    .into_iter()
                    .map(|s| s.mean_seconds() / (batch_size as f64))
                    .collect();
            }
        }
    }
}
