use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::hint;

use hashbrown::raw::RawTable;
use rand::Rng;
use rust_str_bench::benchmark;
use rust_str_bench::Benchmark;

struct MyEntry {
    key: usize,
    value: usize,
    hash: u32,
}

#[derive(Default)]
struct SmallMap {
    entries: Vec<MyEntry>,
    indices: RawTable<usize>,
}

impl SmallMap {
    fn find_index_linear(&self, key: usize, hash: u32) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.hash == hash && e.key == key)
    }

    fn try_insert(&mut self, key: usize) -> bool {
        let hash = hash(key);
        if let Some(_index) = self.find_index_linear(key, hash) {
            false
        } else {
            let index = self.entries.len();
            self.entries.push(MyEntry {
                key,
                value: 0,
                hash,
            });
            self.indices.insert(mix_u32(hash), index, |&index| {
                mix_u32(self.entries[index].hash)
            });
            true
        }
    }
}

struct Input {
    map: SmallMap,
    needle_hash: u32,
    needle: usize,
}

#[inline(always)]
pub(crate) fn mix_u32(n: u32) -> u64 {
    (n as u64).wrapping_mul(0x9e3779b97f4a7c15)
}

fn hash(key: usize) -> u32 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish() as u32
}

fn raw_table(input: &Input) -> Option<usize> {
    unsafe {
        input
            .map
            .indices
            .get(mix_u32(input.needle_hash), |&index| {
                input.map.entries.get_unchecked(index).key == input.needle
            })
            .copied()
    }
}

fn linear(input: &Input) -> Option<usize> {
    input.map.find_index_linear(input.needle, input.needle_hash)
}

fn gen_input(len: usize, positive: bool) -> Input {
    let mut map = SmallMap::default();
    while map.indices.len() < len {
        map.try_insert(rand::thread_rng().gen());
    }

    let needle = if positive && len != 0 {
        map.entries[rand::thread_rng().gen_range(0..len)].key
    } else {
        loop {
            let needle = rand::thread_rng().gen();
            if map.find_index_linear(needle, hash(needle)).is_none() {
                break needle;
            }
        }
    };

    let needle_hash = hash(needle);

    Input {
        map,
        needle_hash,
        needle,
    }
}

const ITERATIONS_IN_BENCHMARK: usize = 1000;
const LENGTH: usize = 32;

#[derive(Default)]
struct Inputs {
    inputs: [[Vec<Input>; 2]; LENGTH],
}

fn make_inputs() -> Inputs {
    println!("Generating inputs...");
    let mut inputs = Inputs::default();
    for len in 0..LENGTH {
        for positive in [true, false] {
            for _ in 0..ITERATIONS_IN_BENCHMARK {
                let input = gen_input(len, positive);
                inputs.inputs[len][positive as usize].push(input);
            }
        }
    }
    inputs
}

fn test_input(input: &Input) {
    assert_eq!(linear(input), raw_table(input));
}

fn test(inputs: &Inputs) {
    println!("Testing...");
    for _ in 1..1000 {
        let input = &inputs.inputs[rand::thread_rng().gen_range(0..LENGTH)]
            [rand::thread_rng().gen_range(0..2)]
            [rand::thread_rng().gen_range(0..ITERATIONS_IN_BENCHMARK)];
        test_input(input);
    }
}

fn main() {
    let inputs = make_inputs();
    test(&inputs);
    let mut benchmarks = Vec::new();
    for len in 0..LENGTH {
        for positive in [true, false] {
            let inputs = &inputs.inputs[len][positive as usize];
            benchmarks.extend(vec![
                Benchmark::new(
                    &format!("linear    len={} pos={}", len, positive),
                    move || {
                        for input in inputs {
                            hint::black_box(input);
                            hint::black_box(linear(input));
                        }
                    },
                ),
                Benchmark::new(
                    &format!("raw_table len={} pos={}", len, positive),
                    move || {
                        for input in inputs {
                            hint::black_box(input);
                            hint::black_box(raw_table(input));
                        }
                    },
                ),
            ]);
        }
    }
    benchmark(ITERATIONS_IN_BENCHMARK, &benchmarks);
    drop(benchmarks);
    drop(inputs);
}
