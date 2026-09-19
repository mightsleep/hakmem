//! Property tests: every exported law, over `u32`, `u64` and `u128`.
//!
//! Run twice to cover both code paths:
//! `cargo test -p hakmem` (portable) and
//! `RUSTFLAGS="-C target-feature=+bmi2" cargo test -p hakmem`.

use hakmem::laws;
use proptest::prelude::*;

macro_rules! laws_for {
    ($m:ident, $t:ty) => {
        laws_for!($m, $t, any::<$t>(), <$t>::MAX);
    };
    ($m:ident, $t:ty, $strategy:expr, $max:expr) => {
        mod $m {
            use super::*;
            const BITS: u32 = <$t as hakmem::Word>::BITS;

            proptest! {
                // runs
                #[test]
                fn run_starts_one_is_identity(x in $strategy) {
                    prop_assert!(laws::run_starts_one_is_identity(x));
                }
                #[test]
                fn run_starts_shrinks(x in $strategy, k in 1..BITS) {
                    prop_assert!(laws::run_starts_shrinks(x, k));
                }
                #[test]
                fn run_starts_matches_reference(x in $strategy, k in 1..=BITS) {
                    prop_assert!(laws::run_starts_matches_reference(x, k));
                }
                // scan
                #[test]
                fn prefix_xor_after_delta_is_identity(x in $strategy) {
                    prop_assert!(laws::prefix_xor_after_delta_is_identity(x));
                }
                #[test]
                fn delta_after_prefix_xor_is_identity(x in $strategy) {
                    prop_assert!(laws::delta_after_prefix_xor_is_identity(x));
                }
                #[test]
                fn prefix_or_is_smear_from_first_set(x in $strategy) {
                    prop_assert!(laws::prefix_or_is_smear_from_first_set(x));
                }
                #[test]
                fn prefix_xor_matches_reference(x in $strategy) {
                    prop_assert!(laws::prefix_xor_matches_reference(x));
                }
                // set view
                #[test]
                fn rank_is_monotone(x in $strategy, i in 0..BITS) {
                    prop_assert!(laws::rank_is_monotone(x, i));
                }
                #[test]
                fn rank_full_is_popcount(x in $strategy) {
                    prop_assert!(laws::rank_full_is_popcount(x));
                }
                #[test]
                fn first_last_bracket_set_bits(x in $strategy) {
                    prop_assert!(laws::first_last_bracket_set_bits(x));
                }
                #[test]
                fn select_is_rank_inverse(x in $strategy, k in 0..BITS) {
                    prop_assert!(laws::select_is_rank_inverse(x, k));
                }
                #[test]
                fn select_matches_reference(x in $strategy, k in 0..BITS) {
                    prop_assert!(laws::select_matches_reference(x, k));
                }
                // compact / expand
                #[test]
                fn compact_matches_reference(x in $strategy, m in $strategy) {
                    prop_assert!(laws::compact_matches_reference(x, m));
                }
                #[test]
                fn expand_matches_reference(x in $strategy, m in $strategy) {
                    prop_assert!(laws::expand_matches_reference(x, m));
                }
                #[test]
                fn compact_expand_roundtrip(x in $strategy, m in $strategy) {
                    prop_assert!(laws::compact_expand_roundtrip(x, m));
                }
                #[test]
                fn expand_compact_roundtrip(x in $strategy, m in $strategy) {
                    prop_assert!(laws::expand_compact_roundtrip(x, m));
                }
                #[test]
                fn compact_preserves_popcount(x in $strategy, m in $strategy) {
                    prop_assert!(laws::compact_preserves_popcount(x, m));
                }
                #[test]
                fn compact_composes(x in $strategy, m in $strategy, n in $strategy) {
                    prop_assert!(laws::compact_composes(x, m, n));
                }
                // reduces / positions
                #[test]
                fn small_reduces_match_reference(x in $strategy) {
                    prop_assert!(laws::small_reduces_match_reference(x));
                }
                #[test]
                fn positions_enumerate_set_bits(x in $strategy) {
                    prop_assert!(laws::positions_enumerate_set_bits(x));
                }
                // swar
                #[test]
                fn swar_lanes_match_reference(x in $strategy, b in any::<u8>(), n in 1u8..=128) {
                    prop_assert!(laws::swar_lanes_match_reference(x, b, n));
                }
                // slice
                #[test]
                fn slice_ops_match_reference(words in prop::collection::vec($strategy, 1..=4), i in 0usize..(4 * BITS as usize + 8), k in 1..=BITS) {
                    prop_assert!(laws::slice_ops_match_reference(&words, i, k));
                }
                // permute / fill
                #[test]
                fn delta_swap_is_involution(x in $strategy, m in $strategy, s in 1..BITS) {
                    prop_assert!(laws::delta_swap_is_involution(x, m, s));
                }
                #[test]
                fn fills_match_reference(x in $strategy, p in $strategy, s in 1..BITS) {
                    prop_assert!(laws::fills_match_reference(x, p, s));
                }
                // basics / catalogue
                #[test]
                fn basics_match_reference(x in $strategy, y in $strategy, m in $strategy) {
                    prop_assert!(laws::basics_match_reference(x, y, m));
                }
                #[test]
                fn next_same_popcount_matches_reference(x in $strategy) {
                    // The reference is a linear search; keep it near the answer.
                    let x = hakmem::Word::or(x, hakmem::Word::shl($max, (BITS - 4).min(BITS - 1)));
                    prop_assert!(laws::next_same_popcount_matches_reference(x));
                }
                #[test]
                fn pow2_helpers_match_reference(x in $strategy) {
                    prop_assert!(laws::pow2_helpers_match_reference(x));
                }
                #[test]
                fn longest_run_matches_reference(x in $strategy) {
                    prop_assert!(laws::longest_run_matches_reference(x));
                }
                #[test]
                fn gray_code_laws(x in $strategy) {
                    prop_assert!(laws::gray_code_laws(x));
                }
                #[test]
                fn find_escaped_matches_reference(words in prop::collection::vec($strategy, 1..=4), c in any::<bool>()) {
                    prop_assert!(laws::find_escaped_matches_reference(&words, c));
                }
                // algebra: composition / homomorphisms
                #[test]
                fn run_starts_composes(x in $strategy, a in 1..=BITS, b in 1..=BITS) {
                    prop_assume!(a + b - 1 <= BITS);
                    prop_assert!(laws::run_starts_composes(x, a, b));
                }
                #[test]
                fn run_starts_preserves_and(x in $strategy, y in $strategy, k in 1..=BITS) {
                    prop_assert!(laws::run_starts_preserves_and(x, y, k));
                }
                #[test]
                fn fills_are_closure_operators(x in $strategy, p in $strategy, s in 1..BITS) {
                    prop_assert!(laws::fills_are_closure_operators(x, p, s));
                }
                #[test]
                fn xor_linear_combinators(x in $strategy, y in $strategy, m in $strategy, s in 1..BITS) {
                    prop_assert!(laws::xor_linear_combinators(x, y, m, s));
                }
                #[test]
                fn zero_bytes_turns_or_into_and(x in $strategy, y in $strategy) {
                    prop_assert!(laws::zero_bytes_turns_or_into_and(x, y));
                }
                #[test]
                fn expand_composes(x in $strategy, m in $strategy, n in $strategy) {
                    prop_assert!(laws::expand_composes(x, m, n));
                }
                #[test]
                fn delta_swaps_merge(x in $strategy, m1 in $strategy, m2 in $strategy, s in 1..BITS) {
                    prop_assert!(laws::delta_swaps_merge(x, m1, m2, s));
                }
                #[test]
                fn gray_successor_flips_lowest_set(x in $strategy) {
                    prop_assume!(x != $max);
                    prop_assert!(laws::gray_successor_flips_lowest_set(x));
                }
                #[test]
                fn select_inverts_rank_on_set_bits(x in $strategy, i in 0..BITS) {
                    prop_assert!(laws::select_inverts_rank_on_set_bits(x, i));
                }
                // grid
                #[test]
                fn block_starts_laws(rows in prop::collection::vec($strategy, 1..=8), w1 in 1..=BITS, h1 in 1u32..=8, w2 in 1..=BITS, h2 in 1u32..=8) {
                    prop_assume!(w1 + w2 - 1 <= BITS);
                    prop_assert!(laws::block_starts_laws(&rows, w1, h1, w2, h2));
                }
                // dilated / Morton
                #[test]
                fn dilated_roundtrip(x in $strategy) {
                    prop_assert!(laws::dilated_roundtrip::<$t, 2>(x));
                    prop_assert!(laws::dilated_roundtrip::<$t, 3>(x));
                }
                #[test]
                fn dilated_incr_is_add_one(x in $strategy) {
                    prop_assert!(laws::dilated_incr_is_add_one::<$t, 2>(x));
                    prop_assert!(laws::dilated_incr_is_add_one::<$t, 3>(x));
                }
                #[test]
                fn dilated_decr_after_incr_is_identity(x in $strategy) {
                    prop_assert!(laws::dilated_decr_after_incr_is_identity::<$t, 2>(x));
                    prop_assert!(laws::dilated_decr_after_incr_is_identity::<$t, 3>(x));
                }
                #[test]
                fn dilated_add_is_add(a in $strategy, b in $strategy) {
                    prop_assert!(laws::dilated_add_is_add::<$t, 2>(a, b));
                    prop_assert!(laws::dilated_add_is_add::<$t, 3>(a, b));
                }
                #[test]
                fn morton_roundtrip(x in $strategy, y in $strategy) {
                    prop_assert!(laws::morton_roundtrip(x, y));
                }
                #[test]
                fn morton_steps_are_unit_moves(x in $strategy, y in $strategy) {
                    prop_assert!(laws::morton_steps_are_unit_moves(x, y));
                }
                #[test]
                fn morton_aligned_block_is_contiguous(x in $strategy, y in $strategy) {
                    prop_assert!(laws::morton_aligned_block_is_contiguous(x, y));
                }
            }
        }
    };
}

