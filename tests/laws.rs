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
                #[test]
                fn suffix_xor_laws(x in $strategy) {
                    prop_assert!(laws::suffix_xor_laws(x));
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
                // Hilbert
                #[test]
                fn hilbert_matches_reference(x in $strategy, y in $strategy, order in 0..=BITS / 2) {
                    prop_assert!(laws::hilbert_matches_reference(x, y, order));
                }
                #[test]
                fn hilbert_roundtrip(x in $strategy, y in $strategy) {
                    prop_assert!(laws::hilbert_roundtrip(x, y));
                }
                #[test]
                fn hilbert_consecutive_are_adjacent(h in $strategy) {
                    prop_assert!(laws::hilbert_consecutive_are_adjacent(h));
                }
                #[test]
                fn hilbert_order_laws(x in $strategy, y in $strategy, order in 0..BITS / 2) {
                    prop_assert!(laws::hilbert_order_laws(x, y, order));
                }
                #[test]
                fn hilbert3_matches_reference(h in $strategy) {
                    prop_assert!(laws::hilbert3_matches_reference(h));
                }
                #[test]
                fn hilbert3_roundtrip(x in $strategy, y in $strategy, z in $strategy) {
                    prop_assert!(laws::hilbert3_roundtrip(x, y, z));
                }
                #[test]
                fn hilbert3_consecutive_are_adjacent(h in $strategy) {
                    prop_assert!(laws::hilbert3_consecutive_are_adjacent(h));
                }
                #[test]
                fn hilbert3_order_laws(x in $strategy, y in $strategy, z in $strategy, order in 0..BITS / 3) {
                    prop_assert!(laws::hilbert3_order_laws(x, y, z, order));
                }
                // ternary and sign bits
                #[test]
                fn ternary_is_truth_table(a in $strategy, b in $strategy, c in $strategy, t in any::<u8>()) {
                    prop_assert!(laws::ternary_is_truth_table(a, b, c, t));
                }
                #[test]
                fn truth_table_names_the_function(a in $strategy, b in $strategy, c in $strategy) {
                    prop_assert!(laws::truth_table_names_the_function(a, b, c));
                }
                #[test]
                fn signed_overflow_matches_sign_test(a in $strategy, b in $strategy) {
                    prop_assert!(laws::signed_overflow_matches_sign_test(a, b));
                }
                // carry-rippler and gather
                #[test]
                fn next_subset_is_increment_in_mask(x in $strategy, m in $strategy) {
                    prop_assert!(laws::next_subset_is_increment_in_mask(x, m));
                }
                #[test]
                fn subsets_enumerate_each_once(m in $strategy) {
                    let small = <$t as hakmem::Word>::low_ones(12.min(BITS));
                    prop_assert!(laws::subsets_enumerate_each_once(hakmem::Word::and(m, small)));
                }
                #[test]
                fn strided_gather_is_exact(x in $strategy, start in 0..BITS, stride in 1..BITS, k in 1u32..=8, target in 0..BITS) {
                    prop_assert!(laws::strided_gather_is_exact(x, start, stride, k, target));
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

// The batch Hilbert conversions against the per-key ones, on every
// length around the batch sizes (16 and 32 keys on NEON, 64 on
// AVX-512), so each hardware path and its tail is covered.
mod hilbert2_in_place {
    use hakmem::laws;
    use proptest::prelude::*;

    fn xorshift(n: usize, mut s: u64) -> Vec<u64> {
        (0..n)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                s
            })
            .collect()
    }

    #[test]
    fn every_length_to_200() {
        for n in 0..=200 {
            let codes = xorshift(n, 0x9E37_79B9_7F4A_7C15 ^ n as u64);
            let mut scratch = vec![0; n];
            assert!(
                laws::hilbert2_in_place_matches_per_key_u64(&codes, &mut scratch),
                "n={n}"
            );
            // Truncation is the point: the low halves as u32 codes.
            #[allow(clippy::cast_possible_truncation)]
            let codes32: Vec<u32> = codes.iter().map(|&c| c as u32).collect();
            let mut scratch32 = vec![0; n];
            assert!(
                laws::hilbert2_in_place_matches_per_key_u32(&codes32, &mut scratch32),
                "n={n}"
            );
        }
    }

    #[test]
    fn columns_3d() {
        // Every length to 200 and around the groups of 64 and 512;
        // coordinates full width, so the bits above 21 must drop.
        let lengths = (0..=200).chain([511, 512, 513, 575, 577, 1024, 4096 + 64 * 3 + 7]);
        for n in lengths {
            let seed = 0x9E37_79B9_7F4A_7C15 ^ n as u64;
            let (xs, ys, zs) = (
                xorshift(n, seed),
                xorshift(n, seed ^ 1),
                xorshift(n, seed ^ 2),
            );
            let (mut keys, mut out) = (vec![0; n], vec![0; 3 * n]);
            assert!(
                laws::hilbert3_columns_match_per_point(&xs, &ys, &zs, &mut keys, &mut out),
                "n={n}"
            );
        }
    }

    #[test]
    fn hilbert2_columns() {
        // Past the Morton batches (16, 32), the in-place batches (64) and
        // the decode's blocks of 256; coordinates full width.
        for n in (0..=200).chain([255, 256, 257, 511, 513, 1024 + 7]) {
            let seed = 0xD1B5_4A32_D192_ED03 ^ n as u64;
            let (xs, ys) = (xorshift(n, seed), xorshift(n, seed ^ 1));
            let (mut keys, mut out) = (vec![0; n], vec![0; 2 * n]);
            assert!(
                laws::hilbert2_columns_match_per_point_u64(&xs, &ys, &mut keys, &mut out),
                "n={n}"
            );
            #[allow(clippy::cast_possible_truncation)]
            let (xs, ys): (Vec<u32>, Vec<u32>) = (xs.iter().map(|&v| v as u32).collect(), ys.iter().map(|&v| v as u32).collect());
            let (mut keys, mut out) = (vec![0; n], vec![0; 2 * n]);
            assert!(
                laws::hilbert2_columns_match_per_point_u32(&xs, &ys, &mut keys, &mut out),
                "n={n}"
            );
        }
    }

    #[test]
    fn morton2_columns() {
        // Every length to 200 and past the batches of 16 and 32 points;
        // coordinates full width, so the high halves must drop.
        for n in (0..=200).chain([255, 256, 257, 1024 + 7]) {
            let seed = 0x2545_F491_4F6C_DD1D ^ n as u64;
            let (xs, ys) = (xorshift(n, seed), xorshift(n, seed ^ 1));
            let (mut codes, mut out) = (vec![0; n], vec![0; 2 * n]);
            assert!(
                laws::morton2_columns_match_per_point_u64(&xs, &ys, &mut codes, &mut out),
                "n={n}"
            );
            #[allow(clippy::cast_possible_truncation)]
            let (xs, ys): (Vec<u32>, Vec<u32>) = (xs.iter().map(|&v| v as u32).collect(), ys.iter().map(|&v| v as u32).collect());
            let (mut codes, mut out) = (vec![0; n], vec![0; 2 * n]);
            assert!(
                laws::morton2_columns_match_per_point_u32(&xs, &ys, &mut codes, &mut out),
                "n={n}"
            );
        }
    }

    #[test]
    fn group_boundaries_3d() {
        // The `u64` plane kernel takes 512 keys at a time, then whole
        // groups of 64, then leaves the rest to the per-key form; 200
        // never reaches the first of those.
        for n in [
            511,
            512,
            513,
            575,
            576,
            577,
            1023,
            1024,
            1025,
            4096 + 64 * 3 + 7,
        ] {
            let codes = xorshift(n, 0x9E37_79B9_7F4A_7C15 ^ n as u64);
            let mut scratch = vec![0; n];
            assert!(
                laws::hilbert3_in_place_matches_per_key_u64(&codes, &mut scratch),
                "n={n}"
            );
        }
    }

    #[test]
    fn every_length_to_200_3d() {
        for n in 0..=200 {
            let codes = xorshift(n, 0x2545_F491_4F6C_DD1D ^ n as u64);
            let mut scratch = vec![0; n];
            assert!(
                laws::hilbert3_in_place_matches_per_key_u64(&codes, &mut scratch),
                "n={n}"
            );
            #[allow(clippy::cast_possible_truncation)]
            let codes32: Vec<u32> = codes.iter().map(|&c| c as u32).collect();
            let mut scratch32 = vec![0; n];
            assert!(
                laws::hilbert3_in_place_matches_per_key_u32(&codes32, &mut scratch32),
                "n={n}"
            );
        }
    }

    #[test]
    fn every_order() {
        for n in [7usize, 130] {
            let codes = xorshift(n, 0xD1B5_4A32_D192_ED03 ^ n as u64);
            // Truncation is the point: the low halves as u32 codes.
            #[allow(clippy::cast_possible_truncation)]
            let codes32: Vec<u32> = codes.iter().map(|&c| c as u32).collect();
            let (mut a, mut b) = (vec![0; n], vec![0; n]);
            for order in 0..=32 {
                assert!(
                    laws::hilbert2_in_place_order_matches_per_key_u64(&codes, &mut a, order),
                    "2D u64 n={n} order={order}"
                );
            }
            for order in 0..=21 {
                assert!(
                    laws::hilbert3_in_place_order_matches_per_key_u64(&codes, &mut a, order),
                    "3D u64 n={n} order={order}"
                );
            }
            for order in 0..=16 {
                assert!(
                    laws::hilbert2_in_place_order_matches_per_key_u32(&codes32, &mut b, order),
                    "2D u32 n={n} order={order}"
                );
            }
            for order in 0..=10 {
                assert!(
                    laws::hilbert3_in_place_order_matches_per_key_u32(&codes32, &mut b, order),
                    "3D u32 n={n} order={order}"
                );
            }
        }
    }

    #[test]
    fn structured_words() {
        for &w in &[
            0u64,
            u64::MAX,
            0x5555_5555_5555_5555,
            0xAAAA_AAAA_AAAA_AAAA,
            1,
            1 << 63,
        ] {
            let codes = vec![w; 130];
            let mut scratch = vec![0; 130];
            assert!(
                laws::hilbert2_in_place_matches_per_key_u64(&codes, &mut scratch),
                "w={w:#x}"
            );
            #[allow(clippy::cast_possible_truncation)]
            let codes32 = vec![w as u32; 130];
            let mut scratch32 = vec![0; 130];
            assert!(
                laws::hilbert2_in_place_matches_per_key_u32(&codes32, &mut scratch32),
                "w={w:#x}"
            );
        }
    }

    proptest! {
        #[test]
        fn u64_codes(codes in prop::collection::vec(any::<u64>(), 0..=300)) {
            let mut scratch = vec![0; codes.len()];
            prop_assert!(laws::hilbert2_in_place_matches_per_key_u64(&codes, &mut scratch));
        }

        #[test]
        fn u32_codes(codes in prop::collection::vec(any::<u32>(), 0..=300)) {
            let mut scratch = vec![0; codes.len()];
            prop_assert!(laws::hilbert2_in_place_matches_per_key_u32(&codes, &mut scratch));
        }

        #[test]
        fn hilbert2_columns_any(points in prop::collection::vec((any::<u64>(), any::<u64>()), 0..=600)) {
            let xs: Vec<u64> = points.iter().map(|p| p.0).collect();
            let ys: Vec<u64> = points.iter().map(|p| p.1).collect();
            let (mut keys, mut out) = (vec![0; xs.len()], vec![0; 2 * xs.len()]);
            prop_assert!(laws::hilbert2_columns_match_per_point_u64(&xs, &ys, &mut keys, &mut out));
        }

        #[test]
        fn morton2_columns_any(points in prop::collection::vec((any::<u64>(), any::<u64>()), 0..=300)) {
            let xs: Vec<u64> = points.iter().map(|p| p.0).collect();
            let ys: Vec<u64> = points.iter().map(|p| p.1).collect();
            let (mut codes, mut out) = (vec![0; xs.len()], vec![0; 2 * xs.len()]);
            prop_assert!(laws::morton2_columns_match_per_point_u64(&xs, &ys, &mut codes, &mut out));
        }

        #[test]
        fn columns_3d_any(points in prop::collection::vec((any::<u64>(), any::<u64>(), any::<u64>()), 0..=700)) {
            let xs: Vec<u64> = points.iter().map(|p| p.0).collect();
            let ys: Vec<u64> = points.iter().map(|p| p.1).collect();
            let zs: Vec<u64> = points.iter().map(|p| p.2).collect();
            let (mut keys, mut out) = (vec![0; xs.len()], vec![0; 3 * xs.len()]);
            prop_assert!(laws::hilbert3_columns_match_per_point(&xs, &ys, &zs, &mut keys, &mut out));
        }

        #[test]
        fn u64_codes_3d(codes in prop::collection::vec(any::<u64>(), 0..=300)) {
            let mut scratch = vec![0; codes.len()];
            prop_assert!(laws::hilbert3_in_place_matches_per_key_u64(&codes, &mut scratch));
        }

        #[test]
        fn u32_codes_3d(codes in prop::collection::vec(any::<u32>(), 0..=300)) {
            let mut scratch = vec![0; codes.len()];
            prop_assert!(laws::hilbert3_in_place_matches_per_key_u32(&codes, &mut scratch));
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
    use hakmem::affine::Affine8;
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

        #[test]
        fn affine_matches_reference(m in any::<u64>(), add in any::<u8>(), w in any::<u64>()) {
            prop_assert!(laws::affine_matches_reference(Affine8::new(m, add), w));
        }

        #[test]
        fn affine_composes(a in any::<(u64, u8)>(), b in any::<(u64, u8)>(), x in any::<u8>()) {
            prop_assert!(laws::affine_composes(Affine8::new(a.0, a.1), Affine8::new(b.0, b.1), x));
        }

        #[test]
        fn affine_named_maps_match_ops(x in any::<u8>(), n in 0u32..12) {
            prop_assert!(laws::affine_named_maps_match_ops(x, n));
        }
    }
}

/// The Kindergarten constants. Every file gathers into a byte with the
/// derived factor. Every diagonal and antidiagonal gathers by column
/// onto the top rank with the a-file factor, exactly, the antidiagonal
/// against bit order; the literature's b-file factor lands one column
/// up and drops the h-file off the top, which is exact once that cell
/// is left out, as the six-inner-bits index does.
#[test]
fn kindergarten_gathers_are_exact() {
    use hakmem::prelude::*;
    const A_FILE: u64 = 0x0101_0101_0101_0101;
    const B_FILE: u64 = 0x0202_0202_0202_0202;
    let square = |r: u32, c: u32| 1u64 << (8 * r + c);
    for f in 0..8 {
        let file: u64 = (0..8).map(|r| square(r, f)).fold(0, |m, b| m | b);
        let factor = u64::gather_factor(file, 56).unwrap();
        assert!(laws::gather_is_exact(file, factor, 56), "file {f}");
        assert_eq!(factor, 0x0102_0408_1020_4080 >> f, "file {f}");
    }
    for d in -7i32..=7 {
        for anti in [false, true] {
            // Cells in bit order (rank ascending), with their columns.
            let cells: Vec<(u32, u32)> = (0..8i32)
                .filter_map(|r| {
                    let c = if anti { d + 7 - r } else { r + d };
                    (0..8)
                        .contains(&c)
                        .then(|| (r.cast_unsigned(), c.cast_unsigned()))
                })
                .collect();
            let mask = cells
                .iter()
                .map(|&(r, c)| square(r, c))
                .fold(0u64, |m, b| m | b);
            let c_min = cells.iter().map(|&(_, c)| c).min().unwrap();
            let target = 56 + c_min;
            let place = |i: u32| 56 + cells[i as usize].1;
            let factor = u64::gather_factor_by(mask, place).unwrap();
            assert_eq!(factor & !A_FILE, 0, "d={d} anti={anti}");
            assert!(
                laws::gather_is_exact_by(mask, factor, target, place),
                "d={d} anti={anti}"
            );
            assert!(
                laws::gather_is_exact_by(mask, A_FILE, target, place),
                "d={d} anti={anti} a-file"
            );
            if !anti {
                assert_eq!(u64::gather_factor(mask, target), Some(factor), "d={d}");
                assert!(laws::gather_is_exact(mask, A_FILE, target), "d={d} a-file");
            }
            // The b-file variant: one column up, h-file left out.
            let inner: Vec<(u32, u32)> = cells.iter().copied().filter(|&(_, c)| c < 7).collect();
            if inner.is_empty() {
                continue;
            }
            let mask = inner
                .iter()
                .map(|&(r, c)| square(r, c))
                .fold(0u64, |m, b| m | b);
            let target = 57 + inner.iter().map(|&(_, c)| c).min().unwrap();
            let place = |i: u32| 57 + inner[i as usize].1;
            assert!(
                laws::gather_is_exact_by(mask, B_FILE, target, place),
                "d={d} anti={anti} b-file"
            );
        }
    }
}

mod cover {
    use hakmem::laws;
    use proptest::prelude::*;

    /// Budgets from one range to more than any rectangle's exact cover.
    const BUDGETS: [usize; 7] = [1, 2, 3, 5, 16, 64, 2048];

    fn check(x: (u16, u16), y: (u16, u16)) {
        for budget in BUDGETS {
            let mut out = vec![(0u16, 0u16); budget];
            assert!(laws::morton2_cover_matches_cells(x, y, &mut out), "Morton {x:?} {y:?} {budget}");
            assert!(laws::hilbert2_cover_matches_cells(x, y, &mut out), "Hilbert {x:?} {y:?} {budget}");
        }
    }

    #[test]
    fn edges_and_corners() {
        let cases = [
            ((0, 255), (0, 255)),
            ((0, 0), (0, 0)),
            ((255, 255), (255, 255)),
            ((1, 254), (1, 254)),
            ((3, 200), (100, 101)),
            ((17, 17), (0, 255)),
            ((5, 4), (0, 9)),
            ((0, 400), (250, 300)),
            ((128, 127), (128, 127)),
        ];
        for (x, y) in cases {
            check(x, y);
        }
    }

    #[test]
    fn ties_at_the_threshold() {
        // A column one cell wide is 256 runs whose gaps repeat; budgets
        // between one and two times fewer put the threshold among equal
        // gaps, where only some of them may close.
        for x in [0u16, 7, 200] {
            for budget in (128..256).step_by(9) {
                let mut out = vec![(0u16, 0u16); budget];
                assert!(laws::morton2_cover_matches_cells((x, x), (0, 255), &mut out), "Morton {x} {budget}");
                assert!(laws::hilbert2_cover_matches_cells((x, x), (0, 255), &mut out), "Hilbert {x} {budget}");
            }
        }
    }

    #[test]
    fn intersects_corners() {
        let cases = [
            ((0u16, 65_535u16), (0u16, 255u16), (0u16, 255u16)),
            ((0, 0), (0, 0), (0, 0)),
            ((1, 1), (0, 0), (0, 0)),
            ((65_535, 65_535), (255, 255), (255, 255)),
            ((100, 50), (0, 255), (0, 255)),
            ((4, 11), (0, 1), (0, 1)),
            ((0, 65_535), (7, 3), (0, 9)),
            ((12_345, 12_346), (0, 300), (0, 300)),
        ];
        for (k, x, y) in cases {
            assert!(laws::morton2_intersects_matches_cells(k, x, y), "Morton {k:?} {x:?} {y:?}");
            assert!(laws::hilbert2_intersects_matches_cells(k, x, y), "Hilbert {k:?} {x:?} {y:?}");
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(96))]
        #[test]
        fn intersects_random(a in any::<u16>(), b in any::<u16>(), p in 0u16..300, q in 0u16..300, r in 0u16..300, s in 0u16..300) {
            let (k, x, y) = ((a.min(b), a.max(b)), (p.min(q), p.max(q)), (r.min(s), r.max(s)));
            prop_assert!(laws::morton2_intersects_matches_cells(k, x, y));
            prop_assert!(laws::hilbert2_intersects_matches_cells(k, x, y));
        }

        #[test]
        fn intersects_short_intervals(a in any::<u16>(), len in 0u16..64, p in 0u16..256, q in 0u16..256, r in 0u16..256, s in 0u16..256) {
            // Short intervals keep the answer from being always yes.
            let k = (a, a.saturating_add(len));
            let (x, y) = ((p.min(q), p.max(q)), (r.min(s), r.max(s)));
            prop_assert!(laws::morton2_intersects_matches_cells(k, x, y));
            prop_assert!(laws::hilbert2_intersects_matches_cells(k, x, y));
        }

        #[test]
        fn random_rectangles(a in 0u16..300, b in 0u16..300, c in 0u16..300, d in 0u16..300) {
            check((a.min(b), a.max(b)), (c.min(d), c.max(d)));
        }

        #[test]
        fn wide_rectangles(
            a in any::<u32>(), b in any::<u32>(), c in any::<u32>(), d in any::<u32>(),
            budget in 1usize..64,
            points in prop::collection::vec((any::<u32>(), any::<u32>()), 64),
        ) {
            let (x, y) = ((u64::from(a.min(b)), u64::from(a.max(b))), (u64::from(c.min(d)), u64::from(c.max(d))));
            // Corners, edges and the given points, clamped into the box
            // half of the time so most are inside.
            let mut pts: Vec<(u64, u64)> = vec![(x.0, y.0), (x.1, y.0), (x.0, y.1), (x.1, y.1)];
            for (i, &(p, q)) in points.iter().enumerate() {
                let (p, q) = (u64::from(p), u64::from(q));
                pts.push(if i % 2 == 0 { (p.clamp(x.0, x.1), q.clamp(y.0, y.1)) } else { (p, q) });
            }
            let mut out = vec![(0u64, 0u64); budget];
            prop_assert!(laws::morton2_cover_holds_points(x, y, &pts, &mut out));
            prop_assert!(laws::hilbert2_cover_holds_points(x, y, &pts, &mut out));
            prop_assert!(laws::morton2_intersects_agrees_with_cover(x, y, &pts, &mut out));
            prop_assert!(laws::hilbert2_intersects_agrees_with_cover(x, y, &pts, &mut out));
        }
    }
}
