# hakmem: design notes

The decisions behind the crate and the reasons for them, for readers
who want to extend it (a carrier, a combinator, a backend) or judge
it. The API reference is the rustdoc; the recipes are
`hakmem::cookbook`. This document assumes you know what POPCNT, PEXT
and a carry chain do. If not, Hacker's Delight chapters 2 and 5 are
the shorter road.

This is a learning project; the Status section of the README says
what that means in practice. The decisions below are the current
ones, not final ones. Where one turns out wrong, the law that exposes
it goes into the test suite first and the decision changes second.

Numbers below were measured on one machine (AMD Zen 5, one thread,
sandboxed) with the benches in `benches/`. They are evidence for the
shape of a claim, not a promise about yours.

## 1. The claim

A bit instruction is not a fast loop. It is a fixed parallel circuit
exposed as an O(1) operation over a container of 64 cells. Each has
a structural role:

| circuit | instruction | over the container |
|---|---|---|
| reduce (arg-min / arg-max) | TZCNT, LZCNT | priority encoder; `⌊log₂ x⌋` |
| reduce (count) | POPCNT | rank, the pillar of succinct structures |
| scan | the adder's carry chain; PCLMULQDQ | prefix network; CLMUL is a scan whose operator is the constant |
| filter | PEXT | keep the cells a mask selects, in order |
| bijection | PDEP ∘ PEXT | scatter / gather on bits; Morton interleave |
| permutation | delta swap; a Beneš network of them | any fixed rearrangement of cells |

Chaining them is function composition; the depth of the chain is the
depth of the circuit. Code written this way exists (simdjson's
classification masks, Hyperscan, Myers' edit distance, chess
bitboards, fusion trees) as one-offs. The algebra had no name, no
types and no crate. What exists on crates.io: `broadword` is a
collection of functions without laws, `bitintr` wraps intrinsics
and is unmaintained, `bitvec` is storage, `safe_arch` is a mechanical
wrapper. None is a composition layer.

The economics that make the composition worth having: one cache miss
to L2 or L3 costs about as much as 300 chained word operations, and a
current core retires four to six independent ones per cycle. A chain
of twelve operations that saves one pointer chase wins. This is why
two-level bitmap descent beats a free list, and why an edit-distance
column in a register beats a table in memory.

## 2. Three decisions

### 2.1 Types carry the domain, not only the width

A value is not a `u64`; it is a word in a domain, and operations
exist where they are lawful. `Dilated<W, D>` has no `Add`: adding a
plain integer to a Morton coordinate is the bug the type makes
uncompilable. It has `incr` and `wrapping_add`, which fill the gaps
with ones so the carry tunnels across them (Raman and Wise).

Where the type system cannot help, the crate does not pretend. The
mask of `compact` / `expand` is data, and no type can state that two
run-time masks are equal. A run-time witness `{ bits, mask }` could
prove that this `expand` belongs to that `compact`, at the cost of a
second word per value. So `compact` and `expand` are plain functions
with a law (`compact_expand_roundtrip`) instead of a type. The
provenance a type carries is limited to masks that are structural,
such as the stride of a dilated integer.

### 2.2 Selection at compile time, never at run time

For an operation of one to three cycles, run-time dispatch costs more
than the operation. There is no branch per combinator on a CPUID
result:

- `#[cfg(target_feature = "bmi2")]` and `pclmulqdq` choose the
  instruction at compile time, in one module (`word.rs`).
- Without the target feature every primitive has a portable
  definition with the same contract. The build is correct
  everywhere; it is fast with `-C target-feature=+bmi2,+pclmulqdq`
  or `-C target-cpu=native`.
- The `portable` cargo feature turns the hardware paths off even when
  the target feature is on. On AMD Zen 1 and Zen 2, PDEP and PEXT
  are microcoded, about 18 cycles against 3 on Intel since Haswell
  and on Zen 3, and the portable definition is faster.

