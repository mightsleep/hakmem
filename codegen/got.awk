# Which function each GOT slot of a PIE holds, so outlined.awk can name
# the target of `call *slot(%rip)`. rustc calls into other crates through
# the GOT, and objdump names such a call after the nearest dynamic symbol
# (`writev+0x3f2e30`), which is not the function called.
#
#   awk -f got.awk <(llvm-nm -C --defined-only bin) <(llvm-objdump -R bin) > got
#
# Out: slot address, tab, name; addresses in hex without leading zeros.
# A slot filled at load time (R_X86_64_RELATIVE) holds an address in the
# binary, named through nm; one bound to a shared library
# (R_X86_64_GLOB_DAT, JUMP_SLOT) holds the symbol it names.

function hex(s) {
  sub(/^(\*ABS\*\+)?0x/, "", s)
  sub(/^0+/, "", s)
  return s == "" ? "0" : tolower(s)
}

FNR == NR {
  if ($2 ~ /^[Tt]$/) {
    a = hex($1)
    name = $0
    sub(/^[0-9a-f]+ [Tt] /, "", name)
    sub(/::h[0-9a-f]{16}$/, "", name)
    # Aliases share an address; the first is as good as any.
    if (!(a in text)) text[a] = name
  }
  next
}
$2 == "R_X86_64_RELATIVE" { a = hex($3); if (a in text) print hex($1) "\t" text[a]; next }
$2 == "R_X86_64_GLOB_DAT" || $2 == "R_X86_64_JUMP_SLOT" { print hex($1) "\t" $3 }
