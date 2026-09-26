# hakmem: design notes

This document records how the crate's main decisions were made: what
was chosen, why, what else was tried or considered, and what is still
unknown. It is for people who want to extend the crate (a carrier, a
combinator, a backend), and for people who want to argue with it. The
API reference is the rustdoc and the worked examples are
`hakmem::cookbook`. It assumes you know what POPCNT, PEXT and a carry
chain do; Hacker's Delight chapters 2 and 5 are the short way there.

None of this is final. The crate is one person's learning project, and
several decisions below were reversed by a measurement after they were
written down. If you think one is wrong, open an issue and say why; a
benchmark or a counterexample helps, but an argument is enough to
start. Section 9 lists the questions we know are open.

Numbers were measured on one machine (AMD Zen 5, one thread) with the
benches in `benches/`. Treat them as evidence for the shape of a claim,
not as a promise about your machine.

## 1. What the crate is for

A bit instruction is a fixed parallel circuit over a word of 64 cells,
available as one O(1) operation. Each has a structural role:

| circuit | instruction | over the container |
|---|---|---|
| reduce (arg-min / arg-max) | TZCNT, LZCNT | priority encoder; `⌊log₂ x⌋` |
| reduce (count) | POPCNT | rank, the basis of succinct structures |
| scan | the adder's carry chain; PCLMULQDQ | prefix network; CLMUL is a scan whose operator is the constant |
| filter | PEXT | keep the cells a mask selects, in order |
| bijection | PDEP ∘ PEXT | scatter and gather on bits; Morton interleave |
| permutation | delta swap; a Beneš network of them | any fixed rearrangement of cells |

Chaining them is function composition, and the depth of the chain is
the depth of the circuit. Code written this way exists (simdjson's
classification masks, Hyperscan, Myers' edit distance, chess
bitboards, fusion trees), mostly as one-off kernels. The crate tries to
give the pieces names, types and laws, so that a kernel can be
assembled from them and checked.

Why bother: a miss to L2 or L3 costs about as much as 300 chained word
operations, and a current core retires four to six independent ones a
cycle. A chain of twelve operations that saves one pointer chase wins.
That is the reason a two-level bitmap beats a free list, and an
edit-distance column in a register beats a table in memory.

## 2. Decisions

### 2.1 Types carry the domain, not only the width

A Morton coordinate is not a `u64`, even though it fits in one.
`Dilated<W, D>` has no `Add`, because adding a plain integer to a
dilated one is the bug the type exists to prevent. It has `incr` and
`wrapping_add`, which fill the gaps with ones so the carry tunnels
across them (Raman and Wise).

We stopped where the type system stops helping. The mask of `compact`
and `expand` is data, and no type can say that two run-time masks are
equal. A witness type `{ bits, mask }` could tie an `expand` to its
`compact`, at the cost of a second word per value; we judged that too
expensive for what it proves. So `compact` and `expand` are plain
functions with a law (`compact_expand_roundtrip`), and types carry
provenance only where the mask is structural, as the stride of a
dilated integer is.

### 2.2 Laws are the exported API

`hakmem::laws` exports each law as a function `fn(...) -> bool` over
`W: Word`. The crate's tests call them; code downstream can call the
same functions over its own carriers and backends. A backend that fails
a law has a bug. We do not document a failing law as a caveat.

The laws also make the hardware paths safe to have. A primitive's
instruction and its portable definition are interchangeable because the
same laws hold for both, on the same inputs, in the same test run.
Every law is a rewrite rule, and a kernel may use either side of it.

Why exported rather than kept in the test suite: a carrier or a backend
written outside the crate needs an acceptance test, and the laws are
exactly that. This makes a law part of the semver contract (section 8),
which is a cost we accept.

### 2.3 One trait for the combinators

`Bits` holds every combinator, with a blanket implementation for every
`Word`, so `use hakmem::Bits;` is the whole import. The first draft had
one trait per domain (`Runs`, `SetView`, `Scan`, `Compact`, ...). A
kernel needed a dozen imports, and an umbrella trait cannot fix that,
because a supertrait's methods are not in scope through the subtrait.
The domain grouping survives as section headings inside `bits.rs`.

