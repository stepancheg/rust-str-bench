use std::hint;
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

#[derive(Default)]
struct DumbRwLock {
    state: AtomicUsize,
}

impl DumbRwLock {
    fn lock_shared(&self) {
        self.state.fetch_add(1, atomic::Ordering::Acquire);
        return;

        let mut state = self.state.load(atomic::Ordering::Relaxed);
        loop {
            let new_state = state.checked_add(1).unwrap();
            match self.state.compare_exchange_weak(
                state,
                new_state,
                atomic::Ordering::Acquire,
                atomic::Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(s) => state = s,
            }
        }
    }

    fn unlock_shared(&self) {
        self.state.fetch_sub(1, atomic::Ordering::Release);
        return;

        let mut state = self.state.load(atomic::Ordering::Relaxed);
        loop {
            let new_state = state.checked_sub(1).unwrap();
            match self.state.compare_exchange_weak(
                state,
                new_state,
                atomic::Ordering::Release,
                atomic::Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(s) => state = s,
            }
        }
    }
}

fn main() {
    let n_threads = 8;
    let rwlock = DumbRwLock::default();
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
                    rwlock.lock_shared();
                    hint::black_box(&rwlock);
                    rwlock.unlock_shared();
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
