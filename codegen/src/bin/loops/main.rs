//! What the benches' innermost loops are made of: for every loop a bench
//! times, its instructions by the hakmem function they were inlined from,
//! the calls it still makes, and llvm-mca's cycles per iteration.
//!
//!   loops --bench hilbert --bin <exe> --dis <objdump -d> --got <got.awk out>
//!         --root <hakmem source dir> --cpus znver5,x86-64-v3
//!         [--detail <file>] [--check]
//!   loops --dis <objdump -d> --speed      the classifier per width, GB/s
//!
//! The summary goes to stdout, a few lines per loop: the snapshot. The
//! detail file holds the whole inline tree and the loop's instructions,
//! each with the hakmem function it came from.
//!
//! The listing is read the way hakmem reads text (scan.rs): the newline
//! mask of every sixty-four bytes on the widest registers the CPU has,
//! the other classes of a block when a line in it is read, simdjson's
//! nibble classifier; fields by `trailing_zeros` on those masks,
//! addresses by one `compact` of eight hex digits. A tool that checks
//! hakmem must not trust it blindly: `--check` holds every level's masks
//! to a byte-by-byte scan of the same text.
//!
//! Loops are found the textbook way, dominators as bitsets: a back edge
//! goes to a block that dominates its source, the loop is what reaches
//! the source without passing the header, and a loop is innermost when no
//! other loop's blocks are a proper subset of its own.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::{Read as _, Write as _};
use std::process::{Command, Stdio};

use hakmem::prelude::*;

// ---------------------------------------------------------------------------
// Stage one: bytes to masks (scan.rs).

mod scan;
use scan::{COLON, GT, HASH, LT, Level, Masks, TAB};

/// Text with its newline mask, and the other classes of a block when a
/// line asks: a function is one line in thousands the tool reads, so
/// classifying everything was five sixths of the work for nothing. The
/// first or last byte of a class in a range is a `trailing_zeros` or
/// `leading_zeros` a word, and lines are short, so one block cached is
/// nearly every lookup.
struct Text<'a> {
    bytes: &'a [u8],
    level: Level,
    nl: Vec<u64>,
    cache: std::cell::Cell<(usize, Masks)>,
    classified: std::cell::Cell<usize>,
}

impl<'a> Text<'a> {
    fn new(bytes: &'a [u8], level: Level) -> Self {
        let nl = level.newlines(bytes);
        let cache = std::cell::Cell::new((usize::MAX, [0; 6]));
        Self {
            bytes,
            level,
            nl,
            cache,
            classified: std::cell::Cell::new(0),
        }
    }

    fn word(&self, w: usize, k: usize) -> u64 {
        let (at, masks) = self.cache.get();
        if at == w {
            return masks[k];
        }
        let block = self.bytes.get(w * 64..w * 64 + 64).map_or_else(
            || scan::pad(&self.bytes[w * 64..]),
            |b| *b.as_array::<64>().unwrap(),
        );
        let masks = self.level.classes(&block);
        self.classified.set(self.classified.get() + 1);
        self.cache.set((w, masks));
        masks[k]
    }

    fn find(&self, k: usize, from: usize, to: usize) -> Option<usize> {
        if from >= to {
            return None;
        }
        let mut w = from / 64;
        let mut bits = self.word(w, k) & (!0u64 << (from % 64));
        loop {
            if bits != 0 {
                let p = w * 64 + bits.trailing_zeros() as usize;
                return (p < to).then_some(p);
            }
            w += 1;
            if w * 64 >= to {
                return None;
            }
            bits = self.word(w, k);
        }
    }

    fn rfind(&self, k: usize, from: usize, to: usize) -> Option<usize> {
        if from >= to {
            return None;
        }
        let mut w = (to - 1) / 64;
        let mut bits = self.word(w, k) & (!0u64 >> (63 - (to - 1) % 64));
        loop {
            if bits != 0 {
                let p = w * 64 + 63 - bits.leading_zeros() as usize;
                return (p >= from).then_some(p);
            }
            if w * 64 <= from {
                return None;
            }
            w -= 1;
            bits = self.word(w, k);
        }
    }

