# Per function, how many of each instruction: the snapshot the codegen
# check diffs. Input is functions.awk's output.
#
#   awk -f mnemonics.awk functions.s
#
# One line per function, mnemonics sorted, then the total. Operands and
# registers are left out; those move with every register allocation.

/^[^ \t.#][^ \t]*:$/ || /^[^\t.#].*[^:]:$/ {
  flush()
  name = substr($0, 1, length($0) - 1)
  next
}
/^\t[a-z]/ && name != "" {
  # `rep bsf` and `lock xadd` are one instruction each.
  m = ($1 == "rep" || $1 == "repne" || $1 == "lock") ? $1 " " $2 : $1
  count[m]++
  total++
}
END { flush() }

function flush(   m, line, keys, k, i, j, t) {
  if (name == "") return
  k = 0
  for (m in count) keys[++k] = m
  for (i = 2; i <= k; i++)
    for (j = i; j > 1 && keys[j - 1] > keys[j]; j--) {
      t = keys[j]; keys[j] = keys[j - 1]; keys[j - 1] = t
    }
  line = name ":"
  for (i = 1; i <= k; i++) line = line " " keys[i] "=" count[keys[i]]
  print line " total=" total
  delete count
  total = 0
  name = ""
}
