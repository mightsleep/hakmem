//! The rank9 directory on shapes proptest rarely draws: big dense
//! slices (many inventory entries), big sparse ones (many empty blocks
//! between entries), every block-boundary length, and the panics.

use hakmem::rank9::Rank9;
use hakmem::{laws, slice};

fn with_dir(words: &[u64], f: impl FnOnce(&Rank9<'_>)) {
    let mut counts = vec![0; Rank9::counts_len(words.len())];
    let mut select = vec![0; Rank9::select_len(words.len())];
    Rank9::build(words, &mut counts, &mut select);
    f(&Rank9::new(words, &counts, &select));
}

fn xorshift(mut s: u64) -> impl FnMut() -> u64 {
    move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    }
}

fn check_all(words: &[u64]) {
    with_dir(words, |dir| {
        let n = words.len() * 64;
        for i in 0..=n + 1 {
            assert!(
                laws::rank9_matches_slice(dir, i, i),
                "len={} i={i}",
                words.len()
            );
            assert!(
                laws::rank9_rank_steps_by_bit(dir, i),
                "len={} i={i}",
                words.len()
            );
        }
        for k in 0..=dir.count_ones() + 1 {
            assert!(
                laws::rank9_select_inverts_rank(dir, k),
                "len={} k={k}",
                words.len()
            );
        }
    });
}

#[test]
fn every_boundary_length_zeros_ones_and_a_lone_top_bit() {
    for len in [0usize, 1, 2, 7, 8, 9, 15, 16, 17, 23, 24, 25, 63, 64, 65] {
        check_all(&vec![0u64; len]);
        check_all(&vec![u64::MAX; len]);
        if len > 0 {
            let mut lone = vec![0u64; len];
            lone[len - 1] = 1 << 63;
            check_all(&lone);
            let mut alternating = vec![0xAAAA_AAAA_AAAA_AAAAu64; len];
            alternating[0] = 1;
            check_all(&alternating);
        }
    }
}

#[test]
fn dense_4096_words_crosses_hundreds_of_entries() {
    let mut next = xorshift(0x2545_F491_4F6C_DD1D);
    let words: Vec<u64> = (0..4096).map(|_| next()).collect();
    with_dir(&words, |dir| {
        assert!(dir.count_ones() > 100_000, "dense input expected");
        for i in (0..=words.len() * 64).step_by(97) {
            assert_eq!(dir.rank(i), slice::rank(&words, i), "i={i}");
        }
        for k in (0..dir.count_ones()).step_by(89) {
            assert_eq!(dir.select(k), slice::select(&words, k), "k={k}");
            assert!(laws::rank9_select_inverts_rank(dir, k), "k={k}");
        }
        assert_eq!(dir.select(dir.count_ones()), None);
    });
}

#[test]
fn sparse_4096_words_one_bit_per_thousand() {
    let mut words = vec![0u64; 4096];
    let mut p = 5;
    while p < words.len() * 64 {
        words[p / 64] |= 1 << (p % 64);
        p += 997;
    }
    with_dir(&words, |dir| {
        assert!(
            dir.count_ones() < 512,
            "one sample expected, got {} ones",
            dir.count_ones()
        );
        for i in (0..=words.len() * 64).step_by(131) {
            assert_eq!(dir.rank(i), slice::rank(&words, i), "i={i}");
        }
        for k in 0..=dir.count_ones() {
            assert_eq!(dir.select(k), slice::select(&words, k), "k={k}");
        }
    });
}

#[test]
#[should_panic(expected = "counts needs")]
fn build_rejects_a_short_counts_slice() {
    let bits = [1u64; 9];
    let mut counts = vec![0; Rank9::counts_len(9) - 1];
    let mut select = vec![0; Rank9::select_len(9)];
    Rank9::build(&bits, &mut counts, &mut select);
}

#[test]
#[should_panic(expected = "select needs")]
fn build_rejects_a_short_select_slice() {
    let bits = [1u64; 9];
    let mut counts = vec![0; Rank9::counts_len(9)];
    let mut select = vec![0; Rank9::select_len(9) - 1];
    Rank9::build(&bits, &mut counts, &mut select);
}

/// Densities that put every inventory span shape on the table: 512 set
/// bits in up to 4 blocks (tiny), in 4 to 15 blocks (flat), 16 to 63 (two
/// levels), 64 to 127 (16-bit positions) and more (32-bit positions).
#[test]
fn every_inventory_span_shape() {
    for stride in [1usize, 2, 3, 4, 8, 12, 40, 70, 100, 200, 400] {
        // One set bit every `stride` bits: 512 of them span 512 * stride
        // bits, that is `stride` blocks.
        let n_words = 4096;
        let mut words = vec![0u64; n_words];
        let mut p = stride / 2;
        while p < n_words * 64 {
            words[p / 64] |= 1 << (p % 64);
            p += stride;
        }
        with_dir(&words, |dir| {
            assert!(dir.has_select());
            for k in (0..dir.count_ones()).step_by(7) {
                assert_eq!(
                    dir.select(k),
                    slice::select(&words, k),
                    "stride={stride} k={k}"
                );
                assert!(
                    laws::rank9_select_inverts_rank(dir, k),
                    "stride={stride} k={k}"
                );
            }
            assert_eq!(dir.select(dir.count_ones()), None);
        });
        // Mixed: a dense run, then the sparse stride, so spans change
        // shape inside one directory.
        for w in words.iter_mut().take(300) {
            *w = 0xF0F0_F0F0_F0F0_F0F0;
        }
        check_all(&words[..600]);
    }
}

/// No select inventory: `select` falls back to the block-count search.
#[test]
fn without_select_storage() {
    let mut next = xorshift(0xDEAD_BEEF_CAFE_F00D);
    let words: Vec<u64> = (0..1000).map(|_| next() & next() & next()).collect();
    let mut counts = vec![0; Rank9::counts_len(words.len())];
    Rank9::build(&words, &mut counts, &mut []);
    let dir = Rank9::new(&words, &counts, &[]);
    assert!(!dir.has_select());
    for k in (0..=dir.count_ones()).step_by(5) {
        assert_eq!(dir.select(k), slice::select(&words, k), "k={k}");
    }
    for i in (0..=words.len() * 64).step_by(101) {
        assert!(laws::rank9_matches_slice(&dir, i, i));
    }
}