laws_for!(u8_laws, u8);
laws_for!(u16_laws, u16);
laws_for!(u32_laws, u32);
laws_for!(u64_laws, u64);
laws_for!(u128_laws, u128);

proptest! {
    #[test]
    fn board8_permutations_are_correct(x in any::<u64>()) {
        prop_assert!(laws::board8_permutations_are_correct(x));
    }
}

// Multi-limb carrier: the same laws, no special cases.
laws_for!(
    wide2_laws,
    hakmem::Wide<2>,
    any::<[u64; 2]>().prop_map(hakmem::Wide::from_limbs),
    <hakmem::Wide<2> as hakmem::Word>::ONES
);
laws_for!(
    wide3_laws,
    hakmem::Wide<3>,
    any::<[u64; 3]>().prop_map(hakmem::Wide::from_limbs),
    <hakmem::Wide<3> as hakmem::Word>::ONES
);

// The broadword `pext` / `pdep` definitions against the loop they
// replaced, on the wide carriers and regardless of which path
// `Word::pext` takes on this target.
mod broadword_compress {
    use hakmem::prelude::*;
    use hakmem::word::{compress_broadword, expand_broadword};
    use proptest::prelude::*;

    fn compress_loop<W: Word>(x: W, mut mask: W) -> W {
        let mut out = W::ZERO;
        let mut k = 0;
        while !mask.is_zero() {
            if x.bit(mask.trailing_zeros()) {
                out = out.or(W::ONE.shl(k));
            }
            k += 1;
            mask = mask.clear_lowest_set();
        }
        out
    }

