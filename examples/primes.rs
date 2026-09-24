//! A bitmap that answers "how many below `n`" and "where is the `k`-th"
//! in constant time: the primes below ten million, one bit each, with a
//! rank/select directory beside them.
//!
//! `cargo run --release --example primes`

use hakmem::prelude::*;
use hakmem::rank9::Rank9;

/// Bit `n` set when `n` is prime, for `n < limit`.
fn sieve(limit: usize) -> Vec<u64> {
    let mut words = vec![u64::MAX; limit.div_ceil(64)];
    words.clear_bit(0);
    words.clear_bit(1);
    for n in 2..=limit.isqrt() {
        if words.bit(n) {
            for m in (n * n..limit).step_by(n) {
                words.clear_bit(m);
            }
        }
    }
    // Bits past the limit are not primes, they are padding.
    if !limit.is_multiple_of(64) {
        *words.last_mut().unwrap() &= u64::MAX >> (64 - limit % 64);
    }
    words
}

fn main() {
    let limit = 10_000_000;
    let bits = sieve(limit);

    let mut counts = vec![0; Rank9::counts_len(bits.len())];
    let mut select = vec![0; Rank9::select_len(bits.len())];
    let primes = Rank9::build(&bits, &mut counts, &mut select);

    // π(10^7), π(10^6) and the largest prime below 10^6, all known.
    assert_eq!(primes.count_ones(), 664_579);
    // No 664 580th prime below the limit, so no position for it.
    assert_eq!(primes.select(664_579), None);
    assert_eq!(primes.rank(1_000_000), 78_498);
    assert_eq!(primes.select(78_497), Some(999_983));

    // The same answers from the bare words, by scanning.
    assert_eq!(bits.rank(1_000_000), 78_498);
    assert_eq!(bits.select(78_497), Some(999_983));

    // Twin primes: a prime whose successor two further is prime too.
    let twins = bits
        .positions()
        .filter(|&p| bits.next_set_from(p + 1) == Some(p + 2))
        .count();
    println!(
        "{} primes below {limit}, {twins} twin pairs; the directory costs {} words next to {}",
        primes.count_ones(),
        counts.len() + select.len(),
        bits.len()
    );
}
