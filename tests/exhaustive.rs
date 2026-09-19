//! Exhaustive law checks on the narrow carriers.
//!
//! For `u8` every law is checked on every input (all pairs for binary
//! laws, all triples for `compact_composes`). For `u16` unary laws
//! and the `k`-indexed laws are exhaustive in `x`; binary laws sweep
//! all `x` against a structured set of masks. This is the
//! "verified catalogue for w ≤ 16" of doc 13 §9.1, and the proof that
//! the BMI2 and portable paths agree bit-for-bit.
//!
//! Debug builds skip these (two minutes of bit loops); run them as
//! `cargo test -p hakmem --release`, both plain and with
//! `RUSTFLAGS="-C target-feature=+bmi2"`.

use hakmem::laws;
use hakmem::prelude::*;

/// Structured masks for the `u16` binary sweeps: prefixes, dilations,
/// alternating patterns, and a deterministic pseudo-random tail.
fn masks16() -> Vec<u16> {
    let mut m: Vec<u16> = (0..=16).map(u16::low_ones).collect();
    m.extend([
        0xAAAA, 0x5555, 0x3333, 0x0F0F, 0x00FF, 0xFF00, 0x8001, 0x0180,
    ]);
    m.push(Dilated::<u16, 2>::mask());
    m.push(Dilated::<u16, 3>::mask());
    let mut s: u32 = 0x9E37_79B9;
    for _ in 0..40 {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        // Truncation is the point: a pseudo-random 16-bit mask.
        #[allow(clippy::cast_possible_truncation)]
        m.push(s as u16);
    }
    m
}

