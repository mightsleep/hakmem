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
