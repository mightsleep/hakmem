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
        for k in [8, 9, 255, u32::MAX] {
            assert!(laws::select_lowest_is_total(x, k), "x={x} k={k}");
        }
        for k in 0..8 {
            assert!(laws::select_lowest_is_total(x, k), "x={x} k={k}");
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
        for k in [0, 7, 15, 16, 17, 64, u32::MAX] {
            assert!(laws::select_lowest_is_total(x, k), "x={x} k={k}");
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

/// Every cell of every curve up to order 8 on `u16`, and every index of
/// the full 256 × 256 curve: the scan-and-fill kernels against the
/// textbook loops, and the path property at every step.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u16_hilbert_all_orders_all_cells() {
    for order in 0..=8u32 {
        let side = 1u32 << order;
        for x in 0..side {
            for y in 0..side {
                // Fits: side <= 256.
                #[allow(clippy::cast_possible_truncation)]
                let (x, y) = (x as u16, y as u16);
                assert!(
                    laws::hilbert_matches_reference(x, y, order),
                    "order={order} x={x} y={y}"
                );
                if order < 8 {
                    assert!(
                        laws::hilbert_order_laws(x, y, order),
                        "order={order} x={x} y={y}"
                    );
                }
            }
        }
    }
    for h in u16::MIN..=u16::MAX {
        assert!(laws::hilbert_consecutive_are_adjacent(h), "h={h}");
        assert!(laws::hilbert_roundtrip(h, h.rotate_left(7)), "h={h}");
    }
    for x in u8::MIN..=u8::MAX {
        for y in u8::MIN..=u8::MAX {
            assert!(laws::hilbert_matches_reference(x, y, 4), "x={x} y={y}");
            assert!(laws::hilbert_roundtrip(x, y), "x={x} y={y}");
        }
    }
}

/// Every index of the 32 × 32 × 32 curve on `u16`, and every cell of
/// every order up to 5: the 3D decode scan against the twelve-state
/// machine, the path property, the order rotation.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u16_hilbert3_all_indices_all_orders() {
    for h in 0..(1u16 << 15) {
        assert!(laws::hilbert3_matches_reference(h), "h={h}");
        assert!(laws::hilbert3_consecutive_are_adjacent(h), "h={h}");
    }
    for order in 0..5u32 {
        let side = 1u32 << order;
        for x in 0..side {
            for y in 0..side {
                for z in 0..side {
                    // Fits: side <= 16.
                    #[allow(clippy::cast_possible_truncation)]
                    let (x, y, z) = (x as u16, y as u16, z as u16);
                    assert!(
                        laws::hilbert3_order_laws(x, y, z, order),
                        "order={order} ({x},{y},{z})"
                    );
                    assert!(laws::hilbert3_roundtrip(x, y, z), "({x},{y},{z})");
                }
            }
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
        assert!(laws::suffix_xor_laws(x), "x={x}");
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
        assert!(laws::suffix_xor_laws(x), "x={x}");
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

/// Every pair of byte values in every lane of the SWAR carrier, every
/// shift, and every lane value through the table lookup. The lane ops
/// are lane-independent by construction, so pairs cover the space.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8x8_lanes_all_byte_pairs() {
    use hakmem::lanes::{Lanes, U8x8};
    let table = [
        0x00, 0x81, 0x7F, 0x10, 0xFF, 0x0F, 0x80, 0x01, 0x55, 0xAA, 0x3C, 0xC3, 0x02, 0x40, 0xFE,
        0x7E,
    ];
    for a in 0..=255u8 {
        // `a` in the even lanes, its complement in the odd ones, so the
        // borrow and carry between neighbouring lanes is exercised too.
        let x = U8x8::load(&[a, !a, a, !a, a, !a, a, !a]);
        for b in 0..=255u8 {
            let y = U8x8::load(&[b, b, !b, !b, b, !b, b, !b]);
            for n in 0..9 {
                assert!(
                    laws::lanes_match_reference(x, y, n, table),
                    "a={a:#04x} b={b:#04x} n={n}"
                );
            }
        }
    }
}

/// Two's-complement overflow from three sign bits, against the
/// standard library's checked arithmetic on the signed twin: every
/// pair of bytes, and every `u16` against the structured masks.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn signed_overflow_matches_checked_arithmetic() {
    for a in u8::MIN..=u8::MAX {
        for b in u8::MIN..=u8::MAX {
            let (x, y) = (a.cast_signed(), b.cast_signed());
            assert_eq!(
                a.signed_add_overflows(b),
                x.checked_add(y).is_none(),
                "a={a} b={b}"
            );
            assert_eq!(
                a.signed_sub_overflows(b),
                x.checked_sub(y).is_none(),
                "a={a} b={b}"
            );
            assert!(laws::signed_overflow_matches_sign_test(a, b), "a={a} b={b}");
        }
    }
    let masks = masks16();
    for a in u16::MIN..=u16::MAX {
        for &b in &masks {
            let (x, y) = (a.cast_signed(), b.cast_signed());
            assert_eq!(
                a.signed_add_overflows(b),
                x.checked_add(y).is_none(),
                "a={a} b={b}"
            );
            assert_eq!(
                a.signed_sub_overflows(b),
                x.checked_sub(y).is_none(),
                "a={a} b={b}"
            );
        }
    }
}

/// Every truth table on every pair of bytes, with a third operand from
/// a small structured set; and the named functions on all pairs.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_ternary_all_tables_all_pairs() {
    for table in u8::MIN..=u8::MAX {
        for a in u8::MIN..=u8::MAX {
            for b in u8::MIN..=u8::MAX {
                for c in [0x00, 0xFF, 0x0F, 0xF0, 0x55, 0xAA, a ^ b, a.wrapping_add(b)] {
                    assert!(
                        laws::ternary_is_truth_table(a, b, c, table),
                        "a={a} b={b} c={c} table={table:#04x}"
                    );
                }
            }
        }
    }
    for a in u8::MIN..=u8::MAX {
        for b in u8::MIN..=u8::MAX {
            assert!(laws::truth_table_names_the_function(
                a,
                b,
                a.wrapping_mul(b)
            ));
        }
    }
}