    /// Every line, without its newline, walking the newline mask.
    fn lines(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let mut start = 0;
        self.nl
            .iter()
            .enumerate()
            .flat_map(move |(w, &m)| {
                let mut bits = m;
                std::iter::from_fn(move || {
                    if bits == 0 {
                        return None;
                    }
                    let end = w * 64 + bits.trailing_zeros() as usize;
                    bits = bits.clear_lowest_set();
                    Some(end)
                })
            })
            .map(move |end| {
                let line = (start, end);
                start = end + 1;
                line
            })
    }

    fn str(&self, from: usize, to: usize) -> &'a str {
        std::str::from_utf8(&self.bytes[from..to]).unwrap_or("?")
    }
}

/// Up to sixteen hex digits, eight at a time: every digit's nibble in its
/// byte (`'a'..='f'` have bit 6 and need 9 more), then one `compact`
/// gathers the eight nibbles in order, PEXT or its broadword ladder.
fn hex(digits: &[u8]) -> u64 {
    debug_assert!(digits.len() <= 16 && digits.iter().all(u8::is_ascii_hexdigit));
    let mut v = 0u64;
    for chunk in digits.rchunks(8).rev() {
        let mut b = [b'0'; 8];
        b[8 - chunk.len()..].copy_from_slice(chunk);
        let x = u64::from_be_bytes(b);
        let nibbles = (x & 0x0F0F_0F0F_0F0F_0F0F) + ((x >> 6) & 0x0101_0101_0101_0101) * 9;
        v = (v << (4 * chunk.len())) | nibbles.compact(0x0F0F_0F0F_0F0F_0F0F);
    }
    v
}

fn trim_end(bytes: &[u8], from: usize, mut to: usize) -> usize {
    while to > from && bytes[to - 1] == b' ' {
        to -= 1;
    }
    to
}

// ---------------------------------------------------------------------------
// Stage two: lines to functions.

#[derive(Debug)]
struct Insn<'a> {
    addr: u64,
    mnem: &'a str,
    ops: &'a str,
    /// A branch's target, when it is an address.
    target: Option<u64>,
    /// A call's callee, by name.
    callee: Option<String>,
}

struct Func<'a> {
    name: &'a str,
    insns: Vec<Insn<'a>>,
}

