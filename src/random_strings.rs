use rand::Rng;

pub fn random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        s.push(rng.gen_range(b'a'..=b'z') as char);
    }
    s
}