macro_rules! for_all {
    ($t:ty, $x:ident => $body:expr) => {
        for $x in <$t>::MIN..=<$t>::MAX {
            assert!(
                $body,
                concat!("law failed for ", stringify!($t), " x = {}"),
                $x
            );
        }
    };
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_unary_laws() {
    for_all!(u8, x => laws::run_starts_one_is_identity(x));
    for_all!(u8, x => laws::prefix_xor_after_delta_is_identity(x));
    for_all!(u8, x => laws::delta_after_prefix_xor_is_identity(x));
    for_all!(u8, x => laws::prefix_or_is_smear_from_first_set(x));
    for_all!(u8, x => laws::prefix_xor_matches_reference(x));
    for_all!(u8, x => laws::rank_full_is_popcount(x));
    for_all!(u8, x => laws::first_last_bracket_set_bits(x));
    for_all!(u8, x => laws::dilated_roundtrip::<u8, 2>(x));
    for_all!(u8, x => laws::dilated_roundtrip::<u8, 3>(x));
    for_all!(u8, x => laws::dilated_incr_is_add_one::<u8, 2>(x));
    for_all!(u8, x => laws::dilated_incr_is_add_one::<u8, 3>(x));
    for_all!(u8, x => laws::dilated_decr_after_incr_is_identity::<u8, 2>(x));
    for_all!(u8, x => laws::dilated_decr_after_incr_is_identity::<u8, 3>(x));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_indexed_laws() {
    for x in u8::MIN..=u8::MAX {
        for k in 1..8 {
            assert!(laws::run_starts_shrinks(x, k), "x={x} k={k}");
        }
        for k in 1..=8 {
            assert!(laws::run_starts_matches_reference(x, k), "x={x} k={k}");
        }
        for i in 0..8 {
            assert!(laws::rank_is_monotone(x, i), "x={x} i={i}");
            assert!(laws::select_is_rank_inverse(x, i), "x={x} k={i}");
            assert!(laws::select_matches_reference(x, i), "x={x} k={i}");
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_binary_laws_all_pairs() {
    for x in u8::MIN..=u8::MAX {
        for m in u8::MIN..=u8::MAX {
            assert!(laws::compact_matches_reference(x, m), "x={x} m={m}");
            assert!(laws::expand_matches_reference(x, m), "x={x} m={m}");
            assert!(laws::compact_expand_roundtrip(x, m), "x={x} m={m}");
            assert!(laws::expand_compact_roundtrip(x, m), "x={x} m={m}");
            assert!(laws::compact_preserves_popcount(x, m), "x={x} m={m}");
            assert!(laws::dilated_add_is_add::<u8, 2>(x, m), "a={x} b={m}");
            assert!(laws::dilated_add_is_add::<u8, 3>(x, m), "a={x} b={m}");
            assert!(laws::morton_roundtrip(x, m), "x={x} y={m}");
            assert!(laws::morton_steps_are_unit_moves(x, m), "x={x} y={m}");
            assert!(
                laws::morton_aligned_block_is_contiguous(x, m),
                "x={x} y={m}"
            );
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_compact_composes_all_triples() {
    for x in u8::MIN..=u8::MAX {
        for m in u8::MIN..=u8::MAX {
            for n in u8::MIN..=u8::MAX {
                assert!(laws::compact_composes(x, m, n), "x={x} m={m} n={n}");
            }
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u16_unary_laws() {
    for_all!(u16, x => laws::run_starts_one_is_identity(x));
    for_all!(u16, x => laws::prefix_xor_after_delta_is_identity(x));
    for_all!(u16, x => laws::delta_after_prefix_xor_is_identity(x));
    for_all!(u16, x => laws::prefix_or_is_smear_from_first_set(x));
    for_all!(u16, x => laws::prefix_xor_matches_reference(x));
    for_all!(u16, x => laws::rank_full_is_popcount(x));
    for_all!(u16, x => laws::first_last_bracket_set_bits(x));
    for_all!(u16, x => laws::dilated_roundtrip::<u16, 2>(x));
    for_all!(u16, x => laws::dilated_roundtrip::<u16, 3>(x));
    for_all!(u16, x => laws::dilated_incr_is_add_one::<u16, 2>(x));
    for_all!(u16, x => laws::dilated_incr_is_add_one::<u16, 3>(x));
    for_all!(u16, x => laws::dilated_decr_after_incr_is_identity::<u16, 2>(x));
    for_all!(u16, x => laws::dilated_decr_after_incr_is_identity::<u16, 3>(x));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u16_indexed_laws() {
    for x in u16::MIN..=u16::MAX {
        for k in 1..16 {
            assert!(laws::run_starts_shrinks(x, k), "x={x} k={k}");
        }
        for k in 1..=16 {
            assert!(laws::run_starts_matches_reference(x, k), "x={x} k={k}");
        }
        for i in 0..16 {
            assert!(laws::rank_is_monotone(x, i), "x={x} i={i}");
            assert!(laws::select_is_rank_inverse(x, i), "x={x} k={i}");
            assert!(laws::select_matches_reference(x, i), "x={x} k={i}");
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u16_binary_laws_all_x_structured_masks() {
    let masks = masks16();
    for x in u16::MIN..=u16::MAX {
        for &m in &masks {
            assert!(laws::compact_matches_reference(x, m), "x={x} m={m}");
            assert!(laws::expand_matches_reference(x, m), "x={x} m={m}");
            assert!(laws::compact_expand_roundtrip(x, m), "x={x} m={m}");
            assert!(laws::expand_compact_roundtrip(x, m), "x={x} m={m}");
            assert!(laws::compact_preserves_popcount(x, m), "x={x} m={m}");
            assert!(laws::dilated_add_is_add::<u16, 2>(x, m), "a={x} b={m}");
            assert!(laws::dilated_add_is_add::<u16, 3>(x, m), "a={x} b={m}");
            for &n in &masks {
                assert!(laws::compact_composes(x, m, n), "x={x} m={m} n={n}");
            }
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u16_morton_all_coordinates() {
    // Both coordinates fit in 8 bits: 65 536 cells, every one visited.
    for x in 0..=0xFFu16 {
        for y in 0..=0xFFu16 {
            assert!(laws::morton_roundtrip(x, y), "x={x} y={y}");
            assert!(laws::morton_steps_are_unit_moves(x, y), "x={x} y={y}");
            assert!(
                laws::morton_aligned_block_is_contiguous(x, y),
                "x={x} y={y}"
            );
        }
    }
}

/// `select_broadword64` on every 16-bit pattern at every byte-pair
/// offset of the word, every rank: exercises both compare-and-count
/// phases at all lane positions, independent of the target's PDEP.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u64_broadword_select_all_u16_patterns_all_offsets() {
    use hakmem::word::select_broadword64;
    for pat in u16::MIN..=u16::MAX {
        for shift in [0u32, 8, 16, 24, 32, 40, 48] {
            let x = u64::from(pat) << shift;
            for k in 0..x.count_ones() {
                let want = laws::reference::select(x, k).unwrap();
                assert_eq!(select_broadword64(x, k), want, "x={x:#x} k={k}");
            }
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_u16_swar_and_reduces() {
    for x in u8::MIN..=u8::MAX {
        assert!(laws::small_reduces_match_reference(x), "x={x}");
        assert!(laws::positions_enumerate_set_bits(x), "x={x}");
        for b in u8::MIN..=u8::MAX {
            for n in [1u8, 2, 64, 127, 128] {
                assert!(
                    laws::swar_lanes_match_reference(x, b, n),
                    "x={x} b={b} n={n}"
                );
            }
        }
    }
    for x in u16::MIN..=u16::MAX {
        assert!(laws::small_reduces_match_reference(x), "x={x}");
        assert!(laws::positions_enumerate_set_bits(x), "x={x}");
        for b in [0u8, 0x41, 0x80, 0xFF] {
            for n in [1u8, 32, 128] {
                assert!(
                    laws::swar_lanes_match_reference(x, b, n),
                    "x={x} b={b} n={n}"
                );
            }
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_slice_pairs_all_i_all_k() {
    for a in u8::MIN..=u8::MAX {
        for b in u8::MIN..=u8::MAX {
            let words = [a, b];
            for i in 0..=17 {
                for k in 1..=8 {
                    assert!(
                        laws::slice_ops_match_reference(&words, i, k),
                        "a={a} b={b} i={i} k={k}"
                    );
                }
            }
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_fills_and_delta_swaps_all_inputs() {
    for x in u8::MIN..=u8::MAX {
        for p in u8::MIN..=u8::MAX {
            for s in 1..8 {
                assert!(laws::fills_match_reference(x, p, s), "x={x} p={p} s={s}");
                assert!(laws::delta_swap_is_involution(x, p, s), "x={x} m={p} s={s}");
            }
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_u16_catalogue_all_inputs() {
    for x in u8::MIN..=u8::MAX {
        assert!(laws::next_same_popcount_matches_reference(x), "x={x}");
        assert!(laws::pow2_helpers_match_reference(x), "x={x}");
        assert!(laws::longest_run_matches_reference(x), "x={x}");
        assert!(laws::gray_code_laws(x), "x={x}");
        for c in [false, true] {
            assert!(laws::find_escaped_matches_reference(&[x], c), "x={x} c={c}");
        }
        for y in u8::MIN..=u8::MAX {
            assert!(
                laws::basics_match_reference(x, y, x.rotate_left(3) ^ y),
                "x={x} y={y}"
            );
            for c in [false, true] {
                assert!(
                    laws::find_escaped_matches_reference(&[x, y], c),
                    "x={x} y={y} c={c}"
                );
            }
        }
    }
    for x in u16::MIN..=u16::MAX {
        assert!(laws::pow2_helpers_match_reference(x), "x={x}");
        assert!(laws::longest_run_matches_reference(x), "x={x}");
        assert!(laws::gray_code_laws(x), "x={x}");
        assert!(
            laws::basics_match_reference(x, x.rotate_left(5), x.rotate_left(11)),
            "x={x}"
        );
        for c in [false, true] {
            assert!(laws::find_escaped_matches_reference(&[x], c), "x={x} c={c}");
        }
    }
    // Gosper on u16: walk every 4-subset in order and check the count.
    let mut count = 0u32;
    let mut cur = Some(0b1111u16);
    while let Some(x) = cur {
        count += 1;
        cur = x.next_same_popcount();
    }
    assert_eq!(count, 1820, "C(16, 4)");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_composition_and_grid_all_inputs() {
    for x in u8::MIN..=u8::MAX {
        for a in 1..=8u32 {
            for b in 1..=(9 - a) {
                assert!(laws::run_starts_composes(x, a, b), "x={x} a={a} b={b}");
            }
        }
        if x != u8::MAX {
            assert!(laws::gray_successor_flips_lowest_set(x), "x={x}");
        }
        for y in u8::MIN..=u8::MAX {
            assert!(laws::zero_bytes_turns_or_into_and(x, y), "x={x} y={y}");
            for k in 1..=8 {
                assert!(laws::run_starts_preserves_and(x, y, k), "x={x} y={y} k={k}");
            }
            // Every 2-row u8 grid, every block shape with a composable pair.
            let rows = [x, y];
            for (w1, h1, w2, h2) in [
                (1, 1, 1, 1),
                (2, 1, 2, 2),
                (3, 2, 2, 1),
                (4, 1, 5, 1),
                (2, 2, 1, 1),
                (8, 1, 1, 2),
            ] {
                assert!(
                    laws::block_starts_laws(&rows, w1, h1, w2, h2),
                    "rows={rows:?} {w1}x{h1} {w2}x{h2}"
                );
            }
        }
    }
}

/// The loop the broadword `pext` replaced, kept as its reference.
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

/// The loop the broadword `pdep` replaced, kept as its reference.
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

/// The broadword definitions themselves, whatever `Word::pext` compiles
/// to on this target: every `u8` pair, every `u16` against the mask set.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_u16_broadword_compress_expand_match_loop() {
    use hakmem::word::{compress_broadword, expand_broadword};
    for x in u8::MIN..=u8::MAX {
        for m in u8::MIN..=u8::MAX {
            assert_eq!(
                compress_broadword(x, m),
                compress_loop(x, m),
                "compress x={x:#b} m={m:#b}"
            );
            assert_eq!(
                expand_broadword(x, m),
                expand_loop(x, m),
                "expand x={x:#b} m={m:#b}"
            );
        }
    }
    let masks = masks16();
    for x in u16::MIN..=u16::MAX {
        for &m in &masks {
            assert_eq!(
                compress_broadword(x, m),
                compress_loop(x, m),
                "compress x={x:#b} m={m:#b}"
            );
            assert_eq!(
                expand_broadword(x, m),
                expand_loop(x, m),
                "expand x={x:#b} m={m:#b}"
            );
        }
    }
}
