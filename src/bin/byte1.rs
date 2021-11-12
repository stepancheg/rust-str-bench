#![feature(bench_black_box)]

use std::hint;

use rand::Rng;
use rust_str_bench::Benchmark;

fn gen_random_strings() -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut strings = Vec::new();
    for _ in 0..1_000 {
        let len = rng.gen_range(0..=30);
        let mut s = String::new();
        for _ in 0..len {
            s.push(rng.gen_range(b'a'..=b'z') as char);
        }
        strings.push(s);
    }
    strings
}

#[inline(never)]
fn find_char_chars_enumerate(s: &str, c: char) -> Option<usize> {
    for (i, ch) in s.chars().enumerate() {
        if ch == c {
            return Some(i);
        }
    }
    None
}

#[inline(never)]
fn find_char_string_find_char(s: &str, c: char) -> Option<usize> {
    s.find(c)
}

#[inline(never)]
fn find_char_with_ascii(s: &str, needle: char) -> Option<usize> {
    if needle as u32 <= 0xf7 {
        return s.as_bytes().iter().position(|&b| b == needle as u8);
    }
    find_char_string_find_char(s, needle)
}

#[inline(never)]
fn find_char_with_ascii_memchr(s: &str, needle: char) -> Option<usize> {
    if needle as u32 <= 0xf7 {
        return memchr::memchr(needle as u8, s.as_bytes());
    }
    find_char_string_find_char(s, needle)
}

#[inline(never)]
fn find_str(s: &str, needle: &str) -> Option<usize> {
    s.find(needle)
}

fn main() {
    let strings = gen_random_strings();
    rust_str_bench::benchmark(
        strings.len(),
        &[
            Benchmark::new("str::find(str)", || {
                for s in &strings {
                    hint::black_box(find_str(s, "a"));
                }
            }),
            Benchmark::new("str::find(char)", || {
                for s in &strings {
                    hint::black_box(find_char_string_find_char(s, 'a'));
                }
            }),
            Benchmark::new("chars_enumerate", || {
                for s in &strings {
                    hint::black_box(find_char_chars_enumerate(s, 'a'));
                }
            }),
            Benchmark::new("find_char_with_ascii", || {
                for s in &strings {
                    hint::black_box(find_char_with_ascii(s, 'a'));
                }
            }),
            Benchmark::new("find_char_with_ascii_memchr", || {
                for s in &strings {
                    hint::black_box(find_char_with_ascii_memchr(s, 'a'));
                }
            }),
        ],
    );
}
