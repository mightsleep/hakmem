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
    let even = W::splat_byte(0x55);
    assert_eq!(
        cpu.unzip(x),
        (reference::pext(x, even), reference::pext(x, even.shl(1))),
        "{cpu:?} unzip {x:x}"
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

/// The slice methods under every level agree with `Portable`, and the
/// plain ones (which pick a level per call) with both.
fn slices_of<I: Isa>(cpu: I, xs: &[u64]) {
    use hakmem::Words;
    let p = isa::Portable;
    assert_eq!(xs.count_ones_in(cpu), xs.count_ones_in(p), "{cpu:?}");
    assert_eq!(xs.count_ones(), xs.count_ones_in(p));
    let bits = xs.len() * 64;
    for i in [
        0,
        1,
        63,
        64,
        65,
        bits / 2,
        bits.saturating_sub(1),
        bits,
        bits + 7,
    ] {
        assert_eq!(xs.rank_in(i, cpu), xs.rank_in(i, p), "{cpu:?} rank {i}");
        assert_eq!(xs.rank(i), xs.rank_in(i, p));
    }
    let n = xs.count_ones_in(p);
    for k in [0, 1, n / 3, n / 2, n.saturating_sub(1), n, n + 1] {
        assert_eq!(
            xs.select_in(k, cpu),
            xs.select_in(k, p),
            "{cpu:?} select {k}"
        );
        assert_eq!(xs.select(k), xs.select_in(k, p));
    }
    let (mut a, mut b) = (Vec::new(), Vec::new());
    xs.for_each_position_in(cpu, |i| a.push(i));
    xs.for_each_position(|i| b.push(i));
    assert_eq!(a, xs.positions().collect::<Vec<_>>(), "{cpu:?}");
    assert_eq!(b, a);
}

#[test]
fn every_level_agrees_on_slices() {
    let xs: Vec<u64> = samples().take(37).collect();
    for level in isa::available() {
        for len in [0, 1, 2, 5, 37] {
            hakmem::dispatch!(level, |cpu| slices_of(cpu, &xs[..len]));
        }
    }
}

/// `Rank9` queries under every level agree with the slice scans, on a
/// dense and a sparse bitmap, with and without the select inventory.
fn rank9_of<I: Isa>(cpu: I, bits: &[u64]) {
    use hakmem::Words;
    use hakmem::rank9::Rank9;
    let mut counts = vec![0; Rank9::counts_len(bits.len())];
    let mut select = vec![0; Rank9::select_len(bits.len())];
    let dir = Rank9::build(bits, &mut counts, &mut select);
    let mut counts2 = vec![0; Rank9::counts_len(bits.len())];
    let rank_only = Rank9::build(bits, &mut counts2, &mut []);
    let n = bits.count_ones();
    let len = bits.len() * 64;
    for i in (0..=len + 64).step_by(61) {
        assert_eq!(dir.rank_in(i, cpu), bits.rank(i), "{cpu:?} rank {i}");
        assert_eq!(dir.rank(i), bits.rank(i));
    }
    for k in (0..n + 2).step_by(7) {
        assert_eq!(dir.select_in(k, cpu), bits.select(k), "{cpu:?} select {k}");
        assert_eq!(
            rank_only.select_in(k, cpu),
            bits.select(k),
            "{cpu:?} search {k}"
        );
        assert_eq!(dir.select(k), bits.select(k));
    }
}

#[test]
fn every_level_agrees_on_rank9() {
    let dense: Vec<u64> = samples().take(150).collect();
    let sparse: Vec<u64> = samples().take(150).map(|x| 1 << (x % 64)).collect();
    for level in isa::available() {
        hakmem::dispatch!(level, |cpu| rank9_of(cpu, &dense));
        hakmem::dispatch!(level, |cpu| rank9_of(cpu, &sparse));
    }
}