/// The functions whose names pass `keep`, with their instructions.
/// Header: `<16 hex digits> <name>:`; instruction:
/// `  <hex>:<spaces>\t<mnemonic>[\t<operands>][ # <comment>]`.
fn functions<'a>(
    text: &Text<'a>,
    keep: impl Fn(&str) -> bool,
    got: &HashMap<u64, String>,
) -> Vec<Func<'a>> {
    let b = text.bytes;
    let mut out: Vec<Func<'a>> = Vec::new();
    let mut inside = false;
    for (s, e) in text.lines() {
        if s == e {
            continue;
        }
        if b[s].is_ascii_hexdigit() {
            inside = false;
            // `<16 hex digits> <name>:`, so the name is where it is and a
            // header's block is never classified.
            if e - s < 20 || &b[s + 16..s + 18] != b" <" || &b[e - 2..e] != b">:" {
                continue;
            }
            let name = text.str(s + 18, e - 2);
            if keep(name) {
                out.push(Func {
                    name,
                    insns: Vec::new(),
                });
                inside = true;
            }
            continue;
        }
        if !inside || b[s] != b' ' {
            continue;
        }
        let Some(colon) = text.find(COLON, s, e) else {
            continue;
        };
        let a = b[s..colon]
            .iter()
            .position(|&c| c != b' ')
            .map_or(colon, |i| s + i);
        let Some(t1) = text.find(TAB, colon, e) else {
            continue;
        };
        let hash = text.find(HASH, t1, e);
        let body_end = trim_end(b, t1, hash.unwrap_or(e));
        let t2 = text.find(TAB, t1 + 1, body_end);
        let mnem = text.str(t1 + 1, t2.unwrap_or(body_end));
        let ops = t2.map_or("", |t| text.str(t + 1, body_end));
        let ops_at = t2.map_or(body_end, |t| t + 1);
        let direct = ops.starts_with("0x");
        let target = if mnem.starts_with('j') && direct {
            let digits = &b[ops_at + 2..ops_at + 2 + ops[2..].find(' ').unwrap_or(ops.len() - 2)];
            Some(hex(digits))
        } else {
            None
        };
        let callee = if !mnem.starts_with("call") {
            None
        } else if direct {
            let (lt, gt) = (
                text.find(LT, ops_at, body_end),
                text.rfind(GT, ops_at, body_end),
            );
            lt.zip(gt).map(|(l, g)| text.str(l + 1, g).to_owned())
        } else {
            // `*0x..(%rip)  # 0x<slot> <nearest symbol>`: the slot names it.
            hash.and_then(|h| {
                let c = text.str(h + 1, e).trim_start();
                let slot = c.strip_prefix("0x")?.split(' ').next()?;
                Some(
                    got.get(&hex(slot.as_bytes()))
                        .cloned()
                        .unwrap_or_else(|| c.to_owned()),
                )
            })
            .or_else(|| Some(ops.to_owned()))
        };
        let insn = Insn {
            addr: hex(&b[a..colon]),
            mnem,
            ops,
            target,
            callee,
        };
        if let Some(f) = out.last_mut() {
            f.insns.push(insn);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Stage three: control flow, dominators, innermost loops.

/// A set of blocks, one bit each.
#[derive(Clone, PartialEq, Eq)]
struct Set(Vec<u64>);

impl Set {
    fn empty(n: usize) -> Self {
        Self(vec![0; n.div_ceil(64)])
    }
    fn full(n: usize) -> Self {
        let mut s = Self(vec![!0; n.div_ceil(64)]);
        if !n.is_multiple_of(64) {
            *s.0.last_mut().unwrap() = u64::low_ones(n as u32 % 64);
        }
        s
    }
    fn has(&self, i: usize) -> bool {
        self.0[i / 64].bit(i as u32 % 64)
    }
    fn insert(&mut self, i: usize) {
        self.0[i / 64] |= 1 << (i % 64);
    }
    fn and(&mut self, o: &Self) {
        self.0.iter_mut().zip(&o.0).for_each(|(a, b)| *a &= b);
    }
    fn subset_of(&self, o: &Self) -> bool {
        self.0.iter().zip(&o.0).all(|(a, b)| a & !b == 0)
    }
    fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().enumerate().flat_map(|(w, &bits)| {
            let mut bits = bits;
            std::iter::from_fn(move || {
                (bits != 0).then(|| {
                    let i = w * 64 + bits.trailing_zeros() as usize;
                    bits = bits.clear_lowest_set();
                    i
                })
            })
        })
    }
}

fn ends_flow(mnem: &str) -> bool {
    mnem.starts_with("ret") || mnem == "ud2" || mnem == "int3" || mnem.starts_with("jmp")
}

/// The innermost loops of `f`, each as the indices of its instructions
/// in address order.
fn innermost_loops(f: &Func) -> Vec<Vec<usize>> {
    let insns = &f.insns;
    if insns.is_empty() {
        return Vec::new();
    }
    let at: HashMap<u64, usize> = insns.iter().enumerate().map(|(i, x)| (x.addr, i)).collect();
    // Leaders: the entry, every branch target inside, whatever follows a branch.
    let mut leader = vec![false; insns.len()];
    leader[0] = true;
    for (i, x) in insns.iter().enumerate() {
        if let Some(&t) = x.target.as_ref().and_then(|t| at.get(t)) {
            leader[t] = true;
        }
        if (x.mnem.starts_with('j') || ends_flow(x.mnem)) && i + 1 < insns.len() {
            leader[i + 1] = true;
        }
    }
    let starts: Vec<usize> = (0..insns.len()).filter(|&i| leader[i]).collect();
    let n = starts.len();
    let block_of = |i: usize| starts.partition_point(|&s| s <= i) - 1;
    let end = |b: usize| starts.get(b + 1).copied().unwrap_or(insns.len());
    let mut succ = vec![Vec::new(); n];
    let mut pred = vec![Vec::new(); n];
    for b in 0..n {
        let last = &insns[end(b) - 1];
        let jump = last.target.and_then(|t| at.get(&t)).map(|&t| block_of(t));
        let falls = !ends_flow(last.mnem) && end(b) < insns.len();
        for s in jump.into_iter().chain(falls.then_some(b + 1)) {
            succ[b].push(s);
            pred[s].push(b);
        }
    }
    // Reverse postorder from the entry; blocks it never reaches stay out.
    let mut order = Vec::with_capacity(n);
    let mut seen = Set::empty(n);
    let mut stack = vec![(0usize, 0usize)];
    seen.insert(0);
    while let Some((b, k)) = stack.pop() {
        if let Some(&s) = succ[b].get(k) {
            stack.push((b, k + 1));
            if !seen.has(s) {
                seen.insert(s);
                stack.push((s, 0));
            }
        } else {
            order.push(b);
        }
    }
    order.reverse();
    // dom(b) = {b} ∪ ⋂ dom(p): a word of AND per 64 blocks per edge.
    let mut dom = vec![Set::full(n); n];
    dom[0] = Set::empty(n);
    dom[0].insert(0);
    let mut changed = true;
    while changed {
        changed = false;
        for &b in order.iter().skip(1) {
            let mut d = Set::full(n);
            for &p in pred[b].iter().filter(|&&p| seen.has(p)) {
                d.and(&dom[p]);
            }
            d.insert(b);
            if d != dom[b] {
                dom[b] = d;
                changed = true;
            }
        }
    }
    // Natural loops, one per header: what reaches a back edge's source
    // without passing the header.
    let mut loops: HashMap<usize, Set> = HashMap::new();
    for &u in &order {
        for &h in &succ[u] {
            if dom[u].has(h) {
                let body = loops.entry(h).or_insert_with(|| {
                    let mut s = Set::empty(n);
                    s.insert(h);
                    s
                });
                let mut work = vec![u];
                while let Some(x) = work.pop() {
                    if !body.has(x) {
                        body.insert(x);
                        work.extend(pred[x].iter().copied().filter(|&p| seen.has(p)));
                    }
                }
            }
        }
    }
    let bodies: Vec<&Set> = loops.values().collect();
    let mut inner: Vec<Vec<usize>> = bodies
        .iter()
        .filter(|l| !bodies.iter().any(|m| m != *l && m.subset_of(l)))
        .map(|l| l.iter().flat_map(|b| starts[b]..end(b)).collect())
        .collect();
    inner.sort();
    inner
}

// ---------------------------------------------------------------------------
// Stage four: whose code it is, and what it costs.

/// A frame of an inline stack: name and file, innermost first.
type Stack = Vec<(String, String)>;

fn symbolize(bin: &str, addrs: &[u64]) -> Vec<Stack> {
    let mut child = Command::new("llvm-symbolizer")
        .args(["--obj", bin, "--inlining", "--demangle"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("llvm-symbolizer on PATH");
    let mut input = String::new();
    for a in addrs {
        let _ = writeln!(input, "{a:#x}");
    }
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let mut out = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    writer.join().unwrap().unwrap();
    child.wait().unwrap();
    let stacks: Vec<Stack> = out
        .split("\n\n")
        .filter(|s| !s.trim().is_empty())
        .map(|block| {
            let lines: Vec<&str> = block.lines().collect();
            lines
                .chunks(2)
                .map(|f| (f[0].to_owned(), f.get(1).copied().unwrap_or("").to_owned()))
                .collect()
        })
        .collect();
    assert_eq!(
        stacks.len(),
        addrs.len(),
        "llvm-symbolizer answered every address once"
    );
    stacks
}

fn mca(cpu: &str, asm: &str) -> Option<f64> {
    let mut child = Command::new("llvm-mca")
        .args([&format!("-mcpu={cpu}"), "-iterations=100", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(asm.as_bytes()).ok()?;
    let out = child.wait_with_output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let field = |k: &str| -> Option<f64> {
        text.lines()
            .find_map(|l| l.strip_prefix(k))
            .and_then(|v| v.trim().parse().ok())
    };
    Some(field("Total Cycles:")? / field("Iterations:")?)
}

/// Instruction counts by inline path, hakmem frames only, outermost first.
#[derive(Default)]
struct Tree {
    count: usize,
    children: Vec<(String, Tree)>,
}

impl Tree {
    fn add(&mut self, path: &[&str]) {
        self.count += 1;
        if let Some((first, rest)) = path.split_first() {
            let i = match self.children.iter().position(|(n, _)| n == first) {
                Some(i) => i,
                None => {
                    self.children.push(((*first).to_owned(), Tree::default()));
                    self.children.len() - 1
                }
            };
            self.children[i].1.add(rest);
        }
    }

    /// Children, biggest first; a chain that adds no instructions of its
    /// own prints as one `a → b → c`.
    fn print(&self, out: &mut String, depth: usize, max: usize) {
        let mut kids: Vec<&(String, Tree)> = self.children.iter().collect();
        kids.sort_by(|a, b| b.1.count.cmp(&a.1.count).then(a.0.cmp(&b.0)));
        for (name, t) in kids {
            let mut label = name.clone();
            let mut t = t;
            while t.children.len() == 1 && t.children[0].1.count == t.count {
                label.push_str(" → ");
                label.push_str(&t.children[0].0);
                t = &t.children[0].1;
            }
            let _ = writeln!(out, "{:>5}  {}{label}", t.count, "  ".repeat(depth));
            if depth + 1 < max {
                t.print(out, depth + 1, max);
            }
        }
    }
}

fn ours(file: &str, src: &str) -> bool {
    file.starts_with(src)
}

/// hakmem's test oracles: `laws` holds the reference loops the benches
/// race against, not what a user runs.
fn oracle(file: &str, src: &str) -> bool {
    file.strip_prefix(src)
        .is_some_and(|f| f.starts_with("laws.rs"))
}

/// The line of `path:line[:column]`; 0 is code no line owns.
fn line_of(file: &str) -> u32 {
    let mut parts = file.rsplit(':');
    let last = parts.next().and_then(|p| p.parse().ok());
    let before = parts.next().and_then(|p| p.parse().ok());
    before.or(last).unwrap_or(0)
}

/// A frame as `module::name`: the file stem stands in for the module
/// path DWARF leaves out of an inlined frame, so `hilbert::encode` and
/// `dilated::encode` stay apart. `hakmem::` inside generics is noise.
fn frame(name: &str, file: &str) -> String {
    let stem = short(file).split('.').next().unwrap_or("");
    format!("{stem}::{}", tidy(name))
}

/// A name to read: without `hakmem::`, and without generic arguments that
/// say more about the compiler than the code (a closure's environment, an
/// iterator adapter's type). `<u64>` and `<u64, 2>` stay.
fn tidy(name: &str) -> String {
    let name = name.replace("hakmem::", "");
    let b = name.as_bytes();
    let mut out = String::with_capacity(name.len());
    let mut i = 0;
    while i < b.len() {
        let after_ident =
            i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_' || b[i - 1] == b':');
        if b[i] != b'<' || !after_ident {
            out.push(char::from(b[i]));
            i += 1;
            continue;
        }
        let mut depth = 0i32;
        let mut j = i;
        while j < b.len() {
            depth += match b[j] {
                b'<' => 1,
                b'>' => -1,
                _ => 0,
            };
            if depth == 0 {
                break;
            }
            j += 1;
        }
        let args = &name[i..=j.min(b.len() - 1)];
        if args.contains('{') || args.len() > 22 {
            if out.ends_with("::") {
                out.truncate(out.len() - 2);
            }
        } else {
            out.push_str(args);
        }
        i = j + 1;
    }
    out
}

fn short(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

// ---------------------------------------------------------------------------

struct Args {
    bench: String,
    bin: String,
    dis: String,
    got: Option<String>,
    root: String,
    cpus: Vec<String>,
    detail: Option<String>,
    check: bool,
    speed: bool,
}

fn args() -> Args {
    let mut a = Args {
        bench: String::new(),
        bin: String::new(),
        dis: String::new(),
        got: None,
        root: String::new(),
        cpus: Vec::new(),
        detail: None,
        check: false,
        speed: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut val = || it.next().unwrap_or_else(|| panic!("{flag} wants a value"));
        match flag.as_str() {
            "--bench" => a.bench = val(),
            "--bin" => a.bin = val(),
            "--dis" => a.dis = val(),
            "--got" => a.got = Some(val()),
            "--root" => a.root = val(),
            "--cpus" => a.cpus = val().split(',').map(str::to_owned).collect(),
            "--detail" => a.detail = Some(val()),
            "--check" => a.check = true,
            "--speed" => a.speed = true,
            _ => panic!("unknown argument {flag}"),
        }
    }
    a.root = a.root.trim_end_matches('/').to_owned();
    a
}

fn main() {
    let a = args();
    let bytes = std::fs::read(&a.dis).expect("--dis: objdump -d output");
    let level = Level::detect();
    let t0 = std::time::Instant::now();
    let text = Text::new(&bytes, level);
    let dt = t0.elapsed();
    eprintln!(
        "loops: {} MB of newlines in {:.1} ms, {:?}",
        bytes.len() >> 20,
        dt.as_secs_f64() * 1e3,
        level
    );
    if a.check {
        // Every level the machine has, every class of every block, against
        // a byte at a time: a bug in hakmem or in scan.rs cannot pass as a
        // clean snapshot.
        let want = scan::classes_naive(&bytes);
        let nl: Vec<u64> = want.iter().map(|m| m[scan::NL]).collect();
        for l in Level::all() {
            assert!(
                l.newlines(&bytes) == nl,
                "{l:?}: newlines differ from the byte scan"
            );
            assert!(
                l.classes_all(&bytes) == want,
                "{l:?}: classes differ from the byte scan"
            );
        }
    }
    if a.speed {
        // Best of twenty into a warm buffer, per level: no page faults.
        let best = |f: &mut dyn FnMut()| {
            (0..20)
                .map(|_| {
                    let t = std::time::Instant::now();
                    f();
                    t.elapsed().as_secs_f64()
                })
                .fold(f64::INFINITY, f64::min)
        };
        let gbs = |s: f64| bytes.len() as f64 / s / 1e9;
        let (mut nl, mut all) = (Vec::new(), Vec::new());
        for l in Level::all() {
            let n = best(&mut || {
                nl.clear();
                l.newlines_into(&bytes, &mut nl);
            });
            let c = best(&mut || {
                all.clear();
                l.classes_all_into(&bytes, &mut all);
            });
            eprintln!(
                "loops: {l:?}: newlines {:.1} GB/s, every class {:.1} GB/s",
                gbs(n),
                gbs(c)
            );
        }
        return;
    }
    let got: HashMap<u64, String> = a.got.as_ref().map_or_else(HashMap::new, |p| {
        std::fs::read_to_string(p)
            .expect("--got")
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .map(|(s, n)| (hex(s.as_bytes()), n.to_owned()))
            .collect()
    });
    let mine = format!("{}::", a.bench);
    let funcs = functions(
        &text,
        |n| n.contains("criterion::bencher::Bencher>::iter") && n.contains(&mine),
        &got,
    );
    eprintln!(
        "loops: {} of {} blocks classified, for {} functions",
        text.classified.get(),
        text.nl.len(),
        funcs.len()
    );
    let loops: Vec<(&Func, Vec<usize>)> = funcs
        .iter()
        .flat_map(|f| innermost_loops(f).into_iter().map(move |l| (f, l)))
        .collect();
    let addrs: Vec<u64> = loops
        .iter()
        .flat_map(|(f, l)| l.iter().map(|&i| f.insns[i].addr))
        .collect();
    let stacks = if addrs.is_empty() {
        Vec::new()
    } else {
        symbolize(&a.bin, &addrs)
    };
    let src = format!("{}/src/", a.root);
    let benches = format!("{}/benches/", a.root);

    let mut summary = Vec::new();
    let mut detail = String::new();
    let mut next = 0;
    for (f, l) in &loops {
        let stacks = &stacks[next..next + l.len()];
        next += l.len();
        let mut tree = Tree::default();
        for s in stacks {
            // An instruction under a `laws` frame is the oracle's, whatever
            // hakmem primitives it is built from.
            if s.iter().any(|(_, file)| oracle(file, &src)) {
                tree.add(&[]);
                continue;
            }
            let path: Vec<String> = s
                .iter()
                .rev()
                .filter(|(_, file)| ours(file, &src))
                .map(|(n, file)| frame(n, file))
                .collect();
            tree.add(&path.iter().map(String::as_str).collect::<Vec<_>>());
        }
        let hakmem: usize = tree.children.iter().map(|(_, t)| t.count).sum();
        // hakmem's loop: a quarter of it inlined from hakmem, or a call into
        // hakmem. DWARF drops an inline record now and then, so an
        // incumbent's loop can show a stray hakmem line or two.
        let calls_hakmem = l.iter().any(|&i| {
            f.insns[i]
                .callee
                .as_deref()
                .is_some_and(|c| c.contains("hakmem::"))
        });
        if hakmem * 4 < l.len() && !calls_hakmem {
            continue;
        }
        // Where the bench calls it: the innermost frame in benches/.
        let site = stacks
            .iter()
            .flatten()
            .find(|(_, file)| file.starts_with(&benches) && line_of(file) != 0)
            .map_or_else(
                || "?".to_owned(),
                |(_, file)| {
                    let file = short(file);
                    file.rsplit_once(':')
                        .map_or(file, |(fl, _col)| fl)
                        .to_owned()
                },
            );
        // A call through a pointer the bench holds (compact's table of
        // implementations) names nothing: that kernel is in the outlined
        // list, not in any loop.
        let mut calls: Vec<String> = l
            .iter()
            .filter_map(|&i| f.insns[i].callee.as_deref())
            .map(tidy)
            .collect();
        calls.sort_unstable();
        calls.dedup();
        let asm: String = l
            .iter()
            .map(|&i| &f.insns[i])
            .filter(|x| {
                !x.mnem.starts_with('j') && !(x.mnem.starts_with("call") && x.ops.starts_with("0x"))
            })
            .map(|x| format!("{} {}\n", x.mnem, x.ops))
            .collect();
        let cycles: Vec<String> = a
            .cpus
            .iter()
            .map(|c| mca(c, &asm).map_or_else(|| format!("{c} n/a"), |v| format!("{c} {v:.1}")))
            .collect();
        let mut s = format!("{site}  {} insns  {}\n", l.len(), cycles.join("  "));
        if !calls.is_empty() {
            let _ = writeln!(s, "       calls {}", calls.join(", "));
        }
        let mut t = String::new();
        tree.print(&mut t, 0, 3);
        s.push_str(&t);
        let other = l.len() - hakmem;
        if other > 0 {
            let _ = writeln!(s, "{other:>5}  (not hakmem)");
        }
        summary.push(s.clone());

        let _ = writeln!(detail, "== {} {site}\n   in {}", a.bench, f.name);
        detail.push_str(&s);
        let mut full = String::new();
        tree.print(&mut full, 0, usize::MAX);
        let _ = writeln!(detail, "-- whole tree\n{full}-- instructions");
        for (&i, st) in l.iter().zip(stacks) {
            let x = &f.insns[i];
            let from = st
                .iter()
                .find(|(_, file)| file.starts_with(&src))
                .map_or_else(String::new, |(n, file)| format!("{n} {}", short(file)));
            let _ = writeln!(detail, "  {:x}  {:<8} {:<48} {from}", x.addr, x.mnem, x.ops);
        }
        detail.push('\n');
    }
    let mut out = std::io::stdout().lock();
    for s in summary {
        let _ = write!(out, "{} {s}", a.bench);
    }
    if let Some(p) = a.detail {
        let mut prev = std::fs::read_to_string(&p).unwrap_or_default();
        prev.push_str(&detail);
        std::fs::write(p, prev).expect("--detail");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tidy_names() {
        assert_eq!(tidy("hakmem::word::pext<u64>"), "word::pext<u64>");
        assert_eq!(tidy("to_int<u64, 2>"), "to_int<u64, 2>");
        assert_eq!(
            tidy("run<usize, rank9::{impl#1}::rank::{closure_env#0}>"),
            "run"
        );
        assert_eq!(
            tidy(
                "<hakmem::isa::x86v2::X86V2 as hakmem::isa::Isa>::run::trampoline::<usize, <hakmem::rank9::Rank9>::rank::{closure#4}>"
            ),
            "<isa::x86v2::X86V2 as isa::Isa>::run::trampoline"
        );
    }

    #[test]
    fn hex_digits() {
        for s in [
            "0",
            "f",
            "1886a0",
            "00000000001886a0",
            "deadbeefcafe1234",
            "abcdef0123456789",
            "7",
        ] {
            assert_eq!(
                hex(s.as_bytes()),
                u64::from_str_radix(s, 16).unwrap(),
                "{s}"
            );
        }
    }

    #[test]
    fn finds_the_inner_loop() {
        // An outer loop at 10 around an inner one at 20.
        let dis = "\
0000000000000010 <iter::<b::c>>:
      10:      \tmovq\t%rdi, %rax
      14:      \txorl\t%ecx, %ecx
      20:      \taddq\t$0x1, %rcx
      24:      \tcmpq\t%rax, %rcx
      28:      \tjne\t0x20 <iter::<b::c>+0x10>
      2a:      \tcallq\t*0x100(%rip)         # 0x3000 <writev+0x3000>
      30:      \tdecq\t%rax
      34:      \tjne\t0x14 <iter::<b::c>+0x4>
      36:      \tretq

";
        let got = HashMap::from([(0x3000, "black_box".to_owned())]);
        for level in Level::all() {
            let text = Text::new(dis.as_bytes(), level);
            let f = functions(&text, |_| true, &got);
            assert_eq!(f.len(), 1, "{level:?}");
            assert_eq!(f[0].name, "iter::<b::c>");
            assert_eq!(f[0].insns[4].target, Some(0x20));
            assert_eq!(f[0].insns[5].callee.as_deref(), Some("black_box"));
            let loops = innermost_loops(&f[0]);
            assert_eq!(loops, vec![vec![2, 3, 4]]);
        }
    }
}
