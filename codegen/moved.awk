# The loops of one codegen snapshot that moved: `awk -v snap=NAME -f
# moved.awk OLD NEW`, a Markdown table row per loop that appeared, went,
# or moved by more than a tenth in instructions or znver5 cycles. The rest
# of a new compiler's change is instructions changing hands between hakmem
# functions, which a lock bump brings every week and nobody needs to read.
#
# A loop's line in loops.rs's output:
#   <bench> <file>:<line>  <n> insns  znver5 <cycles>  x86-64-v3 <cycles>
# The same bench and line can hold several loops; they pair up in order.
# Both files whole, not a diff: a diff leaves out lines it matched, and
# two loops that trade places would pair with each other's numbers.

/^[^ ]+ [^ ]+:[0-9]+  [0-9]+ insns  znver5 / {
  side = FILENAME == ARGV[1] ? "old" : "new"
  key = $1 " " $2
  k = key SUBSEP (++seen[side, key])
  if (side == "old") old[k] = $3 " " $6
  else new[k] = $3 " " $6
  loops[k] = key
}

function moved(a, b) {
  return a > 0 ? (b - a) / a > 0.1 || (a - b) / a > 0.1 : b > 0
}

END {
  for (k in loops) {
    row = "| " snap " | " loops[k] " | "
    if (!(k in old)) {
      split(new[k], n, " ")
      print row "new | " n[1] " | " n[2] " |"
    } else if (!(k in new)) {
      split(old[k], o, " ")
      print row "gone | " o[1] " | " o[2] " |"
    } else {
      split(old[k], o, " ")
      split(new[k], n, " ")
      if (moved(o[1] + 0, n[1] + 0) || moved(o[2] + 0, n[2] + 0))
        print row "moved | " o[1] " → " n[1] " | " o[2] " → " n[2] " |"
    }
  }
}
