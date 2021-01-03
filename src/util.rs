use std::time;
use rand::seq::SliceRandom;

pub fn ms(millis: u64) -> time::Duration {
    time::Duration::from_millis(millis)
}

pub fn choose<T: Copy>(s: Vec<T>) -> Option<T> {
    s.choose(&mut rand::thread_rng()).map(|x| *x)
}