Run-time dispatch belongs a layer up, on whole kernels that run
thousands of times per call, and that layer is the consumer's.

### 2.3 Laws are the exported API

An algebra without laws is a wrapper. `hakmem::laws` exports each
law as a function `fn(...) -> bool` over `W: Word`. The crate tests
them; downstream code runs the same functions over its own carriers
and backends. A law that fails is a bug in the backend, never a
caveat in the documentation.

The laws are also what make section 2.2 sound: the hardware and the
portable definition of a primitive are interchangeable because the
same laws hold for both, on the same inputs, in the same test run.
Every law is a rewrite rule, and a kernel may use either side.

## 3. Shape of the API

One trait, `Bits`, with every combinator, and a blanket
implementation for every `Word`. `use hakmem::Bits;` is the whole
import. The first draft had one trait per domain (`Runs`, `SetView`,
`Scan`, `Compact`, and so on): twelve imports for a kernel, and no
umbrella trait can fix it, because a supertrait's methods are not in
scope through the subtrait. The domain grouping survives as section
headings inside `bits.rs`, which is where docs.rs shows it.

Primitives keep the names `std` uses: `count_ones`, `trailing_zeros`,
`leading_zeros`. The instruction names (POPCNT, TZCNT) are in the
documentation, not in the API; a Rust programmer searches for the
`std` name.

`Word` is open. It exposes exactly the primitive circuits the
combinators are built from, so a carrier is one `impl` block and no
more, with portable defaults for everything that can be derived: bitwise operations, shifts, wrapping `add` / `sub` / `mul`,
byte splat, `count_ones`, `trailing_zeros`, `leading_zeros`,
`clear_lowest_set`, `pext`, `pdep`, `select_lowest`, `xor_scan`,
`xor_scan_down`, `low_ones`. Everything else is derived.

Free functions and types stay in their modules: `slice`, `grid`,
`myers`, `permute::board8`, `dilated::{Dilated, Morton2}`,
`set::Positions`. The README is the crate documentation
(`#![doc = include_str!]`) and a doctest, so its examples cannot
drift from the code.

## 4. Carriers

`Word` is implemented for `u8`, `u16`, `u32`, `u64`, `u128` and for
`Wide<N>`, which is `[u64; N]` viewed as one word: the carry chain
propagates across limbs, shifts cross limb boundaries, `count_ones`
sums. One operation costs `N` limb operations instead of one
instruction, which is still bit-parallel.

`Wide<N>` is the smallest test of a constraint the carrier trait was
designed under: a word need not be one register. `myers::edit_distance`
was written against `Word`, not against `u64`, and it runs on
`Wide<8>` for 512-byte patterns without a change to the algorithm.

A vector register is not a `Word`: its lanes do not carry into each
other, so the laws of the carry chain do not hold across it. It gets
its own trait, `Lanes`, with the operations that are lawful lane by
lane (bitwise, wrapping add and subtract, shifts, unsigned compares to
masks, the 16-entry table lookup) and one bridge, `to_bits`, which
folds a lane mask into a `Word` with a bit per lane. That bridge is
where simdjson's first stage hands its masks to its second, and where
this crate's two algebras meet. The carriers are `U8x8` (eight lanes in
a `u64` by SWAR, the exhaustive-test carrier) and `U8x16` (SSSE3 or
NEON, chosen by `cfg(target_feature)` like BMI2; two `U8x8` halves
otherwise). Wider carriers are more `impl` blocks. A GPU warp mask
would be a third kind of carrier again; it is not in the crate.

A third algebra sits under the lanes and is small: `Affine8`, the
affine maps on the bits of one byte, 8×8 matrices over GF(2) with a
constant. Every byte shift, rotate, reversal and NOT is one, and
composition is matrix multiplication, so a chain of them is one map
computed at compile time. `Lanes::affine` applies a map to every lane:
one GFNI instruction, two nibble lookups by linearity without it,
eight parity folds on the SWAR carrier. It is the first place the
crate has a closed algebra with a hardware witness rather than a list
of tricks.

