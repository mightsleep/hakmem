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

### 2.2 Selection at compile time per word, at run time per batch

This is the rule as built. Section 11 proposes the next one: the same
two granularities, with the instruction set a value the caller can
hold, pass and choose.

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
thousands of times per call. The batch conversions are such kernels,
so they dispatch: on `x86_64` every kernel is compiled under its own
`#[target_feature]`, and a call asks `cpu.rs` once which one to run.
The answer is a constant when the build has the features (the
detection is then not even compiled) and otherwise CPUID and XCR0,
read once through `core::arch` and cached in an atomic, so the crate
stays `no_std` and without dependencies. Under Miri, which has no
CPUID, and with the `portable` feature only the compile-time answer
counts. The per-word combinators keep the rule above. Before this a
build without `-C target-cpu` saw none of the kernels, and the 2D
encode ran at the speed of the table crates it was written to beat.

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

Batch operations are transforms of indices over words the caller
owns: a slice of keys in, the same slice converted in place, no
allocation, and coordinates never enter the kernel.
`Hilbert2::<u64>::from_morton_in_place(&mut keys)` is the first: the
caller fills the keys with `Morton2::encode` from whatever layout its
points are in (arrays of pairs, separate columns, quantised floats)
and the kernel sees words only. So the crate commits to no point type,
the batch has the shape of `slice` and `Rank9` (words in, caller's
storage, O(1) work a word), and a convenience form over coordinate
slices can be added later without breaking anything, where the
reverse would not. It was, for the 3D curve on `u64`:
`Hilbert3::<u64>::encode_columns(xs, ys, zs, out)` and
`decode_columns`, three columns of words and still no point type. The
columns are not a convenience there but the faster form: the byte
planes want an octant a byte, and three `vpmultishiftqb` read bit `l`
of `x`, `l - 1` of `y` and `l - 2` of `z` into one, so the Morton code
the keys form would be built only to be taken apart. They are inherent functions on the concrete widths
that have a kernel (`u32`, `u64`), not on every `Word`: dispatching on
the width of a generic `W` would need an unsafe cast of the slice, and
the crate's `unsafe` is intrinsic calls and the kernel dispatch.

Free functions and types stay in their modules: `grid`, `myers`,
`permute::board8`, `set::Positions`. The prelude holds what a caller
names or whose methods it calls: the traits `Word`, `Bits`, `Words`
(the slice operations, on `[W]` itself), `Lanes`, `Curve2` and
`Curve3`, and the key and carrier types. The README is the crate
documentation (`#![doc = include_str!]`) and a doctest, so its
examples cannot drift from the code.

The conventions, settled before 0.2 while breaking was free. A range
of coordinates or keys is a `RangeBounds`, not a pair: three pairs of
the same type in a row are two swaps waiting to happen, and `..` is a
valid column. A function that fills a caller's buffer returns the
part it filled, as `char::encode_utf8` does, not a count to slice by.
Conversions of `Copy` values are `to_`, as `f64::to_bits`; between
curves they are also `From`. A choice between two behaviours is two
functions, not a `bool` (`fill_block`, `clear_block`). The curves share
a trait so an index can change curve by changing a type; the inherent
methods keep their own names (`code`, `index`) and win where both are
in scope. `Word` is ordered, hashable and printable in binary because
every carrier is, and asking later would break every carrier written
meanwhile.

## 4. Carriers

`Word` is implemented for `u8`, `u16`, `u32`, `u64`, `u128` and for
`Wide<N>`, which is `[u64; N]` viewed as one word: the carry chain
propagates across limbs, shifts cross limb boundaries, `count_ones`
sums. One operation costs `N` limb operations instead of one
instruction, which is still bit-parallel.

`Wide<N>` is the smallest test of a constraint the carrier trait was
designed under: a word need not be one register. `myers::distance_in`
was written against `Word`, not against `u64`, and it runs on
`Wide<8>` for 512-byte patterns without a change to the algorithm.

A vector register is not a `Word`: its lanes do not carry into each
other, so the laws of the carry chain do not hold across it. It gets
its own trait, `Lanes`, with the operations that are lawful lane by
lane (bitwise, wrapping add and subtract, shifts, unsigned compares to
masks, the 16-entry table lookup) and one bridge, `to_bitmask`, which
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

The batch Hilbert conversions have their own kernels, one level
machine read as a table and applied to a register of keys at a time:

