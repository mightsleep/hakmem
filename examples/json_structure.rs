//! The first stage of simdjson: where the structural characters of a
//! JSON text are, outside strings, 64 bytes at a time. Byte lanes
//! classify, a mask per class comes out as a word, and from there it is
//! carry chains: escaped quotes, then the inside of strings as a prefix
//! XOR of the quotes.
//!
//! `cargo run --release --example json_structure`

use hakmem::lanes::U8x16;
use hakmem::prelude::*;

/// Structural characters `{ } [ ] : ,` by nibble lookup, as in the
/// module docs of `hakmem::lanes`.
const LO: [u8; 16] = [16, 0, 0, 0, 0, 0, 0, 0, 0, 32, 34, 12, 1, 44, 0, 0];
const HI: [u8; 16] = [32, 0, 17, 2, 0, 4, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];

/// One mask per class over 64 bytes: bit `i` for byte `i`.
fn masks(block: &[u8; 64]) -> (u64, u64, u64) {
    let (mut structural, mut quotes, mut backslashes) = (0u64, 0u64, 0u64);
    for (i, chunk) in block.as_chunks::<16>().0.iter().enumerate() {
        let lanes = U8x16::load(chunk);
        let class = lanes.lut16_nibbles(LO, HI);
        let s = class
            .and(U8x16::splat(15))
            .cmp_eq(U8x16::zero())
            .not()
            .to_bitmask();
        let q = lanes.cmp_eq(U8x16::splat(b'"')).to_bitmask();
        let b = lanes.cmp_eq(U8x16::splat(b'\\')).to_bitmask();
        structural |= u64::from(s) << (16 * i);
        quotes |= u64::from(q) << (16 * i);
        backslashes |= u64::from(b) << (16 * i);
    }
    (structural, quotes, backslashes)
}

/// Positions of the structural characters that are not inside strings.
fn structure(json: &[u8]) -> Vec<usize> {
    let (mut out, mut escape_carry, mut in_string) = (Vec::new(), false, false);
    for (b, chunk) in json.chunks(64).enumerate() {
        // The last block is padded with spaces, which classify as nothing.
        let mut block = [b' '; 64];
        block[..chunk.len()].copy_from_slice(chunk);
        let (structural, quotes, backslashes) = masks(&block);
        // Both scans carry into the next block the same way.
        let escaped;
        (escaped, escape_carry) = backslashes.find_escaped(escape_carry);
        // Inside a string: an odd number of unescaped quotes at or
        // before the byte, this block and every one before.
        let strings;
        (strings, in_string) = (quotes & !escaped).prefix_xor_carry(in_string);
        out.extend(
            (structural & !strings)
                .positions()
                .map(|p| 64 * b + p as usize),
        );
    }
    out
}

fn main() {
    // Long enough that a string, and an escaped quote in it, cross the
    // 64-byte boundary between blocks.
    let json = br#"{"name": "a [tricky] \"string\", with: {braces}", "note": "hmm \" the quote is byte 64, [more] after", "list": [1, 2, {"k": "v"}]}"#;
    let found = structure(json);
    let expected: Vec<usize> = json
        .iter()
        .enumerate()
        .filter(|&(_, c)| b"{}[]:,".contains(c))
        .map(|(i, _)| i)
        .filter(|&i| {
            // Outside strings by the slow definition: an even count of
            // unescaped quotes before it.
            let mut inside = false;
            let mut escaped = false;
            for &c in &json[..i] {
                match c {
                    b'\\' if !escaped => escaped = true,
                    b'"' if !escaped => inside = !inside,
                    _ => escaped = false,
                }
                if c != b'\\' {
                    escaped = false;
                }
            }
            !inside
        })
        .collect();
    assert_eq!(found, expected);
    let shown: String = found.iter().map(|&i| json[i] as char).collect();
    println!(
        "{} structural characters outside strings: {shown}",
        found.len()
    );
}
