use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hint;

use fnv::FnvHasher;
use rust_str_bench::benchmark;
use rust_str_bench::random_strings::random_string;
use rust_str_bench::Benchmark;

fn bm_with_hash_function<'a, F: Fn(&[u8]) -> u64 + 'a>(
    name: &str,
    f: F,
    strings: &'a [String],
) -> Benchmark<'a> {
    Benchmark::new(name, move || {
        for s in strings {
            let s = hint::black_box(s);
            hint::black_box(f(s.as_bytes()));
        }
    })
}

fn bm_with_hasher<'a, H: Hasher + Default>(name: &str, strings: &'a [String]) -> Benchmark<'a> {
    bm_with_hash_function(
        name,
        |s| {
            let mut hasher = H::default();
            hasher.write(s);
            hasher.finish()
        },
        strings,
    )
}

fn main() {
    let mut strings = Vec::new();
    for i in 0..100 {
        strings.push(random_string(i));
    }

    benchmark(
        strings.len(),
        &[
            bm_with_hasher::<DefaultHasher>("DefaultHasher", &strings),
            bm_with_hasher::<FnvHasher>("FnvHasher", &strings),
            // twox-hash
            bm_with_hasher::<twox_hash::XxHash64>("twox_hash::XxHash64", &strings),
            bm_with_hasher::<twox_hash::xxh3::Hash64>("twox_hash::xxh3::Hash64", &strings),
            bm_with_hash_function("twox_hash::xxh3::hash64", twox_hash::xxh3::hash64, &strings),
            // xxhash-rust
            bm_with_hasher::<xxhash_rust::xxh64::Xxh64>("xxhash_rust::xxh64::Xxh64", &strings),
            bm_with_hasher::<xxhash_rust::xxh3::Xxh3>("xxhash_rust::xxh3::Xxh3", &strings),
            bm_with_hash_function(
                "xxhash_rust::xxh3::xxh3_64",
                xxhash_rust::xxh3::xxh3_64,
                &strings,
            ),
            bm_with_hash_function(
                "xxhash_rust::xxh64::xxh64",
                |s| xxhash_rust::xxh64::xxh64(s, 0),
                &strings,
            ),
        ],
    );
}