| batch | with the target feature | without |
|---|---|---|
| `Hilbert2::from_morton_in_place` (`u32`, `u64`) | AVX-512 VBMI: one `vpermi2b` through 128 entries a step, three levels a step, the frame modulo the reflection of both axes (a XOR mask on the cells, the swap bit in the index byte), three `vpternlog` around the lookup; NEON: the same reduction two levels a step, 32 entries in two registers for `tbl` | `from_morton` per key |
| `Hilbert3::from_morton_in_place` (`u32`, `u64`) | AVX-512 VBMI: one `vpermi2b` through the 96-byte encode table, padded to two registers, a level a step, per register of keys (`u32`) or per plane of 64 keys (`u64`: `vpmultishiftqb` and three rounds of `vpermt2b` in, the same rounds and a `vpmaddubsw` / `vpmaddwd` pack out, the transposes checked at compile time); NEON: `tbl` over four registers and `tbx` over two; AVX2 alone: the machine modulo its translations, 24 entries in two PSHUFB of 16 a level, the second read through `index ^ 0x80` | `from_morton` per key |
| `Hilbert3::to_morton_in_place` (`u32`, `u64`) | the same kernels through the inverse table (`state · 8 + triple → state_below · 8 + octant`, a bijection per state, checked at compile time); AVX2 the factored inverse, the translation in index bits 4 and 5, which PSHUFB ignores | `to_morton` per key, the algebraic scan |
| `Hilbert3::<u64>::encode_columns` / `decode_columns` | AVX-512 VBMI and GFNI: the `u64` planes fed from three columns by `vpmultishiftqb` at offsets `l`, `l - 1`, `l - 2`, and emptied into them by the way back with the levels reversed and one `gf2p8affineqb` bit transpose a register | Morton code and the batch above |
| `Morton2::encode_columns` / `decode_columns` (`u32`, `u64`) | AVX2: a coordinate's bytes widened to 16 bits and a nibble a byte, one PSHUFB through 16 entries spreads it over the even (odd) bits; back, two PSHUFB per axis and a pack | `encode` / `decode` per point |
| `Hilbert2::encode_columns` / `decode_columns` (`u32`, `u64`) | the Morton columns and then `from_morton_in_place`; decoding, `to_morton` per key into 256 codes on the stack and the Morton columns | the same, per key where the parts are |

Measured on Zen 5 (Ryzen AI 5 340, `target-cpu=native`, one core,
1024 keys, `benches/hilbert.rs`), from coordinates to keys and back, ns a key:

| conversion | batch | per key | incumbent |
|---|---|---|---|
| 2D encode, `u64` keys, 32 levels | 1.6 | 5.9 | `fast_hilbert` 11.9 |
| 2D encode, `u16` coordinates to `u32` keys, 16 levels (the packed R-tree case) | 0.90 | 6.1 | `fast_hilbert` 8.3 |
| 3D encode, `u64` keys, 21 levels | 1.7 | 13.1 | rawrunprotected's tables 15.2 |
| 3D decode, `u64` keys, 21 levels | 1.8 | 8.8 | rawrunprotected's tables 15.1 |
| 3D encode from three columns, 21 levels | 1.2 | 13.1 | rawrunprotected's tables 15.2 |
| 3D decode to three columns, 21 levels | 1.0 | 8.8 | rawrunprotected's tables 15.1 |
| 2D Morton encode from two columns, `u64` | 0.14 | 0.47 (PDEP) | |
| 2D Morton decode to two columns, `u64` | 0.22 | 0.73 (PEXT) | |
| 2D Hilbert encode from two columns, `u64`, any build | 1.3 | 5.9 | `fast_hilbert` 11.9 |

The query side, `cover` (`cover.rs`): a rectangle on the full `u64`
grid, sides up to `2^8`, `2^16` or `2^24` cells, turns into ranges in
about the same time whatever its size, since the depth stops where the
budget does: 0.3 µs for 16 Morton ranges and 1.0 for 64, 1.0 and 2.8
for Hilbert (3.5, 12, 8.5 and 30 before the four changes below).