## 5. Hardware paths

Exactly five `Word` primitives have one, plus `Lanes::affine` with the
byte maps built on it, and every `Lanes` method of
`U8x16`, which is a different register:

| primitive | with the target feature | without |
|---|---|---|
| `pext`, `pdep` | one BMI2 instruction | Hacker's Delight 7-4 and 7-5: `log₂ w` rounds of one prefix-XOR scan each, constant time |
| `select_lowest` | `trailing_zeros(pdep(1 << k, x))` (Pandey, Bender and Johnson, 2017) | Vigna's broadword select, about 30 ALU operations, no table, constant time |
| `xor_scan` (prefix XOR) | PCLMULQDQ by all ones | a log-depth smear, six operations on `u64` |
| `xor_scan_down` (suffix XOR, the Gray decode) | the high half of the same PCLMULQDQ product is the exclusive suffix parity; one XOR more | the smear run downward, six operations |
| `Lanes::affine`, and `reverse_bits`, `sra`, `rotl`, `rotr` (and `shl`, `shr` on x86) through it | one GFNI `gf2p8affineqb`; NEON has `rbit` and native byte shifts | two nibble lookups (`lut16`) and an XOR, by linearity; eight parity folds on the SWAR carrier |

The choice lives in `word.rs` and, for the lanes, `lanes.rs`, and
nowhere else. The `unsafe` in the crate is the intrinsic calls in those
two modules, allowed only when the matching `target_feature` is a
compile-time fact. The safe-intrinsics route of Rust 1.87 does not
apply: it needs `#[target_feature]` on the calling function, which a
trait method cannot carry, and the build configuration does not count.
Miri runs `tests/miri.rs` over both paths (`nix run .#miri-hakmem`).

What the bench says about the portable select (`benches/select.rs`,
1024 words, `k` = half the population):

| `select` on `u64`, per word | dense | sparse |
|---|---|---|
| PDEP | 1.1 ns | |
| Vigna broadword | 4.7 ns | |
| `x &= x - 1` loop, `k` times | 2.7 ns | 1.05 ns |

