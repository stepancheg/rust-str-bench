#![feature(portable_simd)]

use std::hint;
use std::mem;
use std::simd::u32x4;
use std::simd::u32x8;
use std::simd::Mask;
use std::simd::Simd;
use std::simd::SimdPartialEq;
use std::simd::ToBitMask;
use std::slice;

use rand::Rng;
use rust_str_bench::Benchmark;

#[repr(C)]
pub struct Entry {
    hash: u32,
    key: usize,
    value: usize,
}

const SIZE_OF_ENTRY_IN_U32: usize = mem::size_of::<Entry>() / mem::size_of::<u32>();

#[inline]
fn entries_to_u32(haystack: &[Entry]) -> &[u32] {
    unsafe {
        slice::from_raw_parts(
            haystack.as_ptr() as *const u32,
            haystack.len() * mem::size_of::<Entry>() / SIZE_OF_ENTRY_IN_U32,
        )
    }
}

/// Copy-paste this into the playground.
pub unsafe fn test_256_do_something(haystack: &[Entry], needle: u32) -> Option<usize> {
    type V = u32x8;
    let needle = V::splat(needle);
    let next = V::gather_select_unchecked(
        entries_to_u32(haystack),
        Mask::splat(true),
        Simd::from_array([
            0 * SIZE_OF_ENTRY_IN_U32,
            1 * SIZE_OF_ENTRY_IN_U32,
            2 * SIZE_OF_ENTRY_IN_U32,
            3 * SIZE_OF_ENTRY_IN_U32,
            4 * SIZE_OF_ENTRY_IN_U32,
            5 * SIZE_OF_ENTRY_IN_U32,
            6 * SIZE_OF_ENTRY_IN_U32,
            7 * SIZE_OF_ENTRY_IN_U32,
        ]),
        V::splat(0),
    );
    let mask = next.simd_eq(needle);
    if mask.any() {
        return Some(mask.to_bitmask().trailing_zeros() as usize);
    } else {
        return None;
    }
}

fn gen_inputs() -> Vec<(Vec<Entry>, u32)> {
    let mut rng = rand::thread_rng();
    let mut inputs = Vec::new();
    for _ in 0..10000 {
        let mut haystack = Vec::new();
        let len = rng.gen_range(0..=12);
        for i in 0..len {
            haystack.push(Entry {
                hash: i,
                key: 100,
                value: 1000,
            });
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

fn find_position(haystack: &[Entry], needle: u32) -> Option<usize> {
    haystack.iter().position(|b| b.hash == needle)
}

pub fn find_position_simd_128(haystack: &[Entry], needle: u32) -> Option<usize> {
    type V = u32x4;
    let needles = V::splat(needle);
    let mut i = 0;
    while i + V::LANES <= haystack.len() {
        let chunk = unsafe {
            V::gather_select_unchecked(
                entries_to_u32(&haystack[i..]),
                Mask::splat(true),
                Simd::from_array([
                    0 * SIZE_OF_ENTRY_IN_U32,
                    1 * SIZE_OF_ENTRY_IN_U32,
                    2 * SIZE_OF_ENTRY_IN_U32,
                    3 * SIZE_OF_ENTRY_IN_U32,
                ]),
                V::splat(0),
            )
        };
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
            Benchmark::new("find_position_simd_128", || {
                hint::black_box(&inputs);
                for (haystack, needle) in &inputs {
                    hint::black_box(find_position_simd_128(haystack, *needle));
                }
            }),
        ],
    );
}