    fn expand_loop<W: Word>(x: W, mut mask: W) -> W {
        let mut out = W::ZERO;
        let mut k = 0;
        while !mask.is_zero() {
            if x.bit(k) {
                out = out.or(W::ONE.shl(mask.trailing_zeros()));
            }
            k += 1;
            mask = mask.clear_lowest_set();
        }
        out
    }

    proptest! {
        #[test]
        fn u64_matches_loop(x in any::<u64>(), m in any::<u64>()) {
            prop_assert_eq!(compress_broadword(x, m), compress_loop(x, m));
            prop_assert_eq!(expand_broadword(x, m), expand_loop(x, m));
        }

        #[test]
        fn u128_matches_loop(x in any::<u128>(), m in any::<u128>()) {
            prop_assert_eq!(compress_broadword(x, m), compress_loop(x, m));
            prop_assert_eq!(expand_broadword(x, m), expand_loop(x, m));
        }

        #[test]
        fn wide3_matches_loop(x in any::<[u64; 3]>(), m in any::<[u64; 3]>()) {
            let (x, m) = (Wide::from_limbs(x), Wide::from_limbs(m));
            prop_assert_eq!(compress_broadword(x, m), compress_loop(x, m));
            prop_assert_eq!(expand_broadword(x, m), expand_loop(x, m));
        }
    }
}

