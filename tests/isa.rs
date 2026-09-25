//! Every level this machine has, in one run: the primitives with each
//! token's instructions against the bit-loop reference, on every carrier.
//! Where a build used to see one path per `RUSTFLAGS`, a Zen 5 now checks
//! `Portable` and `X86V3` together, and CI whatever its runner has.

use hakmem::isa::{self, Isa};
use hakmem::laws::reference;
use hakmem::{Wide, Word};

/// xorshift64, and the corner cases in front.
fn samples() -> impl Iterator<Item = u64> {
    let mut s = 0x9E37_79B9_7F4A_7C15_u64;
    [0, 1, u64::MAX, 0x8000_0000_0000_0000, 0x5555_5555_5555_5555]
        .into_iter()
        .chain(core::iter::from_fn(move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            Some(s)
        }))
        .take(200)
}

fn primitives<I: Isa, W: Word>(cpu: I, x: W, m: W) {
    assert_eq!(
        cpu.pext(x, m),
        reference::pext(x, m),
        "{cpu:?} pext {x:x} {m:x}"
    );
    assert_eq!(
        cpu.pdep(x, m),
        reference::pdep(x, m),
        "{cpu:?} pdep {x:x} {m:x}"
    );
    assert_eq!(
        cpu.xor_scan(x),
        reference::prefix_xor(x),
        "{cpu:?} xor_scan {x:x}"
    );
    assert_eq!(
        cpu.xor_scan_down(x),
        reference::prefix_xor_from_top(x),
        "{cpu:?} xor_scan_down {x:x}"
    );
    for k in [
        0,
        1,
        x.count_ones().saturating_sub(1),
        x.count_ones(),
        W::BITS,
    ] {
        assert_eq!(
            cpu.select(x, k),
            reference::select(x, k),
            "{cpu:?} select {x:x} {k}"
        );
    }
}

fn every_carrier<I: Isa>(cpu: I) {
    let xs: Vec<u64> = samples().collect();
    for pair in xs.windows(3) {
        let [a, b, c] = [pair[0], pair[1], pair[2]];
        // Truncation is the point: the same bit patterns on every width.
        #[allow(clippy::cast_possible_truncation)]
        {
            primitives(cpu, a as u8, b as u8);
            primitives(cpu, a as u16, b as u16);
            primitives(cpu, a as u32, b as u32);
        }
        primitives(cpu, a, b);
        primitives(
            cpu,
            u128::from(a) << 64 | u128::from(c),
            u128::from(b) << 64 | u128::from(a),
        );
        primitives(
            cpu,
            Wide::<3>::from_limbs([a, b, c]),
            Wide::<3>::from_limbs([c, a, b]),
        );
    }
}

#[test]
fn every_level_agrees_with_the_reference() {
    let mut seen = 0;
    for level in isa::available() {
        hakmem::dispatch!(level, |cpu| every_carrier(cpu));
        seen += 1;
    }
    assert!(seen >= 1);
}

/// The plain methods are `Native`, and `Native` is a token like any other.
#[test]
fn native_is_the_plain_methods() {
    for x in samples() {
        let m = x.rotate_left(17);
        assert_eq!(isa::Native.pext(x, m), x.pext(m));
        assert_eq!(isa::Native.select(x, 3), hakmem::Bits::select(x, 3));
    }
}

/// The lanes of every level against the per-lane definitions: in a build
/// without flags this is the only place the SSSE3 carrier runs.
fn lanes_of<I: Isa>(cpu: I) {
    use hakmem::lanes::Lanes;
    use hakmem::laws;
    let table = [
        9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0x80, 0x7F, 0xFF, 0x10, 0x20, 0x30,
    ];
    let texts: [&[u8; 16]; 3] = [b"hello, world!\t\n\r", b"{\"a\": [1, 2]}   ", &[0xFF; 16]];
    for x in texts {
        for y in texts {
            let (a, b) = (I::U8x16::load(cpu, x), I::U8x16::load(cpu, y));
            for n in 0..9 {
                assert!(laws::lanes_match_reference(a, b, n, table), "{cpu:?} n={n}");
            }
            assert!(laws::lut16_composes(a, table, [0xFF; 16]), "{cpu:?}");
            assert_eq!(a.isa(), cpu);
        }
    }
}

#[test]
fn every_level_has_lawful_lanes() {
    for level in isa::available() {
        hakmem::dispatch!(level, |cpu| lanes_of(cpu));
    }
}