The depth is counted, not walked. A run of the exact cover starts at a
cell of the rectangle whose predecessor on the curve is not in it. On
Z-order the predecessor is a borrow: with the lowest set bit at bit
`t` of `x`, `(x, y)` follows `(x - 1, y | low(t))`, and that leaves the
rectangle through `x = x0` or through `y1` when the aligned block
sticks out past it; per level and lane a product of two counts of
arithmetic progressions, 20 ns at any depth. Hilbert is continuous, so
a run starts where a step enters across a side. Across the line
`x = X0 - 1/2` only the nodes at level `tz(X0) + 1` step, one column
of them, and what each contributes depends on its frame alone. The
nodes strictly between the ends are counted by frame (a weight per
frame and height, and the count below a node number as a sum along its
path, both from the bottom level up in four lanes), the two end nodes
cell by cell. Their frames come from the indices of the rectangle's
corners: read top down, the decoder's frames form the Klein group, so
the frame above level `l` is the parity of the digits above it that
swap and of those that flip. 150 to 250 ns a count. The counts fall
as the depth grows, so a guess from the perimeter and the budget and a
step or two either way find the depth, two counts a query on average.

The walks go three levels a step: the 64 descendants of a node as one
`u64` in curve order, those meeting the rectangle the AND of a mask of
its columns and a mask of its rows, each looked up per frame (tables
built at compile time from the one-level digit table, about 5 KB a
frame). A run of ones is a range of keys; only the partial children
are descended into. Hilbert pays twice Morton's walk because it fits
the budget a level finer, the curve having about half the runs for
the same perimeter: a better cover for the time.

The threshold between the gaps closed and those kept was a fifth of a
Morton cover: a quickselect over up to twice the budget, its compares
unpredictable. A branch-free partition was no faster, so it was not
the mispredictions but the number of words. The gaps are not
arbitrary numbers: each is a sum of the sizes of a few nodes left out,
so they bunch at a few bit lengths. The first walk counts them by bit
length as it writes them, the counts name the bucket the threshold is
in and how many gaps lie below, and the quickselect runs on that
bucket alone. That is the first level of an offset allocator's size
classes, and the second level finishes the job: the three bits under
the leading one split the bucket in eight, counted with the least and
greatest gap of each. Measured on random rectangles, a bucket held one
value 93 % of the time on Z-order but only 20 to 27 % on Hilbert (two
values, as a rule); the class held one value every time. The threshold
is then read off the counts, and the quickselect stays for a class of
several values, which the unit tests make on purpose. The search
starts at the least bit length seen, the allocator's bitmap of
non-empty classes in one number, since the threshold sits half a bit
length above it on average. A fifth off Morton at 64 ranges, a tenth
off Hilbert at 16. An approximate class (close whole classes, bound
the over-cover by `1 + 2^-3`) would drop the select entirely and
weaken the law; with the classes exact on every measured input there
was nothing to buy.

Against the incumbents at the same number of ranges: GeoMesa's
`zranges` (a breadth-first descent with a loose range limit) leaves
2 to 12 % more over-cover and takes 2 to 8 times as long (the walk
that counted by walking was 1.2 to 2 times slower than it at small
budgets); the S2 region coverer, which approximates in cells rather
than ranges, takes 20 to 80 µs.

`intersects` answers the question the other way round, for pruning a
block of sorted keys by its ends: one descent of about two paths, since a node is an interval of keys and a square of cells
at once and only the nodes on the paths of the two ends are partly in
the interval. It steps three levels at a time as the cover does: the
descendants the interval meets and those inside it are two runs of
bits, the rectangle gives its two masks, and a descendant inside one
while meeting the other ends it. On `u64` a hit costs 40 to 90 ns,
less for bigger rectangles, which offer a descendant inside sooner; a
near miss, the granule next to the rectangle, 100, since proving no is
a walk to the bottom; a granule nowhere near 10. A level at a time
they were 5 to 7 times that, bar the last. The first benchmark timed
only the last kind and reported a 2x; the real one was bigger, which
is the rare way for a benchmark to be wrong.

Without VBMI the batch is still faster than the per-key form (7.0
against 12.3 µs for the 2D `u64` case on the same core): Morton first
and the conversion second is two loops the compiler schedules better
than one fused one. The 3D encode is where the kernel matters most:
its transition monoid has no algebra (section 8), so a level is a
lookup and not a scan, exactly the case a table in a register serves.
The step keeps the symmetry the monoid loses: the translations
`V₄ ⊲ A₄` act on octants as a XOR and commute with it, so the 96
entries are 24 between two XORs, the size of two 16-entry shuffles
(checked at compile time in `hilbert3.rs`). That is the AVX2 kernel:
on the same core built for `x86-64-v3`, 5.3 ns a key for the 3D
encode and 5.7 for the decode, against 12.6 and 8.7 per key. Eight
rows of keys in flight spill and still beat four that fit, by 5 %.