// The rank9 directory against the linear scans it indexes, on dense and
// on sparse bit slices, across block and sample boundaries.
mod rank9 {
    use hakmem::laws;
    use hakmem::rank9::Rank9;
    use proptest::prelude::*;

    fn with_dir(words: &[u64], f: impl FnOnce(&Rank9<'_>)) {
        let mut counts = vec![0; Rank9::counts_len(words.len())];
        let mut select = vec![0; Rank9::select_len(words.len())];
        Rank9::build(words, &mut counts, &mut select);
        f(&Rank9::new(words, &counts, &select));
    }

    /// Mostly empty words, some full, some single bits: long empty
    /// stretches between set bits exercise the block search.
    fn sparse_words() -> impl Strategy<Value = Vec<u64>> {
        prop::collection::vec(
            prop_oneof![
                7 => Just(0u64),
                1 => any::<u64>(),
                1 => (0u32..64).prop_map(|b| 1u64 << b),
            ],
            0..=160,
        )
    }

    proptest! {
        #[test]
        fn dense_matches_slice(words in prop::collection::vec(any::<u64>(), 0..=40), i in 0usize..=40 * 64 + 8, k in 0usize..=40 * 64 + 8) {
            with_dir(&words, |dir| {
                assert!(laws::rank9_matches_slice(dir, i, k), "i={i} k={k}");
                assert!(laws::rank9_select_inverts_rank(dir, k), "k={k}");
                assert!(laws::rank9_rank_steps_by_bit(dir, i), "i={i}");
            });
        }

        #[test]
        fn sparse_matches_slice(words in sparse_words(), i in 0usize..=160 * 64 + 8, k in 0usize..=160 * 64 + 8) {
            with_dir(&words, |dir| {
                assert!(laws::rank9_matches_slice(dir, i, k), "i={i} k={k}");
                assert!(laws::rank9_select_inverts_rank(dir, k), "k={k}");
                assert!(laws::rank9_rank_steps_by_bit(dir, i), "i={i}");
            });
        }
    }
}

// Sliding attacks on the 8×8 board against a square-by-square walk.
mod board8 {
    use hakmem::laws;
    use hakmem::permute::board8::Dir;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn slides_match_reference(pieces in any::<u64>(), empty in any::<u64>(), d in 0usize..8) {
            prop_assert!(laws::board8_slides_match_reference(pieces, empty, Dir::ALL[d]));
        }

        #[test]
        fn single_piece_slides_match_reference(sq in 0u32..64, empty in any::<u64>(), d in 0usize..8) {
            prop_assert!(laws::board8_slides_match_reference(1 << sq, empty, Dir::ALL[d]));
        }
    }
}

// Byte lanes: both carriers against the per-lane definitions, the table
// composition, the bridge to the word algebra, and the wide carrier
// against its two SWAR halves (which on x86 with SSSE3 and on aarch64
// pits the vector instructions against the scalar definitions).
mod lanes {
    use hakmem::lanes::{Lanes, U8x8, U8x16};
    use hakmem::laws;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn u8x8_matches_reference(a in any::<u64>(), b in any::<u64>(), n in 0u32..10, t in any::<[u8; 16]>()) {
            prop_assert!(laws::lanes_match_reference(U8x8::new(a), U8x8::new(b), n, t));
        }

        #[test]
        fn u8x16_matches_reference(a in any::<[u8; 16]>(), b in any::<[u8; 16]>(), n in 0u32..10, t in any::<[u8; 16]>()) {
            prop_assert!(laws::lanes_match_reference(U8x16::load(&a), U8x16::load(&b), n, t));
        }

        #[test]
        fn lut16_composes(x in any::<[u8; 16]>(), a in any::<[u8; 16]>(), b in any::<[u8; 16]>()) {
            prop_assert!(laws::lut16_composes(U8x16::load(&x), a, b));
            prop_assert!(laws::lut16_composes(U8x8::load(&x[..8]), a, b));
        }

        #[test]
        fn lanes_agree_with_bits(x in any::<u64>(), b in any::<u8>()) {
            prop_assert!(laws::lanes_agree_with_bits(x, b));
        }

        #[test]
        fn u8x16_agrees_with_halves(a in any::<(u64, u64)>(), b in any::<(u64, u64)>(), n in 0u32..10, t in any::<[u8; 16]>()) {
            prop_assert!(laws::u8x16_agrees_with_halves(a, b, n, t));
        }
    }
}
