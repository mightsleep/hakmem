//! The claims the README makes about instructions, checked in the asm.
//!
//! Every `#[no_mangle]` function here is one instantiation to look at;
//! the kernels, not being generic, are looked at in hakmem's own asm. A
//! `CHECK-LABEL` names a function, and the lines after it are
//! [FileCheck](https://llvm.org/docs/CommandGuide/FileCheck.html)
//! directives over that function alone (`nix/codegen.nix` cuts the asm
//! into functions and puts them in this file's order). The prefix is the
//! build: `CHECK` holds in every cell, `BMI2` where the cell has
//! `+bmi2,+pclmulqdq,+ssse3,+avx2`, `PORTABLE` in the build without flags.
//!
//! The same functions, as counts of each mnemonic, are the snapshot next
//! to this crate: a change in codegen is a diff in review, not a line on
//! the benchmark graph three weeks later.

use hakmem::lanes::{Lanes, U8x16};
use hakmem::{Bits, Hilbert2};

// CHECK-LABEL: cg_compact_u64:
// BMI2: pext
// PORTABLE-NOT: pext
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_compact_u64(x: u64, mask: u64) -> u64 {
    x.compact(mask)
}

// CHECK-LABEL: cg_expand_u64:
// BMI2: pdep
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_expand_u64(x: u64, mask: u64) -> u64 {
    x.expand(mask)
}

// The k-th set bit is one PDEP of `1 << k` and a TZCNT, which without
// `+bmi1` LLVM spells `rep bsf`: the same bytes, TZCNT on anything with
// BMI1, and the input is never zero here.
// CHECK-LABEL: cg_select_u64:
// BMI2: pdep
// BMI2: {{tzcnt|rep[[:space:]]+bsf}}
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_select_u64(x: u64, k: u32) -> Option<u32> {
    x.select(k)
}

// A carry-less multiply by all ones.
// CHECK-LABEL: cg_prefix_xor_u64:
// BMI2: pclmulqdq
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_prefix_xor_u64(x: u64) -> u64 {
    x.prefix_xor()
}

// Straight-line: no calls, no panics, no loop to speak of.
// CHECK-LABEL: cg_hilbert2_encode_u64:
// CHECK-NOT: call
// CHECK-NOT: panic
#[unsafe(no_mangle)]
pub fn cg_hilbert2_encode_u64(x: u64, y: u64) -> u64 {
    Hilbert2::<u64>::encode(x, y).index()
}

// Sixteen bytes to a mask: a compare and a PMOVMSKB with SSSE3.
// CHECK-LABEL: cg_quote_mask:
// BMI2: pcmpeqb
// BMI2: pmovmskb
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_quote_mask(block: &[u8; 16]) -> u16 {
    U8x16::load(block).cmp_eq(U8x16::splat(b'"')).to_bitmask()
}

// The 3D batch with AVX2 in the build: the kernel inlined, two PSHUFB a
// level. It calls CPUID once, the VBMI kernel when there is one, the
// per-key conversion for the tail and a bounds check's panic; never
// memcpy or memset, which is how a kernel quietly copies its batch.
// CHECK-LABEL: <hakmem::hilbert3::Hilbert3<u64>>::from_morton_in_place:
// CHECK-NOT: {{call.*mem(cpy|set|move)}}
// BMI2: vpshufb
// CHECK-NOT: {{call.*mem(cpy|set|move)}}

// The token path, the point of `hakmem::isa`: in a build without flags,
// `dispatch!` runs the body under x86-64-v3 and the primitive in it is
// the instruction, inlined, where `x.compact(m)` above is broadword.
// CHECK-LABEL: <hakmem::isa::x86v3::X86V3 as hakmem::isa::Isa>::run::trampoline::<u64, hakmem_codegen::cg_dispatch_gather::{closure#1}>:
// CHECK: pext
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_dispatch_gather(xs: &[u64], mask: u64) -> u64 {
    use hakmem::isa::Isa;
    hakmem::dispatch!(|cpu| xs.iter().fold(0, |a, &x| a ^ cpu.pext(x, mask)))
}

// Sixteen lanes under a token: in a build without flags the X86V3
// carrier is an XMM register, a compare and a PMOVMSKB, where
// `cg_quote_mask` above is SWAR arithmetic.
// CHECK-LABEL: <hakmem::isa::x86v3::X86V3 as hakmem::isa::Isa>::run::trampoline::<u16, hakmem_codegen::cg_dispatch_quote_mask::{closure#1}>:
// CHECK: pcmpeqb
// CHECK: pmovmskb
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_dispatch_quote_mask(block: &[u8; 16]) -> u16 {
    #[inline]
    fn quotes<I: hakmem::isa::Isa>(cpu: I, block: &[u8; 16]) -> u16 {
        let lanes = I::U8x16::load(cpu, block);
        lanes.cmp_eq(I::U8x16::splat(cpu, b'"')).to_bitmask()
    }
    hakmem::dispatch!(|cpu| quotes(cpu, block))
}

// A slice method with no token in sight: it asks once per call and runs
// under x86-64-v3, so the build without flags counts with POPCNT.
// CHECK-LABEL: <hakmem::isa::x86v3::X86V3 as hakmem::isa::Isa>::run::trampoline::<usize, <[u64] as hakmem::slice::Words>::count_ones::{closure#1}>:
// CHECK: popcnt
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_words_count_ones(xs: &[u64]) -> usize {
    use hakmem::Words;
    xs.count_ones()
}

// GFNI through a token: in a build without flags, the X86V4 carrier
// reverses the bits of sixteen bytes with one `gf2p8affineqb`, where
// SSSE3 alone needs two lookups and a shift.
// CHECK-LABEL: <hakmem::isa::x86v4::X86V4 as hakmem::isa::Isa>::run::trampoline::<u16, hakmem_codegen::cg_dispatch_reverse_bits::{closure#2}>:
// CHECK: gf2p8affineqb
// CHECK-NOT: call
#[unsafe(no_mangle)]
pub fn cg_dispatch_reverse_bits(block: &[u8; 16]) -> u16 {
    #[inline]
    fn reversed<I: hakmem::isa::Isa>(cpu: I, block: &[u8; 16]) -> u16 {
        I::U8x16::load(cpu, block).reverse_bits().to_bitmask()
    }
    hakmem::dispatch!(|cpu| reversed(cpu, block))
}