With VBMI the count that decides is the same kind of instruction: on
this core shuffles, variable and immediate shifts and the byte
multiply-adds issue once a cycle, bitwise operations twice
(`vpternlog` fused into the lane kernel bought 2 %, not the 20 % the
instruction count promised). The lane kernel spends three of the first
kind a level on eight keys; the `u64` planes spend one `vpermi2b` a
level on 64 keys and pay about five shuffles a key to transpose in and
out, 1.0 ns a key against 1.85 for the kernel alone.
The NEON paths are checked in CI on the aarch64 runner and not yet
measured.

The choice lives in `word.rs`, `lanes.rs` and the batch kernels at the
end of `hilbert.rs` and `hilbert3.rs`, and nowhere else. The `unsafe`
in the crate is the intrinsic calls in those modules, allowed only
when the matching `target_feature` is a compile-time fact, and, since
the batch kernels dispatch, the calls into a `#[target_feature]`
kernel that `cpu.rs` found the features for, and CPUID and XGETBV
themselves. The safe-intrinsics route of Rust 1.87 does not
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
  twelve-state machine that no algebra composes across levels (one
  level factors through `A₄` to 24 entries), built at compile time
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
| `Hilbert2::to_morton` | Hacker's Delight 16-2, the parallel-prefix form of Lam and Shapiro's state machine (1994); the quadrant order of figure 16-1 |
| `Hilbert2::from_morton` | rawrunprotected, *2D Hilbert curves in O(log n)*, 2016: the frame maps of Lam and Shapiro composed by parallel prefix, linear parts in GF(4)*; here in the dilated layout on any carrier |
| `Hilbert3` | the curve of rawrunprotected's 3D tables (2016, 2020); the frames as `A₄ ≅ AGL(1, 4)`, the decode as the 2D encode's scan, the encode as the machine memoised at compile time |
| `myers::distance_in`, `myers::search` | Myers, 1999; Hyyrö's formulation |
| `rank9::Rank9` | Vigna, 2008: rank9, and a select inventory in the shape of his select9, cases cut at block boundaries |
| `lanes::U8x8` add and subtract | Hacker's Delight 2-18 (SWAR without inter-lane carry) |
| `lanes::Lanes::cmp_le` on SWAR | Hacker's Delight 6-1, the lane compare with full lanes |
| `lanes::Lanes::to_bitmask` on SWAR | the multiply that gathers the top bits of eight bytes; on NEON the `shrn` narrowing of a compare mask |
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
- The 3D factored kernel on SSSE3 and on NEON `tbl` over two
  registers: the AVX2 one at half the width, and on NEON against the
  four-register `tbl` it has now, which nobody has measured either. The
  same factoring for the 10 710 curves with frames `S₄` is
  likely where their `V₄` is the even reflections; not checked.
- A batched Hilbert encode: the scan is throughput-bound, so on many
  points at once the four-state loop interleaved eight ways may match
  it; not measured.
- A 3 % directory (poppy, Zhou, Andersen and Kaminsky 2013) next to
  the 25 % rank9, and select0. On select the crate is within 15 % of
  sux's select9 on sparse slices, the checked indexing it keeps; that
  gap closes only with `unsafe`, which the crate does not take.
- Banded Myers is not planned; recipe 7 of the cookbook is the
  instruction sheet.
- `cover` spends most of its time emitting runs, and the obvious fixes
  did not move it. On Zen 5 a call retires about three instructions a
  cycle with next to no L1 misses; a fifth of the cycles go to
  mispredicted branches, most of them set by the shape of the
  rectangle's edge (which child is inside, how many runs a mask has).
  The profile blamed the store of the sink's state after every run
  (the bounds check may panic, so it must be written) and a table of
  gap counts bumped in memory, each gap waiting on the one before.
  Keeping the state in registers for a mask's runs, and counting the
  gaps in registers at the threshold instead, were both within noise,
  Hilbert a few per cent worse; the stalls the profile showed were
  hidden under other work. Not tried: extracting a mask's runs a fixed
  number at a time with the extra writes overwritten later, as
  simdjson does for structural indexes (its loops are as short and as
  unpredictable as ours), and `vpcompressb`, too much machinery for
  one to four runs a mask. What would change the count is less work,
  not faster work: one walk instead of two, if the depth stops at
  `budget` runs rather than twice that and nothing is merged (about
  twice as fast, a coarser cover by a level; not measured how much
  coarser), or the Hilbert counts at `s` and `s − 1` sharing their
  path (a third of the time at 16 ranges). Open, to come back to.

