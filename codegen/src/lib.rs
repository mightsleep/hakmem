//! The claims the README makes about instructions, checked in the asm.
//!
//! Every `#[no_mangle]` function here is one instantiation to look at;
//! the kernels, not being generic, are looked at in hakmem's own asm. A
//! `CHECK-LABEL` names a function, and the lines after it are
//! [FileCheck](https://llvm.org/docs/CommandGuide/FileCheck.html)
//! directives over that function alone (`nix/codegen.nix` cuts the asm
//! into functions and puts them in this file's order). The prefix is the
//! build: `CHECK` holds in every cell, `BMI2` where the cell has
//! `+bmi2,+pclmulqdq,+ssse3,+avx2`.
//!
//! The same functions, as counts of each mnemonic, are the snapshot next
//! to this crate: a change in codegen is a diff in review, not a line on
//! the benchmark graph three weeks later.

use hakmem::lanes::{Lanes, U8x16};
use hakmem::{Bits, Hilbert2};

// CHECK-LABEL: cg_compact_u64:
// BMI2: pext
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
