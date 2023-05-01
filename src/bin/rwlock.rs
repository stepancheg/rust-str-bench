use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use parking_lot::RwLock;

fn main() {
    let n_threads = 8;
    let rwlock = RwLock::new(());
    let rwlock = Arc::new(rwlock);

    let mut threads = Vec::new();
    for i in 0..n_threads {
        let rwlock = rwlock.clone();
        threads.push(thread::spawn(move || loop {
            let start = Instant::now();
            let batch = 1000_0000;
            let mut total = 0;
            while start.elapsed() < Duration::from_secs(1) {
                for _ in 0..batch {
                    let _guard = rwlock.read();
                }
                total += batch;
            }
            let elapsed_ns = start.elapsed().as_nanos() as u64;
            println!("Thread {}: {}ns", i, elapsed_ns / total);
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }
}