/// The GF(2) affine maps: the named ones on every byte and shift
/// count, and a spread of random maps on every byte, alone, composed,
/// and as lanes of the SWAR carrier.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn affine_all_bytes() {
    use hakmem::affine::Affine8;
    let mut maps = vec![
        Affine8::IDENTITY,
        Affine8::NOT,
        Affine8::ZERO,
        Affine8::REVERSE,
        Affine8::PARITY,
    ];
    for n in 0..9 {
        maps.extend([
            Affine8::shl(n),
            Affine8::shr(n),
            Affine8::sra(n),
            Affine8::rotl(n),
        ]);
    }
    let mut s: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..64 {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        // The low byte is as random as any.
        #[allow(clippy::cast_possible_truncation)]
        maps.push(Affine8::new(s, s as u8));
    }
    for x in u8::MIN..=u8::MAX {
        for n in 0..12 {
            assert!(laws::affine_named_maps_match_ops(x, n), "x={x} n={n}");
        }
        let word = (u64::from(x) * 0x0101_0101_0101_0101) ^ 0xF0E1_D2C3_B4A5_9687;
        for &a in &maps {
            assert!(laws::affine_matches_reference(a, word), "x={x} a={a:?}");
            for &b in &maps {
                assert!(laws::affine_composes(a, b, x), "x={x} a={a:?} b={b:?}");
            }
        }
    }
}

/// The carry-rippler on every `u8` pair and every `u8` mask, and every
/// `u16` mask with at most 12 bits; the strided gather theorem on every
/// `u8` parameter set.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn u8_u16_subsets_and_gathers() {
    for m in u8::MIN..=u8::MAX {
        assert!(laws::subsets_enumerate_each_once(m), "m={m}");
        for x in u8::MIN..=u8::MAX {
            assert!(laws::next_subset_is_increment_in_mask(x, m), "x={x} m={m}");
        }
    }
    for m in u16::MIN..=u16::MAX {
        if m.count_ones() <= 12 {
            assert!(laws::subsets_enumerate_each_once(m), "m={m}");
        }
    }
    for x in u8::MIN..=u8::MAX {
        for start in 0..8 {
            for stride in 1..8 {
                for k in 1..=8 {
                    for target in 0..8 {
                        assert!(
                            laws::strided_gather_is_exact(x, start, stride, k, target),
                            "x={x} start={start} stride={stride} k={k} target={target}"
                        );
                    }
                }
            }
        }
    }
}

/// `intersects` on the 16 × 16 grid of `u8`: every rectangle against
/// intervals of every start and five lengths. Four levels is a step of
/// three and a step of one, the step the wider tests never take.
#[test]
#[cfg_attr(debug_assertions, ignore = "exhaustive sweep: run with --release")]
fn intersects_u8_every_rectangle() {
    for x0 in 0..16u8 {
        for x1 in x0..16 {
            for y0 in 0..16u8 {
                for y1 in y0..16 {
                    for a in 0..=255u8 {
                        for len in [0u8, 1, 4, 17, 100] {
                            let k = (a, a.saturating_add(len));
                            let (x, y) = ((x0, x1), (y0, y1));
                            assert!(
                                laws::morton2_intersects_matches_cells(k, x, y),
                                "Morton {k:?} {x:?} {y:?}"
                            );
                            assert!(
                                laws::hilbert2_intersects_matches_cells(k, x, y),
                                "Hilbert {k:?} {x:?} {y:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}