A broadword select from 2008 loses to the loop on this
microarchitecture for `k` below roughly 28: `blsr` has one cycle of
latency, so the loop is `k` cycles, while the broadword version is
about 20 serial operations including two multiplications. It is kept
as the portable definition for two properties the loop lacks:
constant time (the worst case is `k = 63`) and no data-dependent
latency. A hybrid for small `k`, or a 2 KB table for the last step
(what the `broadword` crate does, and why it is 14 % faster than
this crate's portable path), are open choices.

The portable `pext` / `pdep` tell the same story (`benches/compact.rs`,
1024 word pairs): the parallel-suffix compress costs about 11 ns per
word with a smear scan and 8 ns with PCLMULQDQ, regardless of the mask;
a loop over the mask's set bits costs about 0.65 ns per set bit, so it
wins below roughly 17 set bits (12 with the scan) and loses above.
PEXT does it in 1.3 ns. Constant time and no data-dependent branch is
why the broadword version is the definition.

## 6. Verification

Three layers, all in the test suite:

1. **Laws under proptest**, on `u8` through `u128` and on `Wide<2>`
   and `Wide<3>`, with and without the hardware paths. 330 property
   tests.
2. **Exhaustive sweeps for widths up to 16 bits**, in release builds:
   every `u8` for the unary laws, every pair of `u8` for the binary
   ones, every triple for `compact` composition, every `u16` against
   every structured mask, every Morton coordinate of a 256 × 256 grid,
   Vigna's select on every 16-bit pattern at every byte offset of a
   `u64`, every byte value for the SWAR lanes. Fourteen sweeps.
3. **Kernels without a local algebraic proof** are checked against
   the textbook algorithm. `myers` has no exported law; it is compared
   with the dynamic-programming table on `u8`, `u64`, `u128` and
   `Wide<4>`.

Every bug found in this crate so far was caught by layer 2, not by
layer 1. One example: the carry into `find_escaped` from the previous
word may only affect bit 0 of the mask; an earlier version flipped
the whole word, and the sweep found it at `x = 2, carry = 1`. Random
inputs at 64 bits do not hit that corner in any reasonable number of
cases. Write the law before the kernel, and sweep it at 8 and 16
bits.

CI builds the same matrix as data (`nix/matrix.nix`): hardware path ×
feature flag, tests and doctests and clippy per cell, rustdoc with
warnings as errors, the MSRV build from the `cargo package` tarball,
licence and advisory checks offline. Every cell is a Nix derivation
without network access, and only the cells whose output is not yet in
the binary cache are built: the output path is a function of the inputs
and is known before building, and a path exists in the cache only if
that derivation was built and passed (`.github/plan.sh`).

## 7. Deliberately out

- **Data-dependent control flow and pointer chasing.** The domain is
  straight-line, fixed-width, data-independent kernels, the same
  boundary as bitslicing.
- **Tables.** One, 96 bytes, the 3D Hilbert encode: a memo of a
  twelve-state machine that no algebra composes, built at compile time
  from the algebra that defines it. Everything else is arithmetic.
- **Bit storage.** `bitvec` territory. This is an algebra over words,
  not a container of bits. `slice` is the index-free layer between a
  word and a succinct structure; `rank9` is the one structure, and it
  owns nothing: the caller supplies the directory's words, which is the
  case none of the maintained crates (`sux`, `sucds`, `vers-vecs`)
  covers, all three want an allocator.
- **Promises about autovectorisation.** A combinator compiles to a
  known instruction or to a documented fallback. Nothing in between.
- **Big-integer arithmetic.** `u128` is the register ceiling.
  `Wide<N>` has what `Word` needs and nothing else.
- **Division by constants, square roots, CRC, floating point**
  (Hacker's Delight chapters 8 to 11, 14, 17). A different algebra.
  The 32 × 32 transpose (section 7-3) waits for a consumer.
- **Banded Myers.** For two long strings a few edits apart,
  `triple_accel` beats this crate's full-column Myers on `Wide<8>` by
  about 2×, because it computes only a diagonal band. The variant is
  not shipped. The crate's value is primitives with laws and a
  recipe; racing a specialised implementation on its own ground is
  not. Recipe 7 of the cookbook sketches the band so that anyone who
  needs it can build it in an afternoon and check it with the same
  reference.
- **Wider and richer lanes** (AVX2, AVX-512 with VPCOMPRESSB; GFNI is
  in, section 4). `Lanes` on stable is `core::arch` behind
  `cfg`; `std::simd` is nightly and would buy only generic width.
  Byte compress and expand are the next lane primitives, after the
  first consumer.
- **Run-time feature detection.** Section 2.2.

## 8. Provenance

Where each combinator comes from. Nothing in the first table is new
here; the crate names it, gives it a type and a law, and tests it on
every carrier.

| combinator | source |
|---|---|
| `run_starts`, `has_run` (halving chain) | Hacker's Delight §6-2, figure 6-5 |
| `longest_run` | Hacker's Delight §6-3, as a binary search over `has_run` |
| `zero_bytes`, `bytes_eq`, `bytes_lt`, `sum_bytes` | Hacker's Delight §6-1 |
| `lowest_set_mask`, `below_lowest_set`, `up_to_lowest_set`, `clear_lowest_run`, `prefix_or` as `x \| -x` | Hacker's Delight chapter 2 |
| `round_up_pow2`, `log2_floor` | Hacker's Delight §3-2, §5-3 |
| `next_same_popcount` | HAKMEM item 175 (Gosper) |
| `gray_encode`, `gray_decode` | Hacker's Delight chapter 13 |
| `compact`, `expand` | Hacker's Delight §7-4, §7-5; BMI2 PEXT / PDEP |
| `delta_swap`, `board8` permutations | Knuth, TAOCP 7.1.3 (δ-swap); Hacker's Delight §7-3; the bitboard literature |
| `select` with PDEP | Pandey, Bender and Johnson, 2017 |
| `select` without PDEP | Vigna, 2008 |
| `prefix_xor` by carry-less multiply; `find_escaped` | Langdale and Lemire, simdjson, 2019 |
| `fill_up`, `fill_down` | Kogge and Stone, 1973, as used for sliding attacks on bitboards |
| `Dilated`, `Morton2` | Morton, 1966; Raman and Wise, 2008 |
| `suffix_xor` | Hacker's Delight chapter 13 (the Gray decode as a downward prefix); the CLMUL high half is the same product read the other way |
| `Hilbert2::into_morton` | Hacker's Delight 16-2, the parallel-prefix form of Lam and Shapiro's state machine (1994); the quadrant order of figure 16-1 |
| `Hilbert2::from_morton` | rawrunprotected, *2D Hilbert curves in O(log n)*, 2016: the frame maps of Lam and Shapiro composed by parallel prefix, linear parts in GF(4)*; here in the dilated layout on any carrier |
| `Hilbert3` | the curve of rawrunprotected's 3D tables (2016, 2020); the frames as `A₄ ≅ AGL(1, 4)`, the decode as the 2D encode's scan, the encode as the machine memoised at compile time |
| `myers::edit_distance`, `myers::search` | Myers, 1999; Hyyrö's formulation |
| `rank9::Rank9` | Vigna, 2008: rank9, and a select inventory in the shape of his select9, cases cut at block boundaries |
| `lanes::U8x8` add and subtract | Hacker's Delight 2-18 (SWAR without inter-lane carry) |
| `lanes::Lanes::cmp_le` on SWAR | Hacker's Delight 6-1, the lane compare with full lanes |
| `lanes::Lanes::to_bits` on SWAR | the multiply that gathers the top bits of eight bytes; on NEON the `shrn` narrowing of a compare mask |
| `lanes::Lanes::lut16`, the nibble classifier | simdjson (Langdale and Lemire, 2019), after Muła's PSHUFB lookups |
| `lanes::Lanes::shuffle`, `concat_shift`, `unpack_*`, `add_sat`, `sum_abs_diff`, `mul_add_pairs` | the SSE2 / SSSE3 instruction set as an algebra: PSHUFB, PALIGNR, PUNPCK, PADDUSB, PSADBW, PMADDUBSW; NEON `tbl`, `tbl2`, `zip`, `uqadd`, `uabd` + `addlv` |
| `affine::Affine8`, `Lanes::affine`, `reverse_bits`, `sra`, `rotl`, `rotr` | GFNI's `gf2p8affineqb` as an algebra: Wunkolo, *gf2p8affineqb: Bit reversal* and *int8 shifting*, 2020; the nibble split is Muła's |
| `Bits::ternary`, `Lanes::ternary`, `bits::truth_table` | VPTERNLOG; `f(0xF0, 0xCC, 0xAA)` is the immediate's own definition read backwards |
| `Bits::signed_add_overflows`, `signed_sub_overflows` | Hacker's Delight 2-13; the lane form with VPTERNLOG `0x42` / `0x18`: Wunkolo, *vpternlog: Signed Saturation*, 2025 |
| `Lanes::avg_round`, `avg_floor`; `0x80` as the average of `0x00` and `0xFF` | Hacker's Delight 2-5; PAVGB, `urhadd`; Wunkolo, *pavgb: most-significant-bit constant*, 2022 |
| `Bits::next_subset`, `subsets` | the carry-rippler, `(x − m) & m`, chess programming folklore (Isenberg's wiki, *Traversing Subsets of a Set*) |
| `Bits::gather`, `gather_factor`, `gather_factor_by` | multiply as parallel shift-and-add; Kindergarten bitboards (Isenberg, 2007) as the named instance, the file and diagonal constants re-derived |

What is not in the canon, as far as I know:

- **`grid`: a rectangular run is a separable erosion.** `block_starts`
  applies the halving chain down the rows first (`⌈log₂ h⌉` passes
  of AND) and then along each row (`run_starts(w)`), which is
  `O(log w + log h)` word operations per row instead of `w × h` tests
  per cell. `find_block` is first fit of a rectangle over a tile
  bitmap. Morphologists know the separable erosion; allocator authors
  do not seem to.
- **The composition and homomorphism laws as an exported, machine-
  checked family.** `run_starts(a) ∘ run_starts(b) = run_starts(a + b − 1)`
  (why the halving chain is correct) and its two-dimensional version;
  `run_starts` preserves AND; `zero_bytes` turns OR into AND; fills are
  closure operators; `prefix_xor`, Gray codes, `compact`, `expand` and
  `delta_swap` are XOR-linear; `pdep(pdep(x, n), m) = pdep(x, pdep(n, m))`
  as the dual of `compact_composes`; delta swaps with one shift and
  disjoint masks merge; the Gray successor flips exactly
  `lowest_set_mask(x + 1)`; `select` inverts `rank` on set bits. Each
  is folklore or a one-line proof. Together, verified exhaustively at
  8 and 16 bits and by proptest at 128, they are a rule base nobody
  had written down.
- **`Wide<N>` as a carrier for the whole algebra**, section 4.
- **The Hilbert encode as the carry chain over GF(4), stated so.**
  Read with the coordinates as input, the per-level frame maps are
  affine on `GF(2)²` with linear parts `[[1,1],[0,1]]` and
  `[[0,1],[1,0]]`, which generate `GL(2, 2) ≅ S₃`; the decode's
  transitions depend only on the output pairs, so they collapse to two
  parities. Hacker's Delight gives the parallel-prefix decode and the
  state-machine encode without saying why the asymmetry. The log-depth
  encode is not in the books but it is on the web: rawrunprotected's
  2016 post composes the frame maps by parallel prefix and folds the
  linear parts into GF(4) multiplication. What this crate adds is the
  statement, the adder's `(g, p)` composition is `Aff(1, GF(2))` and
  this is `Aff(1, GF(4))`, so the same Kogge–Stone shape with a
  four-element field; the laws; and the carrier generality. The rule
  behind it, that a state machine over a word is a broadword kernel
  exactly when its transition monoid has a cheap representation, is a
  theorem for the aperiodic and modular cases (Bergeron and Hamel
  2001, Serre 2004, Paperman, Salvati and Soyez-Martin 2023) and open
  for the group case, which is where the Hilbert encode sits. The
  exclusive scan landing in the other lane of a stride-2 suffix XOR is
  a small trick in the same recipe.

## 9. Compatibility

`no_std`, zero dependencies, stable Rust. MSRV 1.89 (edition 2024
needs 1.85; `cast_signed` 1.87; the GFNI intrinsics 1.89), checked in CI
against the
packaged tarball, not the repository.

What is and is not a breaking change:

- Adding a method to `Bits`, or a primitive with a default to `Word`,
  is not; `Bits` is only implemented through the blanket impl. Adding
  a required primitive to `Word` is: carriers outside the crate exist.
- Adding a carrier is not.
- Changing what a law states is. A law is API; a kernel downstream
  may depend on either side of it.
- Results the documentation calls unspecified (for example
  `Word::select_lowest` with `k` at or above the population) are not
  covered by any of this. `Bits::select` returns `Option` and is.

## 10. Open

- A hybrid portable `select` that loops for small `k`, or a table for
  the last step; section 5 has the numbers to decide by.
- SIMD carriers behind a nightly feature, once `portable_simd` is
  stable enough to depend on.
- The 32 × 32 transpose when something needs it.
- In the same vein, in this order: a transition-monoid analyser in
  `laws` (the script that read the 3D tables, made reusable); balanced
  parentheses next to `rank9` (Vigna 2013); bit-parallel LCS;
  shift-and. A 3D encode table over two levels at a step (1.5 KB)
  would halve its 21 loads; not done, the crate keeps one small table.
- A 3D curve with a log-depth encode does not exist in the natural
  class: of the 10 752 corner-to-corner self-similar 3D curves whose
  sub-cubes are symmetries of the cube (every Hamiltonian path of the
  octants, every choice of symmetry per octant meeting the continuity
  conditions), the frame group is `A₄` for 42 and `S₄` for the rest,
  never abelian, and the encode monoid of every one outgrows any
  affine representation. The 2D encode was affine because `V₄` is
  elementary abelian and acts regularly on the quadrants; no 3D curve
  of this kind has that. Face-gated and non-self-similar curves were
  not searched.
- A batched Hilbert encode: the scan is throughput-bound, so on many
  points at once the four-state loop interleaved eight ways may match
  it; not measured.
- A 3 % directory (poppy, Zhou, Andersen and Kaminsky 2013) next to
  the 25 % rank9, and select0. On select the crate is within 15 % of
  sux's select9 on sparse slices, the checked indexing it keeps; that
  gap closes only with `unsafe`, which the crate does not take.
- Banded Myers is not planned; recipe 7 of the cookbook is the
  instruction sheet.

## Sources

- Beeler, Gosper, Schroeppel. *HAKMEM*. MIT AI Memo 239, 1972.
  Items 161 to 180.
- Warren. *Hacker's Delight*, 2nd edition. Addison-Wesley, 2012.
- Knuth. *The Art of Computer Programming*, volume 4A, section 7.1.3,
  *Bitwise Tricks and Techniques*. 2011.
- Vigna. *Broadword Implementation of Rank/Select Queries*. WEA 2008.
- Pandey, Bender, Johnson. *A General-Purpose Counting Filter*.
  SIGMOD 2017 (the PDEP select).
- Myers. *A Fast Bit-Vector Algorithm for Approximate String Matching
  Based on Dynamic Programming*. Journal of the ACM, 1999.
- Hyyrö. *Explaining and Extending the Bit-parallel Approximate String
  Matching Algorithm of Myers*. Technical report, 2001.
- Raman, Wise. *Converting to and from Dilated Integers*. IEEE
  Transactions on Computers, 2008.
- Lam, Shapiro. *A Class of Fast Algorithms for the Peano-Hilbert
  Space-Filling Curve*. ICIP 1994.
- rawrunprotected. *2D Hilbert curves in O(log n)*, 2016, and *3D
  Hilbert curves in O(log n) optimised*. threadlocalmutex.com, posts
  126 and 149; code at <https://github.com/rawrunprotected/hilbert_curves>,
  public domain.
- Serre. *Vectorial Languages and Linear Temporal Logic*. TCS, 2004.
  Paperman, Salvati, Soyez-Martin. *An Algebraic Approach to Vectorial
  Programs*. STACS, 2023.
- Wunkolo. *Wunk*, blog, 2020 to 2025: *gf2p8affineqb: Bit reversal*,
  *gf2p8affineqb: int8 shifting*, *pavgb: most-significant-bit
  constant*, *vpternlog: Signed Saturation*.
  <https://wunkolo.github.io/>
- Isenberg et al. *Chess Programming Wiki*: *Kindergarten Bitboards*,
  *Traversing Subsets of a Set*, *Obstruction Difference*.
  <https://www.chessprogramming.org/>
- Langdale, Lemire. *Parsing Gigabytes of JSON per Second*. The VLDB
  Journal, 2019.
- Kogge, Stone. *A Parallel Algorithm for the Efficient Solution of a
  General Class of Recurrence Equations*. IEEE Transactions on
  Computers, 1973.
- Beneš. *Optimal Rearrangeable Multistage Connecting Networks*. Bell
  System Technical Journal, 1964.