Primitives keep the names `std` uses (`count_ones`, `trailing_zeros`),
with the instruction names in the documentation, because a Rust
programmer searches for the `std` name.

### 2.4 Carriers: words, lanes and byte maps

`Word` is open: a carrier is one `impl` block, with portable defaults
for everything that can be derived (the bitwise operations, shifts,
wrapping arithmetic, counts, `pext`, `pdep`, `select_lowest`, the XOR
scans, `zip`, `unzip`). It is implemented for `u8` through `u128` and
for `Wide<N>`, which is `[u64; N]` treated as one word: carries
propagate across limbs and shifts cross limb boundaries, at `N` limb
operations per operation. `Wide<N>` is the test that a word need not be
one register. `myers::distance_in` was written against `Word` and runs
on `Wide<8>` for 512-byte patterns without a change.

A vector register is not a `Word`: its lanes do not carry into each
other, so the laws of the carry chain do not hold across it. It gets
its own trait, `Lanes`, with the operations that are lawful lane by
lane (bitwise, wrapping add and subtract, shifts, unsigned compares,
the 16-entry table lookup) and one bridge, `to_bitmask`, which folds a
lane mask into a `Word`. That bridge is where simdjson's first stage
hands its masks to its second. The carriers are `U8x8` (SWAR in a
`u64`, used by the exhaustive tests) and sixteen lanes per instruction
set (SSSE3, NEON, or two SWAR halves).

Under the lanes sits `Affine8`: affine maps on the bits of a byte, 8×8
matrices over GF(2) with a constant. Every byte shift, rotate, reversal
and NOT is one, and composition is matrix multiplication, so a chain
of them folds to one map at compile time. `Lanes::affine` applies a map
to every lane with one GFNI instruction, or with two nibble lookups by
linearity where GFNI is missing.

### 2.5 Batches work on words the caller owns

A batch operation takes a slice of keys and converts it in place:
`Hilbert2::<u64>::from_morton_in_place(&mut keys)`. The caller fills
the keys from whatever layout its points are in. This keeps the crate
from committing to a point type, gives batches the same shape as the
slice operations and `Rank9` (words in, caller's storage, no
allocation), and leaves room to add forms over coordinate columns
later without breaking anything.

That room was used where columns turned out to be faster rather than
merely convenient. `Hilbert3::<u64>::encode_columns` reads three
columns straight into byte planes (`vpmultishiftqb` at offsets `l`,
`l - 1` and `l - 2`), so the Morton code in between would only be built
to be taken apart. `Morton2` and `Hilbert2` have columns too. They are
inherent functions on the widths that have kernels (`u32`, `u64`), not
generic over `Word`: dispatching on the width of a generic `W` would
need an unsafe cast of the slice, and we keep `unsafe` to intrinsic
calls and kernel dispatch.

### 2.6 Conventions

Settled before 0.2, while breaking was cheap. A range of coordinates
or keys is a `RangeBounds`, not a pair, since three pairs of one type
in a row invite swapped arguments. A function that fills a caller's
buffer returns the filled part, as `char::encode_utf8` does. Copy
conversions are `to_`, as `f64::to_bits`. A choice between two
behaviours is two functions (`fill_block`, `clear_block`), not a
`bool`. The curves share a trait so an index can change curve by
changing a type. `Word` is ordered, hashable and printable in binary,
because asking for that later would break every carrier written in the
meantime.

## 3. Choosing instructions

This is the decision that changed most between 0.1 and 0.2, and the
one we are least sure is finished.

### 3.1 The 0.1 rule and what it missed

In 0.1 the build chose. `#[cfg(target_feature = "bmi2")]` selected PEXT
at compile time in one module; without the feature every primitive had
a portable definition with the same contract. Run-time dispatch was
reserved for whole batch kernels, where one CPUID check is free next to
thousands of keys. The argument against dispatching per word still
holds: a predicted branch and a call around a three-cycle instruction
cost more than the instruction.

