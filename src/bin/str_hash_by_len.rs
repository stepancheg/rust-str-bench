use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::hint;
use std::mem;
use std::time::Instant;

use fnv::FnvHasher;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use twox_hash::xxh3::Hash64;
use twox_hash::XxHash64;

fn random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        s.push(rng.gen_range(b'a'..=b'z') as char);
    }
    s
}

/// To measure overhead of time measurement.
#[derive(Default)]
struct NopHasher;

impl Hasher for NopHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, _bytes: &[u8]) {}
}

#[derive(Default)]
struct Totals {
    count: usize,
    duration_ns: u64,
}

impl Totals {
    fn avg_ns(&self) -> u64 {
        self.duration_ns / self.count as u64
    }

    fn update(&mut self, duration_ns: u64, measure_overhead_ns: u64) {
        let duration_ns = duration_ns.saturating_sub(measure_overhead_ns);
        self.count = self.count.checked_add(1).unwrap();
        self.duration_ns = self.duration_ns.checked_add(duration_ns).unwrap();
    }
}

#[derive(Default)]
struct TotalsByLength {
    default: HashMap<usize, Totals>,
    fnv: HashMap<usize, Totals>,
    xx: HashMap<usize, Totals>,
    xxh3: HashMap<usize, Totals>,
    nop: HashMap<usize, Totals>,
}

fn hash<H: Hasher + Default>(s: &str) -> u64 {
    let s = hint::black_box(s);
    let start = Instant::now();
    let mut hasher = H::default();
    hasher.write(s.as_bytes());
    hint::black_box(hasher.finish());
    start.elapsed().as_nanos() as u64
}

fn measure_time_overhead_ns() -> u64 {
    let count = 1000000;
    let mut sum = 0;
    for _ in 0..count {
        let start = hint::black_box(Instant::now());
        sum += start.elapsed().as_nanos() as u64;
    }
    sum / count
}

fn iteration(strings: &[String], lengths: &[usize], totals_by_len: &mut TotalsByLength) {
    let strings = hint::black_box(strings);

    let measure_overhead_ns = measure_time_overhead_ns();

    let mut order = [0, 1, 2, 3, 4];

    for s in strings {
        order.shuffle(&mut thread_rng());
        for i in order {
            // Populate cache.
            hint::black_box(s.chars().count());
            match i {
                0 => totals_by_len
                    .fnv
                    .entry(s.len())
                    .or_default()
                    .update(hash::<FnvHasher>(s), measure_overhead_ns),
                1 => totals_by_len
                    .xx
                    .entry(s.len())
                    .or_default()
                    .update(hash::<XxHash64>(s), measure_overhead_ns),
                2 => totals_by_len
                    .xxh3
                    .entry(s.len())
                    .or_default()
                    .update(hash::<Hash64>(s), measure_overhead_ns),
                3 => totals_by_len
                    .default
                    .entry(s.len())
                    .or_default()
                    .update(hash::<DefaultHasher>(s), measure_overhead_ns),
                4 => totals_by_len
                    .nop
                    .entry(s.len())
                    .or_default()
                    .update(hash::<NopHasher>(s), measure_overhead_ns),
                _ => unreachable!(),
            }
        }
    }

    println!("iteration");
    for len in lengths {
        let fnv = totals_by_len.fnv.get(len).unwrap().avg_ns();
        let xx = totals_by_len.xx.get(len).unwrap().avg_ns();
        let xxh3 = totals_by_len.xxh3.get(len).unwrap().avg_ns();
        let default = totals_by_len.default.get(len).unwrap().avg_ns();
        let nop = totals_by_len.nop.get(len).unwrap().avg_ns();
        println!(
            "{len:>4}:  fnv={fnv:>4}   xx={xx:>4}  xxh3={xxh3:>4}   def={default:>4}   nop={nop:>4}"
        );
    }
}

fn main() {
    let measure_overhead_ns = measure_time_overhead_ns();
    println!("time measure overhead: {measure_overhead_ns} ns");

    // Generate length 0, 1, 2, 3, 4, 6, 8, 10, 12, 14, 16, 20, 24, 28, 32, ...
    let mut strings = Vec::new();
    let mut lengths = Vec::new();
    strings.push("".to_owned());
    lengths.push(0);
    for i in 1usize..=1024 {
        let significant_bits =
            mem::size_of_val(&i) * 8 - i.leading_zeros() as usize - i.trailing_zeros() as usize;
        if significant_bits <= 3 {
            lengths.push(i);
            for _ in 0..100000 {
                strings.push(random_string(i));
            }
        }
    }

    let mut totals = TotalsByLength::default();

    loop {
        strings.shuffle(&mut thread_rng());
        iteration(&strings, &lengths, &mut totals);
    }
}
