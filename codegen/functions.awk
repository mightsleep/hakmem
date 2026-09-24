# The functions named by `CHECK-LABEL: <name>:` lines, cut out of
# demangled asm and printed in the order the labels come. A function runs
# from its label to `.Lfunc_end`; directives and local labels stay, since
# FileCheck skips what it is not asked about.
#
#   awk -f functions.awk src/lib.rs demangled.s > functions.s
#
# A label that matches no function is an error: a renamed kernel must
# fail the check, not pass it vacuously.

FNR == NR {
  if (match($0, /CHECK-LABEL: (.*):$/)) {
    name = substr($0, RSTART + 13, RLENGTH - 14)
    order[++n] = name
    wanted[name] = 1
  }
  next
}

/^[^ \t.#][^ \t]*:$/ || /^[^\t.#].*[^:]:$/ {
  label = substr($0, 1, length($0) - 1)
  current = (label in wanted) ? label : ""
}
current != "" { body[current] = body[current] $0 "\n" }
/^\.Lfunc_end/ { current = "" }

END {
  for (i = 1; i <= n; i++) {
    if (!(order[i] in body)) {
      print "functions.awk: no function " order[i] > "/dev/stderr"
      failed = 1
      continue
    }
    printf "%s", body[order[i]]
  }
  exit failed
}