What the rule missed: the usual Rust way to dispatch is to compile a
hot loop under `#[target_feature]` and pick the copy after a CPUID
check. A user function marked `#[target_feature(enable = "bmi2")]` that
calls `x.compact(m)` compiled, in a build without flags, to 291
instructions and no PEXT, because `cfg(target_feature)` is evaluated
once for the whole crate and the caller's attribute never reaches it.
The idiomatic dispatch got the portable path. It also hid which batch
kernel ran, made the `portable` feature switch the whole dependency
graph, and let one test run see only one path.

### 3.2 Instruction sets as values

0.2 adds a zero-sized token per instruction-set level, `I: Isa`. A
caller dispatches once at the top of a hot loop (`dispatch!`) and
passes the token down; primitives are methods on it and the lane
carrier is its associated type (`I::U8x16`). `Isa::run` is a trampoline
compiled with the level's `#[target_feature]`, so the closure and
everything it inlines get the level's instructions. Under this
sit leaves (`hakmem::x86::bmi2::pdep`), safe `#[target_feature]`
functions tagged with exactly the features they use, for code under
someone else's dispatch.

Choices within this, and why:

- **Levels, not features.** Tokens are the psABI levels plus the
  extensions the crate uses: `X86V2`, `X86V3` (with PCLMULQDQ), `X86V4`
  (with VBMI and GFNI). Every separate feature doubles the combinations
  a dispatch monomorphises; pulp, fearless_simd, multiversion and
  Highway settled on levels for the same reason. The cost is visible:
  a CPU with VBMI and no GFNI (Cannon Lake) gets the AVX2 kernels.
- **`Native` is its own token, not an alias for a level.** The first
  plan made it the highest level the build proves. Builds are not
  levels: the test matrix's `+bmi2,+pclmulqdq,+ssse3,+avx2` lacks parts
  of x86-64-v3, so the alias would have been `Portable` and PEXT would
  have gone broadword in a build that asked for BMI2. `Native` chooses
  primitive by primitive what the build proves, as 0.1 did.
- **`detect()` never downgrades.** A token below the build compiles the
  loop for less than the build has; with `-C target-cpu=native`,
  `select_in(k, X86V3)` was half again as slow at 1024 words, because
  the build had AVX-512 and the token did not. `detect()` returns
  `Native` when the build already proves the level.
- **Slices and `Rank9` ask the CPU once per call.** We expected to need
  a token held in the structure. The measurement said otherwise: the
  check is one relaxed load and a predicted branch. In a build without
  flags, ns per call:

  | words | `count_ones` Native / dispatched | `select` Native / dispatched |
  |---|---|---|
  | 1 | 1.05 / 1.26 | 5.69 / 1.39 |
  | 1024 | 396.6 / 114.8 | 480.7 / 135.4 |

  For `Rank9` queries over 2^20 bits, dispatch per query beat the old
  default in every case and cost a third of a nanosecond against one
  `dispatch!` around the whole loop, so no type parameter was needed.
  `rank_in` and `select_in` take a token for callers who want that last
  third.
- **`Isa` is sealed.** Opening it to user levels later breaks nothing;
  closing it later would.
- **No NEON token.** NEON is in the baseline of every aarch64 target
  that has it, so `Native` proves it already. The aarch64 level worth
  adding is PMULL, a carry-less multiply for the XOR scans.
- **Kernels and soft-float targets.** `x86_64-unknown-none` turns SSE
  off, and there CPUID describes what userspace may use, not the
  kernel. Tokens and lane carriers require `sse2` in the build, and
  `detect()` answers `Portable` without asking. BMI2 uses general
  registers only, so a kernel built with `+bmi2` keeps PEXT through
  `Native`.
- **Zen 1 and 2** report BMI2 but run PDEP and PEXT in microcode, about
  18 cycles. The `portable` feature turns the instructions off for
  them. `detect()` cannot see this from feature bits; it could from the
  CPU family, and whether it should is open (section 9).