## 11. Instruction sets as values (proposed)

Not built. Section 2.2 describes the crate as it is; this is where the
API is going, written down before the code so the code can disagree
with it in public.

### 11.1 What 2.2 gets wrong

The usual way to dispatch at run time in Rust is to compile a hot loop
under `#[target_feature]` and pick the copy after a CPUID check. In a
build without `-C target-feature`, a user function marked
`#[target_feature(enable = "bmi2,popcnt")]` that calls `x.compact(m)`
compiles to 291 instructions and no PEXT. `cfg(target_feature)` is
evaluated once for the whole crate, and the caller's attribute never
reaches it. The idiomatic dispatch gets the portable path.

The same function gets POPCNT for `Words::rank` and TZCNT and BLSR for
`positions`: plain Rust follows the features of whatever it inlines
into. The damage is confined to where hakmem chooses by `cfg`: the
five primitives of section 5, and `U8x16`, whose representation
changes with the build. sux and vers-vecs choose the same way and
say so in their documentation ("enable BMI2 and popcnt").

Three more things are hidden that a library of instructions should
hand over. The batch kernels choose through `cpu.rs`, and nobody can
ask what was chosen or choose instead. The `portable` feature switches
the whole dependency graph at once. And a test run sees one path per
build, so the CI matrix multiplies builds to see the rest.

### 11.2 Constraints

- `no_std`, no dependencies, stable Rust. Safe `#[target_feature]`
  functions are stable since 1.86; the MSRV (1.89) already covers them.
- No dispatch per word. A predicted branch and a call around a
  three-cycle instruction lose to the portable code they replace. The
  choice is made at run time only where one call covers a slice or a
  batch, and there it is free next to the kernel.
- `x.select(k)` stays what it is. Nothing below is required reading
  for someone who wants a select.

### 11.3 Three layers, one implementation

1. **Native.** The instruction set the build proves, as today. The
   default for methods on a word and for external iterators.
2. **Tokens.** A zero-sized value per instruction-set level, `I: Isa`.
   The caller dispatches once at the top of a hot loop and passes the
   token down; primitives are methods on it and carriers are its
   associated types. Methods on slices and batches dispatch
   internally once per call and have an `_in(isa, ..)` form for code
   that has already chosen.
3. **Leaves.** `hakmem::x86::bmi2::pdep` and its kind: safe
   `#[target_feature]` functions tagged with exactly the features they
   use, for code under somebody else's dispatch (a `multiversion`
   clone, pulp, fearless_simd, a hand-written CPUID check). LLVM
   inlines such a function only into a caller whose features include
   its own, so a leaf tagged with all of x86-64-v3 would stay a call
   inside a clone that lacks F16C. Tagged `bmi2`, it inlines into
   anything that has BMI2.

Tokens call leaves and Native is the token of its level, so each path
has one implementation.

### 11.4 The trait

```rust
pub trait Isa: Copy + Send + Sync + core::fmt::Debug + 'static {
    type U8x16: Lanes<Isa = Self, Bitmask = u16>;
    fn pext<W: Word>(self, x: W, mask: W) -> W;
    fn pdep<W: Word>(self, x: W, mask: W) -> W;
    fn select_lowest<W: Word>(self, x: W, k: u32) -> u32;
    fn xor_scan<W: Word>(self, x: W) -> W;
    fn xor_scan_down<W: Word>(self, x: W) -> W;
    /// `f` compiled with this level's target features.
    fn run<R>(self, f: impl FnOnce(Self) -> R) -> R;
}

hakmem::isa::dispatch!(|cpu| stage1(cpu, input, &mut out));
```

`run` is a trampoline: an inner function under the level's
`#[target_feature]` that calls the closure. Measured on a two-crate
test, the closure and everything it inlines get the instructions: a
PEXT loop through a token inside `run` compiles to nine unrolled PEXT,
and `iter().map(..).fold(..)` inside it keeps them. `dispatch!` is a
`macro_rules!` over `isa::detect()` that repeats the body once per
level, which is the price in code size and compile time. No procedural
macro: that would be the crate's first dependency.

### 11.5 Levels, not features

