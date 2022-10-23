#![feature(portable_simd)]

use std::hint;
use std::simd::u32x4;
use std::simd::u32x8;
use std::simd::LaneCount;
use std::simd::Mask;
use std::simd::Simd;
use std::simd::SimdPartialEq;
use std::simd::SupportedLaneCount;
use std::simd::ToBitMask;

use rand::Rng;
use rust_str_bench::Benchmark;

/// Copy-paste this into the playground.
pub fn test_256_do_something(haystack: &[u32], needle: u32) -> Option<usize> {
    type V = u32x8;
    let needle = V::splat(needle);
    let next = V::from_slice(haystack);
    let mask = next.simd_eq(needle);
    if mask.any() {
        return Some(mask.to_bitmask().trailing_zeros() as usize);
    } else {
        return None;
    }
}

fn gen_inputs() -> Vec<(Vec<u32>, u32)> {
    let mut rng = rand::thread_rng();
    let mut inputs = Vec::new();
    for _ in 0..1000 {
        let mut haystack = Vec::new();
        let len = rng.gen_range(0..=12);
        for i in 0..len {
            haystack.push(i);
        }
        let needle = if len != 0 && rng.gen() {
            rng.gen_range(0..len)
        } else {
            22
        };
        inputs.push((haystack, needle));
    }
    inputs
}

fn find_position(haystack: &[u32], needle: u32) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

#[inline]
fn find_position_simd<const LANES: usize>(haystack: &[u32], needle: u32) -> Option<usize>
where
    LaneCount<LANES>: SupportedLaneCount,
    Mask<i32, LANES>: ToBitMask<BitMask = u8>,
{
    let needles = Simd::<u32, LANES>::splat(needle);
    let mut i = 0;
    while i + LANES <= haystack.len() {
        let chunk = Simd::from_slice(&haystack[i..]);
        let eq = chunk.simd_eq(needles);
        if eq.any() {
            return Some(i + eq.to_bitmask().trailing_zeros() as usize);
        }
        i += LANES;
    }
    find_position(&haystack[i..], needle).map(|pos| pos + i)
}

pub fn find_position_simd_128_generic(haystack: &[u32], needle: u32) -> Option<usize> {
    return find_position_simd::<4>(haystack, needle);
}

pub fn find_position_simd_256_generic(haystack: &[u32], needle: u32) -> Option<usize> {
    return find_position_simd::<8>(haystack, needle);
}

pub fn find_position_simd_256_with_target_feature(haystack: &[u32], needle: u32) -> Option<usize> {
    #[target_feature(enable = "avx")]
    #[inline]
    unsafe fn doit(haystack: &[u32], needle: u32) -> Option<usize> {
        return find_position_simd::<8>(haystack, needle);
    }

    unsafe {
        return doit(haystack, needle);
    }
}

pub fn find_position_simd_128(haystack: &[u32], needle: u32) -> Option<usize> {
    type V = u32x4;
    let needles = V::splat(needle);
    let mut i = 0;
    while i + V::LANES <= haystack.len() {
        let chunk = V::from_slice(&haystack[i..]);
        let eq = chunk.simd_eq(needles);
        if eq.any() {
            return Some(i + eq.to_bitmask().trailing_zeros() as usize);
        }
        i += V::LANES;
    }
    find_position(&haystack[i..], needle).map(|pos| pos + i)
}

fn test() {
    let inputs = gen_inputs();
    for (haystack, needle) in inputs {
        let pos = find_position(&haystack, needle);
        assert_eq!(pos, find_position_simd_128_generic(&haystack, needle));
        assert_eq!(pos, find_position_simd_256_generic(&haystack, needle));
        assert_eq!(pos, find_position_simd_128(&haystack, needle));
    }
}

fn main() {
    test();

    let inputs = gen_inputs();
    rust_str_bench::benchmark(
        inputs.len(),
        &[
            Benchmark::new("find_position", || {
                hint::black_box(&inputs);
                for (haystack, needle) in &inputs {
                    hint::black_box(find_position(haystack, *needle));
                }
            }),
            Benchmark::new("find_position_simd_128_generic", || {
                hint::black_box(&inputs);
                for (haystack, needle) in &inputs {
                    hint::black_box(find_position_simd_128_generic(haystack, *needle));
                }
            }),
            Benchmark::new("find_position_simd_128", || {
                hint::black_box(&inputs);
                for (haystack, needle) in &inputs {
                    hint::black_box(find_position_simd_128(haystack, *needle));
                }
            }),
            Benchmark::new("find_position_simd_256_generic", || {
                hint::black_box(&inputs);
                for (haystack, needle) in &inputs {
                    hint::black_box(find_position_simd_256_generic(haystack, *needle));
                }
            }),
            Benchmark::new("find_position_simd_256_with_target_feature", || {
                hint::black_box(&inputs);
                for (haystack, needle) in &inputs {
                    hint::black_box(find_position_simd_256_with_target_feature(
                        haystack, *needle,
                    ));
                }
            }),
        ],
    );
}
