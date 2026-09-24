//! Small deterministic checks aimed at Miri: every hardware-backed
//! primitive (the only `unsafe` in the crate) on every carrier, against
//! the bit-loop reference. Runs in seconds under the interpreter; the
//! full law suite lives in `tests/laws.rs`.

use hakmem::laws::reference;
use hakmem::{Bits, Wide, Word};

const SAMPLES: [u64; 6] = [
    0,
    1,
    0x8000_0000_0000_0000,
    0xDEAD_BEEF_CAFE_F00D,
    0x5555_5555_5555_5555,
    u64::MAX,
];

fn check<W: Word>(x: W, m: W) {
    assert_eq!(x.pext(m), reference::pext(x, m));
    assert_eq!(x.pdep(m), reference::pdep(x, m));
    assert_eq!(x.xor_scan(), reference::prefix_xor(x));
    assert_eq!(x.xor_scan_down(), reference::prefix_xor_from_top(x));
    for k in 0..x.count_ones() {
        assert_eq!(x.select(k), reference::select(x, k));
    }
}

#[test]
fn primitives_on_every_carrier() {
    for &a in &SAMPLES {
        for &b in &SAMPLES {
            // Truncation is the point: the same bit patterns on every width.
            #[allow(clippy::cast_possible_truncation)]
            {
                check(a as u8, b as u8);
                check(a as u16, b as u16);
                check(a as u32, b as u32);
            }
            check(a, b);
            check(
                u128::from(a) << 64 | u128::from(b),
                u128::from(b) << 64 | u128::from(a),
            );
            check(
                Wide::<3>::from_limbs([a, b, a ^ b]),
                Wide::<3>::from_limbs([b, a, !a]),
            );
        }
    }
}

/// The vector carrier's intrinsics (SSSE3 when enabled) on a few inputs,
/// against the per-lane definitions and the SWAR halves.
#[test]
fn lanes_hardware_paths() {
    use hakmem::lanes::{Lanes, U8x8, U8x16};
    use hakmem::laws;
    let table = [
        9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0x80, 0x7F, 0xFF, 0x10, 0x20, 0x30,
    ];
    let a = U8x16::load(b"hello, world!\t\n\r");
    let b = U8x16::splat(b'l');
    for n in [0, 1, 4, 7, 8] {
        assert!(laws::lanes_match_reference(a, b, n, table), "n={n}");
        assert!(
            laws::u8x16_agrees_with_halves(
                (0x8000_0000_0000_0001, 0xFF00_7F80_0102_0304),
                (0x0102_0304_0506_0708, u64::MAX),
                n,
                table
            ),
            "n={n}"
        );
    }
    assert!(laws::lut16_composes(a, table, [0xFF; 16]));
    assert!(laws::lanes_match_reference(
        U8x8::load(b"hello, w"),
        U8x8::splat(0x80),
        3,
        table
    ));
}

/// The batch Hilbert kernels (VBMI `vpermb` / `vpermi2b`, NEON `tbl`)
/// against the per-key conversions, on lengths around every batch size.
#[test]
fn hilbert_batch_paths() {
    use hakmem::laws;
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for n in [0usize, 1, 15, 16, 17, 32, 63, 64, 65, 130] {
        let codes: Vec<u64> = (0..n)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                s
            })
            .collect();
        // Truncation is the point: the low halves as u32 codes.
        #[allow(clippy::cast_possible_truncation)]
        let codes32: Vec<u32> = codes.iter().map(|&c| c as u32).collect();
        let (mut a, mut b) = (vec![0; n], vec![0; n]);
        assert!(
            laws::hilbert2_in_place_matches_per_key_u64(&codes, &mut a),
            "n={n}"
        );
        assert!(
            laws::hilbert3_in_place_matches_per_key_u64(&codes, &mut a),
            "n={n}"
        );
        let (mut keys, mut out) = (vec![0; n], vec![0; 3 * n]);
        let rotated: Vec<u64> = codes.iter().map(|c| c.rotate_left(21)).collect();
        let shifted: Vec<u64> = codes.iter().map(|c| c >> 7).collect();
        assert!(
            laws::hilbert3_columns_match_per_point(&codes, &rotated, &shifted, &mut keys, &mut out),
            "n={n}"
        );
        let (mut z_codes, mut z_back) = (vec![0; n], vec![0; 2 * n]);
        assert!(
            laws::morton2_columns_match_per_point_u64(&codes, &shifted, &mut z_codes, &mut z_back),
            "n={n}"
        );
        let (mut h_keys, mut h_back) = (vec![0; n], vec![0; 2 * n]);
        assert!(
            laws::hilbert2_columns_match_per_point_u64(&rotated, &codes, &mut h_keys, &mut h_back),
            "n={n}"
        );
        let (mut z_codes32, mut z_back32) = (vec![0; n], vec![0; 2 * n]);
        #[allow(clippy::cast_possible_truncation)]
        let shifted32: Vec<u32> = shifted.iter().map(|&c| c as u32).collect();
        assert!(
            laws::morton2_columns_match_per_point_u32(&codes32, &shifted32, &mut z_codes32, &mut z_back32),
            "n={n}"
        );
        assert!(
            laws::hilbert2_in_place_matches_per_key_u32(&codes32, &mut b),
            "n={n}"
        );
        assert!(
            laws::hilbert3_in_place_matches_per_key_u32(&codes32, &mut b),
            "n={n}"
        );
        for order in [0, 1, 7, 10] {
            assert!(
                laws::hilbert2_in_place_order_matches_per_key_u64(&codes, &mut a, order),
                "n={n} order={order}"
            );
            assert!(
                laws::hilbert3_in_place_order_matches_per_key_u64(&codes, &mut a, order),
                "n={n} order={order}"
            );
            assert!(
                laws::hilbert2_in_place_order_matches_per_key_u32(&codes32, &mut b, order),
                "n={n} order={order}"
            );
            assert!(
                laws::hilbert3_in_place_order_matches_per_key_u32(&codes32, &mut b, order),
                "n={n} order={order}"
            );
        }
    }
}
