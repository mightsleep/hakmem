//! The 3D curve against the tables that define it: rawrunprotected's
//! `mortonToHilbert3D` / `hilbertToMorton3D` (`rawrunprotected/hilbert_curves`
//! on GitHub, public domain), a twelve-state machine walked with
//! two 96-byte tables, `transform | triple` to `transform | triple`. The
//! crate's algebra was read off these tables; this is the check that
//! it was read right, on every cell of every order up to 5 and on
//! samples of the 21-level `u64` curve.

use hakmem::prelude::*;

#[rustfmt::skip]
const MORTON_TO_HILBERT: [u8; 96] = [
    48, 33, 27, 34, 47, 78, 28, 77, 66, 29, 51, 52, 65, 30, 72, 63,
    76, 95, 75, 24, 53, 54, 82, 81, 18,  3, 17, 80, 61,  4, 62, 15,
     0, 59, 71, 60, 49, 50, 86, 85, 84, 83,  5, 90, 79, 56,  6, 89,
    32, 23,  1, 94, 11, 12,  2, 93, 42, 41, 13, 14, 35, 88, 36, 31,
    92, 37, 87, 38, 91, 74,  8, 73, 46, 45,  9, 10,  7, 20, 64, 19,
    70, 25, 39, 16, 69, 26, 44, 43, 22, 55, 21, 68, 57, 40, 58, 67,
];

#[rustfmt::skip]
const HILBERT_TO_MORTON: [u8; 96] = [
    48, 33, 35, 26, 30, 79, 77, 44, 78, 68, 64, 50, 51, 25, 29, 63,
    27, 87, 86, 74, 72, 52, 53, 89, 83, 18, 16,  1,  5, 60, 62, 15,
     0, 52, 53, 57, 59, 87, 86, 66, 61, 95, 91, 81, 80,  2,  6, 76,
    32,  2,  6, 12, 13, 95, 91, 17, 93, 41, 40, 36, 38, 10, 11, 31,
    14, 79, 77, 92, 88, 33, 35, 82, 70, 10, 11, 23, 21, 41, 40,  4,
    19, 25, 29, 47, 46, 68, 64, 34, 45, 60, 62, 71, 67, 18, 16, 49,
];

/// `transformCurve`, on a `u64` so that 21 levels fit.
fn transform(input: u64, levels: u32, table: &[u8; 96]) -> u64 {
    let mut state = 0usize;
    let mut out = 0u64;
    for level in (0..levels).rev() {
        let entry = usize::from(table[state | ((input >> (3 * level)) & 7) as usize]);
        out = (out << 3) | (entry & 7) as u64;
        state = entry & !7;
    }
    out
}

fn table_encode(x: u64, y: u64, z: u64, levels: u32) -> u64 {
    transform(
        Morton3::<u64>::encode(x, y, z).code(),
        levels,
        &MORTON_TO_HILBERT,
    )
}

fn table_decode(h: u64, levels: u32) -> (u64, u64, u64) {
    Morton3::<u64>::from_code(transform(h, levels, &HILBERT_TO_MORTON)).decode()
}

#[test]
fn u16_every_cell_every_order_matches_the_tables() {
    for order in 0..=5u32 {
        let side = 1u64 << order;
        for cx in 0..side {
            for cy in 0..side {
                for cz in 0..side {
                    let expect = table_encode(cx, cy, cz, order);
                    // Fits: side <= 32.
                    #[allow(clippy::cast_possible_truncation)]
                    let cell = (cx as u16, cy as u16, cz as u16);
                    let index = Hilbert3::encode_order(cell.0, cell.1, cell.2, order);
                    assert_eq!(
                        u64::from(index.index()),
                        expect,
                        "order={order} ({cx},{cy},{cz})"
                    );
                    assert_eq!(index.decode_order(order), cell);
                }
            }
        }
        for h in 0..(1u64 << (3 * order)) {
            let (x, y, z) = table_decode(h, order);
            #[allow(clippy::cast_possible_truncation)]
            let got = Hilbert3::from_index(h as u16).decode_order(order);
            assert_eq!(
                (u64::from(got.0), u64::from(got.1), u64::from(got.2)),
                (x, y, z),
                "order={order} h={h}"
            );
        }
    }
}

#[test]
fn u64_samples_match_the_tables_at_21_levels() {
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    for _ in 0..20_000 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let index = seed & ((1 << 63) - 1);
        assert_eq!(
            Hilbert3::from_index(index).decode(),
            table_decode(index, 21),
            "h={index:#x}"
        );
        let (cx, cy, cz) = (
            seed & 0x1F_FFFF,
            (seed >> 21) & 0x1F_FFFF,
            (seed >> 42) & 0x1F_FFFF,
        );
        assert_eq!(
            Hilbert3::encode(cx, cy, cz).index(),
            table_encode(cx, cy, cz, 21),
            "({cx},{cy},{cz})"
        );
    }
}
