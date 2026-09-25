# What a crate that calls hakmem ends up with: every function of a linked
# binary, cut out of `llvm-objdump -d -C --no-leading-addr` output.
#
#   llvm-objdump ... bin | awk -v bench=hilbert -v dir=out -f outlined.awk
#
# Appends to three files in `dir`:
#   outlined  bench, then a hakmem function that survived as a function
#             of its own: the snapshot. A kernel that should melt into
#             its caller and shows up here has lost its inlining.
#   sizes     bench, instruction count, name, for every hakmem function:
#             to read, not to diff; every edit moves it.
#   annotated the disassembly of those and of the bench's own functions,
#             where the inlined kernels actually are, with the source
#             lines objdump found.
#
# Names lose the `.llvm.<n>` of a promoted local and the legacy `::h<hash>`;
# both change with any edit anywhere. With `-v got=file` (got.awk), a call
# through the GOT is annotated with the function it reaches.

BEGIN {
  if (got != "")
    while ((getline line < got) > 0) {
      split(line, f, "\t")
      slot[f[1]] = f[2]
    }
}

/^<.*>:$/ {
  flush()
  name = substr($0, 2, length($0) - 3)
  sub(/ \(\.llvm\.[0-9]+\)$/, "", name)
  sub(/::h[0-9a-f]{16}$/, "", name)
  ours = index(name, "hakmem") > 0
  keep = ours || index(name, bench "::") > 0
  n = 0
  body = ""
  next
}
name == "" { next }
# Source lines from `-l` are `; file:line`. `int3` is the padding up to
# the next function, not code.
/^[ \t]+[a-z]/ && $1 != "int3" { n++ }
keep && match($0, /# 0x[0-9a-f]+ <[^>]*>$/) {
  s = substr($0, RSTART + 4)
  sub(/ .*/, "", s)
  if (s in slot) $0 = substr($0, 1, RSTART - 1) "# <" slot[s] ">"
}
keep && NF { body = body $0 "\n" }

END { flush() }

function flush() {
  if (name == "") return
  if (ours) {
    print bench "\t" name >> (dir "/outlined")
    print bench "\t" n "\t" name >> (dir "/sizes")
  }
  if (keep) printf "\n%s <%s>:\n%s", bench, name, body >> (dir "/annotated.s")
  name = ""
}