The known trap: anything the dispatched closure calls must inline into
it. A helper left out of line is compiled without the level's features,
and each primitive in it becomes a call. Trait methods cannot be safe
`#[target_feature]` functions, so the compiler cannot catch this for
generic code. We check it after the fact in the codegen cells (section
5) and have hit it ourselves (section 6).

## 4. Kernels worth explaining

Most combinators are textbook (section 7). These are the ones where we
chose among alternatives.

**Portable `select` and `compact` stay constant time.** A loop that
clears `k` set bits beats Vigna's broadword select for `k` below about
28 on Zen 5, and a loop over a mask's set bits beats the
parallel-suffix compress below about 17 set bits:

| `select` on `u64`, per word | dense | sparse |
|---|---|---|
| PDEP | 1.1 ns | |
| Vigna broadword | 4.7 ns | |
| clear-lowest loop, `k` times | 2.7 ns | 1.05 ns |

We kept the broadword definitions for constant time and no
data-dependent latency. A hybrid for small `k`, or the 2 KB table the
`broadword` crate uses for the last step (which makes it 14 % faster
than our portable path), are reasonable choices we have not made.

**The 2D Hilbert encode is a carry chain over GF(4).** Read with the
coordinates as input, the per-level frame maps are affine on `GF(2)²`,
with linear parts generating `GL(2, 2) ≅ S₃`. The adder's `(g, p)`
composition is `Aff(1, GF(2))`; this is `Aff(1, GF(4))`, so the same
Kogge–Stone shape works with a four-element field. The log-depth
construction is rawrunprotected's (2016); what the crate adds is the
statement, the laws, and the generality over carriers. The decode is
two suffix XORs over the Morton code (Hacker's Delight 16-2).

**The 3D Hilbert encode is a table.** The curve's frames form
`A₄ ≅ AGL(1, 4)`, which gives the decode a scan (the 2D encode's), but
the encode's transition monoid has no cheap representation, so a level
of the encode is a lookup: a twelve-state machine memoised into 96 bytes,
built at compile time from the algebra. The translations `V₄ ⊲ A₄` act
on octants as a XOR and commute with the machine, so the 96 entries
factor as 24 between two XORs, which fits two 16-entry shuffles; the
AVX2 kernel relies on that.

**The batch kernels are the level machine applied to a register of
keys.** With AVX-512 VBMI a step is one `vpermi2b` per register: in 2D
through 128 entries, three levels a step; in 3D through the 96-byte
table, one level a step, and on `u64` through byte planes of 64 keys.
NEON uses `tbl`, AVX2 alone the factored 3D table. The numbers are in
the README.

**`cover` counts the depth instead of searching for it.** A rectangle
becomes at most `budget` sorted key ranges. A run of the exact cover
starts where the curve's predecessor leaves the rectangle; on Z-order
that is a borrow and a count of arithmetic progressions per level, on
Hilbert a step entering across a side, counted per frame. A guess from
the perimeter and a step or two find the depth. The walks go three
levels a step with 64 descendants as one `u64` mask per frame, and the
threshold between gaps closed and kept is found by size class (bit
length and three mantissa bits, the classes of an offset allocator),
with a quickselect only for a class holding several values. At the
same number of ranges, GeoMesa's `zranges` leaves 2 to 12 % more
over-cover and takes 2 to 8 times as long. The S2 region coverer
approximates in cells rather than ranges, so the comparison is looser:
20 to 80 µs a rectangle.

**`intersects` is one descent.** A node is an interval of keys and a
square of cells at once, so only the nodes on the paths of the block's
two ends are partly in the interval.

## 5. How we check it

**Laws.** Property tests on `u8` through `u128` and on `Wide<2>` and
`Wide<3>`, with and without the hardware paths, and exhaustive sweeps
at 8 and 16 bits: every `u8`, every pair of `u8`, every `u16` against
structured masks, every Morton coordinate of a 256 × 256 grid. Every
bug found in the arithmetic so far was found by a sweep, not by random
inputs. The carry into `find_escaped` from the previous word may only
affect bit 0; an earlier version flipped the whole word, and the sweep
found it at `x = 2, carry = 1`, a case random 64-bit inputs do not
reach. Kernels without a law of their own (`myers`) are compared with
the textbook algorithm.

**CI.** Every check is a Nix derivation built without network, and
only cells whose output is not already in the binary cache are built
(`nix/matrix.nix`, `.github/plan.sh`). The README's "How the checks got
here" explains why each row exists.

**Codegen.** Tests pass whether a primitive inlines or not, and the
tracked benches build with `-C target-cpu=native`, where PEXT is one
instruction however it inlines. So we check the compiler's output
directly. FileCheck holds the README's instruction claims to the asm.
Two more cells read the benches' linked binaries, built as a dependency
would be (release, sixteen codegen units): one lists the hakmem
functions that survived as functions of their own, the other breaks
every timed loop down by the hakmem function each instruction was
inlined from, with llvm-mca's cycles (`codegen/src/bin/loops/`). Clippy's
`missing_inline_in_public_items` makes every public function state
whether it is `#[inline]`.

**Releases.** `nix run .#publish` uploads only a tagged, dated, clean
tree whose tarball holds the file list in `release/package.txt` and
passed the MSRV build.

## 6. Mistakes, and what they changed

Kept here because each one changed a decision or a check, and because
the next person is likely to make the same ones.

- **A lost `#[inline]` made the 2D Hilbert decode eight times slower**
  in a build without flags, and every check stayed green. Routing
  `Word` through the tokens turned `pext` into a provided trait method
  and left the attribute behind with the deleted impl. `perf` found it.
  This is why the codegen cells read linked binaries instead of one
  codegen unit, and why the inline lint is on.
- **The first pass of that lint found `Rank9::rank` and `select`**
  without `#[inline]`, out of reach of any caller outside the crate;
  with it `rank` got 6 to 8 % faster and `select` 7 % on dense bits,
  39 % on sparse. On the Myers entry points the
  same attribute made short patterns 13 % slower by moving LLVM's split
  of the function, so the lint cannot be satisfied by adding
  `#[inline]` everywhere.
- **`array::map` with a closure inside `dispatch!`** stayed out of line
  and lost the level's features; written as a plain loop, the same code
  ran 2.5 times faster.
- **The Morton decode lost two README rows to the incumbents.** Part
  was real: it masked and shifted before two PEXT that ignore the bits
  outside their mask anyway (now `Word::unzip`). The rest was our
  bench, which stored coordinates twice as wide as the incumbents'
  rows did.
- **An optimisation estimate read the codegen tree wrong.** Two inlined
  calls with the same name had been summed into one node, so a
  "twice as fast" encode was 11 % slower. The tree now marks such nodes.
- **FileCheck's `CHECK-NOT: call` after an instruction does not cover
  the code before it.** A review caught it; the lines now sit on both
  sides.
- **The 0.2 package almost shipped the public-API snapshots**, and the
  MSRV cell was testing a tarball without the licences, built from a
  filtered tree. The package's file list is now a checked snapshot.

## 7. Provenance

Nothing in this table is new here; the crate names each item, gives it
a type and a law, and tests it on every carrier.

| combinator | source |
|---|---|
| `run_starts`, `has_run` (halving chain) | Hacker's Delight §6-2, figure 6-5 |
| `longest_run` | Hacker's Delight §6-3, as a binary search over `has_run` |
| `zero_bytes`, `bytes_eq`, `bytes_lt`, `sum_bytes` | Hacker's Delight §6-1 |
| `lowest_set_mask`, `below_lowest_set`, `up_to_lowest_set`, `clear_lowest_run`, `prefix_or` as `x \| -x` | Hacker's Delight chapter 2 |
| `round_up_pow2`, `log2_floor` | Hacker's Delight §3-2, §5-3 |
| `next_same_popcount` | HAKMEM item 175 (Gosper) |
| `gray_encode`, `gray_decode`, `suffix_xor` | Hacker's Delight chapter 13 |
| `compact`, `expand`, `zip`, `unzip` | Hacker's Delight §7-1, §7-2, §7-4, §7-5; BMI2 PEXT / PDEP |
| `delta_swap`, `board8` permutations | Knuth, TAOCP 7.1.3; Hacker's Delight §7-3; the bitboard literature |
| `select` with PDEP | Pandey, Bender and Johnson, 2017 |
| `select` without PDEP | Vigna, 2008 |
| `prefix_xor` by carry-less multiply; `find_escaped` | Langdale and Lemire, simdjson, 2019 |
| `fill_up`, `fill_down` | Kogge and Stone, 1973, as used for sliding attacks on bitboards |
| `Dilated`, `Morton2` | Morton, 1966; Raman and Wise, 2008 |
| `Hilbert2::to_morton` | Hacker's Delight 16-2, the parallel-prefix form of Lam and Shapiro's state machine (1994) |
| `Hilbert2::from_morton` | rawrunprotected, *2D Hilbert curves in O(log n)*, 2016 |
| `Hilbert3` | the curve of rawrunprotected's 3D tables (2016, 2020) |
| `myers::distance_in`, `myers::search` | Myers, 1999; Hyyrö's formulation |
| `rank9::Rank9` | Vigna, 2008: rank9, and a select inventory in the shape of select9 |
| `lanes::U8x8` add, subtract, compare | Hacker's Delight 2-18, 6-1 |
| `lanes::Lanes::lut16`, the nibble classifier | simdjson, after Muła's PSHUFB lookups |
| `affine::Affine8`, `Lanes::affine`, byte shifts and reversal | GFNI's `gf2p8affineqb`: Wunkolo, 2020; the nibble split is Muła's |
| `Bits::ternary`, `bits::truth_table` | VPTERNLOG |
| `Bits::signed_add_overflows`, `signed_sub_overflows` | Hacker's Delight 2-13; the lane form: Wunkolo, 2025 |
| `Lanes::avg_round`, `avg_floor` | Hacker's Delight 2-5; PAVGB; Wunkolo, 2022 |
| `Bits::next_subset`, `subsets` | the carry-rippler, chess programming folklore |
| `Bits::gather`, `gather_factor` | multiply as parallel shift-and-add; Kindergarten bitboards (Isenberg, 2007) |

Things we have not found written down elsewhere. If you know a source,
we would like to cite it.

- **A rectangular run as a separable erosion** (`grid`): the halving
  chain down the rows, then along each row, `O(log w + log h)` word
  operations per row. Morphologists know separable erosion; we have not
  seen it used for first-fit tile allocation.
- **The composition and homomorphism laws as one checked family**, for
  example `run_starts(a) ∘ run_starts(b) = run_starts(a + b − 1)`, the
  XOR-linearity of `compact`, `expand` and the Gray codes, and
  `pdep(pdep(x, n), m) = pdep(x, pdep(n, m))`. Each is folklore or a
  one-line proof.
- **The statement that the Hilbert encode is Kogge–Stone over GF(4)**
  (section 4). The general rule behind it, that a state machine over a
  word is a broadword kernel exactly when its transition monoid has a
  cheap representation, is a theorem for the aperiodic and modular
  cases (Bergeron and Hamel 2001, Serre 2004, Paperman, Salvati and
  Soyez-Martin 2023) and, as far as we know, open for the group case,
  which is where the Hilbert encode sits.

## 8. Compatibility

`no_std`, no dependencies, stable Rust. MSRV 1.89 (edition 2024 needs
1.85, `cast_signed` 1.87, the GFNI intrinsics 1.89), checked against the
packaged tarball rather than the repository.

- Adding a method to `Bits`, or a primitive with a default to `Word`,
  is not breaking: `Bits` is implemented only through the blanket impl.
  Adding a required primitive to `Word` is, since carriers exist
  outside the crate.
- Changing what a law states is breaking. A kernel downstream may rely
  on either side of it.
- Results the documentation calls unspecified (for example
  `Word::select_lowest` with `k` at or above the population) are not
  covered. `Bits::select` returns `Option` and is.

## 9. Open questions

Each of these is a question, not a plan. If you have an answer, a
measurement, or a reason the question is wrong, an issue is welcome.

**API**

- Should `Rank9` hold a token chosen once at construction? Today
  `rank` checks the level on every query; in the portable build its
  loop is 90 instructions, 17 of them the cached check and the rest the
  rank and calls to three trampolines. Holding the token would remove
  the check but put a type parameter on every user.
- Should `detect()` treat Zen 1 and 2 as a level of their own, or keep
  that to the `portable` feature?
- Should `Isa` be opened to user-defined levels?
- Wider lanes for 64-byte blocks, which is how simdjson and the scan
  engines work: `codegen/src/bin/loops/scan.rs` has a draft of `U8x32`
  and `U8x64` behind tokens, 1.7 times the sixteen-lane speed on a full
  classification. Missing: the tokens as `Isa` levels, NEON pairs, and
  a better SWAR fallback (`Swar16::lut16_nibbles` runs at a tenth of
  PSHUFB).

**Performance**

- `Rank9::select` keeps eight bounds checks and five slice-range
  checks. Three are in the `Tiny` span, read under `hi - lo >= 2` and
  `>= 3`, which LLVM does not relate to the slice length; a single
  assert does not remove them. A fixed window into `counts` would, if
  its padding always covers it.
- Under `X86V2` (no BMI2) the `pext` in a `dispatch!` body is
  `compress_broadword`, and it stays a call out of the trampoline.
  Should a broadword leaf that large be forced inline?
- A hybrid portable `select` that loops for small `k` (section 4).
- The Morton columns on AVX-512: about half the cycles of the AVX2
  kernel in cache, and no gain past about a million points, where
  memory is the limit. It belongs with the wider lanes.
- `cover` spends most of its time emitting runs; a fifth of its cycles
  are mispredicted branches set by the rectangle's edge. Keeping the
  sink's state in registers made no difference. Not tried: extracting a
  mask's runs a fixed number at a time, as simdjson does, or one walk
  instead of two at the cost of a coarser cover.
- The 3D factored kernel on SSSE3 and NEON, and the NEON paths in
  general, are checked in CI but not measured.

**Research**

- No corner-to-corner self-similar 3D curve whose sub-cubes are cube
  symmetries has a log-depth encode: of the 10 752 such curves, the
  frame group is `A₄` for 42 and `S₄` for the rest, never abelian.
  Face-gated and non-self-similar curves were not searched.
- Is there a cheap representation for the group case of the
  transition-monoid rule above, and would it give the 3D encode a scan?
- Candidates for the crate, roughly in this order: a transition-monoid
  analyser in `laws`, balanced parentheses next to `rank9`,
  bit-parallel LCS, shift-and.

## Deliberately out

- **Data-dependent control flow and pointer chasing.** The domain is
  straight-line, fixed-width kernels, the same boundary as bitslicing.
- **Tables**, except the 96-byte 3D Hilbert encode (section 4).
- **Bit storage.** That is `bitvec`. `Rank9` is the one structure, and
  it owns nothing: the caller supplies the directory's words, which the
  maintained crates (`sux`, `sucds`, `vers-vecs`) do not allow.
- **Promises about autovectorisation.** A combinator compiles to a
  known instruction or a documented fallback.
- **Big integers, division by constants, square roots, CRC, floating
  point.** A different algebra.
- **Banded Myers.** For two long strings a few edits apart,
  `triple_accel` is about twice as fast, because it computes only a
  band. Cookbook recipe 7 sketches the variant for anyone who needs it.

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
- Dispatch: pulp, <https://github.com/sarah-quinones/pulp>;
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