A token is a level: the psABI levels plus the extensions hakmem uses,
the way Google Highway adds AES and CLMUL to its AVX2 target.
PCLMULQDQ, GFNI and VBMI are in no psABI level (LLVM's `X86.td`).

| token | features | primitives | `U8x16` | batch kernels |
|---|---|---|---|---|
| `Portable` | none | broadword | two SWAR halves | per key |
| `X86V2` | x86-64-v2 (POPCNT, SSE4.2 and so SSSE3) | broadword | SSSE3 | per key |
| `X86V3` | x86-64-v3 and PCLMULQDQ | PEXT, PDEP, CLMUL | SSSE3 | AVX2 |
| `X86V4` | x86-64-v4, VBMI, GFNI | as `X86V3` | SSSE3, GFNI byte maps | VBMI |
| `Neon` | the aarch64 baseline | broadword | NEON | NEON |

Levels, because every added feature doubles the combinations a
dispatch has to monomorphise, and pulp, fearless_simd, multiversion
and Highway all settled on levels for that reason. Exact features stay
where they matter, on the leaves. `Native` is not a level: it is its
own token, choosing primitive by primitive what the build proves, which
is what 0.2 does (section 11.11 says why).

### 11.6 Carriers as associated types

`U8x16` becomes `I::U8x16`: `Swar16<I>` for `Portable`, `X86x16<L>`
for the x86 levels, `Neon16`. `X86x16<L>` holds the register and
`PhantomData<L>`; the byte maps pick GFNI when `L::GFNI` says so, an
associated constant, so the choice folds at compile time where 0.2
had a `cfg`.

A value of an SSSE3 carrier is a proof that the CPU has SSSE3: its
methods wrap the intrinsics and are sound because the value exists,
and `isa()` hands the token back out of it. So the `Lanes`
constructors take the token: `Lanes` gains `type Isa` and `isa()`,
and `splat`, `load` and `zero` take `Self::Isa`. A constructor without
one would let `X86x16::splat` run on a CPU without SSSE3.

The plain user does not see this. A carrier that is sound anywhere
(`U8x8`, `Swar16`) or proven by the build (`X86x16<Native>` with SSSE3
in it, `Neon16`) also has inherent `splat`, `load` and `zero` without a
token, and inherent methods win the lookup, so `U8x16::load(bytes)`
compiles as before and `U8x16` is simply the carrier of `Native`.
Generic code says `I::U8x16::load(cpu, bytes)` and gets the trait's.
`Swar16<I>` takes any level with a `Default` (`Portable`, `Native`),
since `Native`'s carrier in a build without SSSE3 is SWAR, and
`Isa::U8x16` insists on `Lanes<Isa = Self>`.

Wider carriers (`U8x32` for AVX2, `U8x64` for AVX-512) fit later as
more associated types of the levels that have them.

### 11.7 Where tokens come from

`isa::detect()` (the CPUID and XCR0 check `cpu.rs` does now, cached,
`no_std`; under Miri the compile-time answer), `Native` and `Portable`
(always), `unsafe fn new_unchecked()`, and a safe
`#[target_feature(enable = "..")] fn assume()` that can only be
called where the features are already proven (fearless_simd #293).
`isa::available()` lists every level the CPU has.

### 11.8 Where it breaks

- **The generic cliff.** Code generic over `I: Isa` has to inline
  into the trampoline. A helper marked `#[inline(never)]`, or one LLVM
  finds too big, is compiled without the level's features, and every
  primitive in it becomes a call: measured, one `call _pext_u64` a
  word. Trait methods cannot be safe `#[target_feature]` functions, so
  for generic code the compiler cannot catch it; for leaves on a
  concrete path it can, since calling one outside a matching context
  needs `unsafe`. fearless_simd reports the same trap (#338, #380).
  The guard is the codegen check: a `dispatch!` body whose asm must
  hold instructions, not calls.
- **External iterators.** `next()` belongs to the caller, so
  `positions()` stays Native. Internal iteration
  (`for_each_position(|p| ..)`) and `positions_into(&mut buf)`
  dispatch once, with the closure monomorphised inside `run`.
- **Structures queried per word.** A `Rank9` is built once and asked
  a million times, and a query is one select in a word. The plan was
  a type parameter chosen at construction, `Rank9<'a, I = Native>`;
  the measurement in 11.11 made it unnecessary. Each query asks per
  call, which already beats today everywhere, and the caller who wants
  the last quarter wraps the query loop in one `dispatch!` and calls
  `select_in(k, cpu)`: the level lives in the caller's dispatch, not
  in the type.
- **Zen 1 and Zen 2.** They report BMI2 and run PDEP and PEXT in
  microcode, about 18 cycles. `detect()` cannot see that from the
  feature bits; it can from the family (AMD 17h), which is what the
  `portable` feature is for today.

### 11.9 What it buys besides speed

- `for level in isa::available()` runs every path the machine has in
  one `cargo test`. On the Zen 5 this is written on that is
  `Portable`, `X86V2`, `X86V3` and `X86V4` in one run, where today it
  takes one build per `RUSTFLAGS`.
- The laws take a level, so a backend is checked against `Portable`
  inside one binary.
- The codegen checks see every level in every build, since the level
  code is compiled regardless of `RUSTFLAGS`.

### 11.10 Open

- How `Word` carriers route to leaves: `u32` and `u64` directly,
  `u128` and `Wide<N>` by limbs, `u8` and `u16` widened.
- Zen 1 and 2 as a level of their own, or a flag in `detect()`.
- An aarch64 level with PMULL for the scans (11.11), and how a
  `no_std` crate learns it is there.
- `Isa` sealed (hakmem's levels only) or open to user levels. Sealed
  first; opening it later breaks nothing.
- Where slices stop detecting. Measured in 11.11: nowhere, the check
  is lost in one word.

### 11.11 What the prototype changed

The first cut (`isa.rs`, `x86.rs`, the `_in` methods on `Word`, the
`X86V3` token) disagreed with the text above twice, and the text lost.

- `Native` was going to be an alias for the highest level the build
  proves. Builds are not levels: `+bmi2,+pclmulqdq,+ssse3,+avx2`, the
  test matrix's cell, lacks BMI1, LZCNT, FMA and more of x86-64-v3, so
  the alias would have been `Portable` and PEXT would have gone
  broadword in a build that asked for BMI2. `Native` stayed a token of
  its own with a `cfg` per primitive.
- A target feature enabled for the whole build does not make a call to
  a `#[target_feature]` function safe; rustc wants the feature on the
  calling function and says so (E0133). `Native` calls the leaves in
  `unsafe` with the `cfg` as the proof, and a user who builds with
  `-C target-cpu` and calls a leaf from a plain function does the same.

Measured, in a build without flags: a `dispatch!` body folding `pext`
over a slice is five PEXT and no call (codegen cell `portable`), where
`x.compact(m)` is a jump to the broadword definition. `tests/isa.rs`
checks `Portable` and `X86V3` in one run. Routing the plain methods
through `Native` cost one register move in
`Hilbert3::from_morton_in_place`, which the `bmi2` snapshot showed.

The carriers followed without a change to any function already in the
snapshots: `U8x16` over `Native` compiles as it did. Under a token, the
quote mask of a 16-byte block is six instructions (two moves, an
unpack, PCMPEQB, PMOVMSKB, return) in the build without flags, where
the plain `U8x16` there is 38 instructions of SWAR; the unpack is a
load assembled from two halves, which one MOVDQU would replace.
`tests/isa.rs` runs the lane laws on `Swar16` and `X86x16<X86V3>` in
the same run, and flipping the SSSE3 shift fails it. `Isa` gained
`Eq` and `Hash`: a carrier hands its token back, and a test wants to
compare it.

The slices were the third cut, and the measurement decided what the
plain methods do. In a build without flags, ns a call on Zen 5:

| words | `count_ones` Native / plain | `rank` Native / plain | `select` Native / plain |
|---|---|---|---|
| 1 | 1.05 / 1.26 | 1.50 / 1.48 | 5.69 / 1.39 |
| 16 | 6.51 / 2.48 | 6.50 / 3.11 | 14.12 / 2.93 |
| 1024 | 396.6 / 114.8 | 397.5 / 117.2 | 480.7 / 135.4 |

Plain is `dispatch!` on every call; the detection is one relaxed load
and a predicted branch, lost in the noise of a single word, where
POPCNT already beats the SWAR count. So `count_ones`, `rank`,
`select` and `for_each_position` ask per call, and section 11.10's
question of where slices stop detecting has the answer: nowhere.

The same run with `-C target-cpu=native` found the one mistake. A
token held outside, `select_in(k, X86V3)`, was half again as slow at
1024 words as `Native`: the trampoline compiles the loop for
x86-64-v3, and the build had AVX-512, which LLVM uses for the count.
A token below the build is a downgrade, so `detect()` answers
`Level::Native` when the build already proves x86-64-v3, and the
dispatch changes nothing there: `xs.select(k)` and
`xs.select_in(k, Native)` compile to one function (LLVM aliases the
symbols). The benchmark still shows them apart at some lengths, by
the same amount on every run; that is the timing loop inlining the
call differently at two call sites, not the library.

`Rank9` was the fourth cut and overturned 11.8's plan for it. Over
2^20 bits, 1024 random queries, ns a query, in the build without flags:

| query | `Native` | `dispatch!` per query | one `dispatch!` for the loop |
|---|---|---|---|
| `rank` | 2.25 | 1.63 | 1.27 |
| `select`, dense | 14.90 | 7.30 | 6.92 |
| `select`, sparse | 2.29 | 1.93 | 1.33 |

With `-C target-cpu=native` the three columns are the same to the
hundredth, the dispatch having folded to `Native`. So per-query
dispatch beats the old default in every build, and costs a third of a
nanosecond against the loop-wide one; plain `rank` and `select` ask
per call, `rank_in` and `select_in` take a token for the loop, and no
type parameter was needed. `build` is one pass over every word and
asks once: 20.4 µs to 12.5 on the dense bitmap, 34.4 to 25.6 on the
sparse, with the pass marked `inline(always)` so it compiles inside
the trampoline and not beside it.

`X86V4` was the fifth: x86-64-v4, VBMI and GFNI, from the same macro
as `X86V3`, since the two differ in their feature string and in
GFNI. Two consequences. The lanes under the token are GFNI's, so a
build without flags reverses the bits of sixteen bytes with one
`gf2p8affineqb` (codegen cell `portable`), and `tests/isa.rs` runs the
GFNI byte maps on this machine without a special build. And the batch
kernels stopped asking `cpu.rs` feature by feature: `cpu.rs` knows two
levels now, and the kernels take the level `detect` found, or what
the build proves on its own (the Miri cell with `+avx2` alone still
runs the AVX2 kernel). A CPU with VBMI and no GFNI, Cannon Lake and
nothing since, now gets the AVX2 kernels; the price of one level where
there were three flags.

`detect` answers with the higher of the build and the CPU: `Native`
where the build proves v4, `X86V4` where the CPU has it and the build
does not, then the same for v3. The first cut read the cache once per
question and asked twice per call; the `bmi2` snapshot showed three
extra calls in `Hilbert3::from_morton_in_place`, and one read of both
levels took them out and ten instructions more than the old code had.

`X86V2` is the macro once more, with BMI2 and PCLMULQDQ off: x86-64-v2
has POPCNT and PSHUFB and neither of those, so its primitives stay
broadword and it runs no batch kernel. What it buys is every count the
compiler derives (`Words`, `Rank9`) and the SSSE3 lanes, on Nehalem to
Ivy Bridge and the Atoms. Its detection comes before the XSAVE check,
since Nehalem has none. The one mistake it could make, a PEXT under a
v2 token, would run on every machine this is tested on and fault on
the ones it is for; the codegen cell asserts there is no PEXT under it,
which is the only place that can see it. FileCheck also taught that
a prefix and a colon in a comment's prose are a directive.

`Neon` stays out. NEON is in the baseline of every aarch64 target that
has it, so `Native` proves it already; where it is off
(`aarch64-unknown-none-softfloat`) there is no operating system to ask,
and the token could only come from `unsafe`. The aarch64 level worth
having is a different one: PMULL (the `aes` feature) is a carry-less
multiply, which would give `xor_scan` there what PCLMULQDQ gives it on
x86, and asking for it needs `getauxval`, so `std`, or a caller who
knows.

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
- Section 11, dispatch: pulp, <https://github.com/sarah-quinones/pulp>;
  fearless_simd, <https://github.com/linebender/fearless_simd>, and
  Shnatsel, *Safe SIMD in Rust, even on the inside*, 2026,
  <https://shnatsel.github.io/safe-simd-in-rust-even-on-the-inside/>;
  multiversion, <https://github.com/calebzulawski/multiversion>; Google
  Highway, <https://github.com/google/highway>; safe
  `#[target_feature]`, Rust 1.86 release notes, 2025.
- Langdale, Lemire. *Parsing Gigabytes of JSON per Second*. The VLDB
  Journal, 2019.
- Kogge, Stone. *A Parallel Algorithm for the Efficient Solution of a
  General Class of Recurrence Equations*. IEEE Transactions on
  Computers, 1973.
- Beneš. *Optimal Rearrangeable Multistage Connecting Networks*. Bell
  System Technical Journal, 1964.
