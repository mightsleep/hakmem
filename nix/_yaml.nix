# A YAML emitter for the subset GitHub reads: maps, lists, strings, bools,
# numbers, null. Ours rather than `pkgs.formats.yaml` for what a reader of
# the generated files sees: keys in the order a workflow is written in,
# scripts as literal blocks, comments. The leading `_` keeps import-tree
# off this file; it needs `pipe-operators`.
#
# The representation does the work. A document is a list of lines relative
# to column 0, so nesting is `indent`, a map over lines, and nothing ever
# counts columns. A list item is `hang "- "` over the item's own document:
# the dash on the first line, two spaces before the rest. That is the
# whole of block YAML.
#
# `checks.workflows` holds it to a real parser: every rendered file and
# `corpus` below must parse back to `toData` of what was rendered.
{lib}: let
  inherit (builtins) isAttrs isList isString head tail match toJSON elem;
  inherit (lib.strings) concatLines splitString hasInfix hasPrefix hasSuffix removeSuffix toLower;
  inherit (lib.lists) concatMap findFirstIndex sort filter;
  inherit (lib.attrsets) attrNames mapAttrs;

  # Values that are not data: `comment text v` puts `#` lines above v,
  # `raw text` goes out verbatim (`repo@sha # v7.0.1`, whose comment zizmor
  # holds to the SHA, cannot be a string), `ordered keys m` writes those
  # keys of m first.
  node = tag: fields:
    fields
    // {
      _type = "yaml";
      _tag = tag;
    };
  is = tag: v: isAttrs v && v._type or null == "yaml" && v._tag == tag;
  nodes = {
    comment = text: value: node "comment" {inherit text value;};
    raw = text: node "raw" {inherit text;};
    ordered = keys: value: node "ordered" {inherit keys value;};
  };

  # Keys in the order a workflow is written in; the rest alphabetically.
  rank = let
    order = [
      "name"
      "description"
      "id"
      "on"
      "permissions"
      "defaults"
      "concurrency"
      "env"
      "inputs"
      "outputs"
      "runs"
      "jobs"
      "needs"
      "if"
      "runs-on"
      "environment"
      "strategy"
      "steps"
      "shell"
      "uses"
      "with"
      "working-directory"
      "run"
      "version"
      "updates"
      "package-ecosystem"
      "directory"
      "schedule"
    ];
  in
    k: findFirstIndex (x: x == k) (builtins.length order) order;
  # A key `ordered` names but the map lacks is skipped: one order serves
  # the maps built with and without it.
  keysOf = first': m: let
    first = filter (k: m ? ${k}) first';
  in
    first
    ++ (
      attrNames m
      |> filter (k: !elem k first)
      |> sort (a: b:
        if rank a == rank b
        then a < b
        else rank a < rank b)
    );

  # Plain when nothing in it can mean something else to a parser: no
  # indicator first, no `: ` or ` #` inside, not a word YAML 1.1 reads as
  # a bool, null or float. Nor anything spelled in the alphabet of YAML 1.1
  # numbers and dates: hex, octal, binary, sexagesimal (`1:20` is 80) and
  # `2026-09-26` all fall in it, and so does `1.2.3`, quoted for nothing;
  # telling them apart would be a YAML parser. The rest as JSON strings,
  # which are YAML double-quoted scalars.
  plain = s:
    !elem (toLower s) ["" "~" "null" "true" "false" "yes" "no" "on" "off" "y" "n" ".inf" "-.inf" "+.inf" ".nan"]
    && match "[A-Za-z0-9$_./(][^\n]*" s != null
    && match ".*(: | #|:$| $).*" s == null
    && match "[-+.0-9][-+.0-9a-fA-F_:xXoObB]*" s == null;
  scalar = v:
    if isString v
    then
      if plain v
      then v
      else toJSON v
    else if v == null
    then "null"
    else toJSON v;

  # The kernel: documents are lists of lines.
  indent = map (l:
    if l == ""
    then ""
    else "  " + l);
  hang = prefix: lines: let
    pad = lib.fixedWidthString (lib.stringLength prefix) " " "";
  in
    [(prefix + head lines)]
    ++ (
      tail lines
      |> map (l:
        if l == ""
        then ""
        else pad + l)
    );
  # A scalar that starts after a key or a dash: its first line joins the
  # prefix, the rest keeps its own indentation.
  after = prefix: lines: [(prefix + head lines)] ++ tail lines;
  hashes = text:
    splitString "\n" text
    |> map (l:
      if l == ""
      then "#"
      else "# " + l);

  # Scalars that fit after a key or a dash, and the one that does not.
  multiline = s: isString s && hasInfix "\n" s && !hasPrefix " " s;
  literal = s:
    [
      (
        if hasSuffix "\n" s
        then "|"
        else "|-"
      )
    ]
    ++ indent (splitString "\n" (removeSuffix "\n" s));
  empty = v: v == [] || v == {};
  inline = v:
    if is "raw" v
    then v.text
    else if v == []
    then "[]"
    else if v == {}
    then "{}"
    else scalar v;
  block = v: !(is "raw" v) && !empty v && (isAttrs v || isList v);

  # A map entry and a list item, by the shape of the value.
  entry = k: v:
    if is "comment" v
    then hashes v.text ++ entry k v.value
    else if multiline v
    then after "${scalar k}: " (literal v)
    else if isList v && v != []
    then ["${scalar k}:"] ++ doc v
    else if block v
    then ["${scalar k}:"] ++ indent (doc v)
    else ["${scalar k}: ${inline v}"];
  item = v:
    if is "comment" v
    then hashes v.text ++ item v.value
    else if multiline v
    then after "- " (literal v)
    else if block v
    then hang "- " (doc v)
    else ["- ${inline v}"];

  doc = v:
    if is "comment" v
    then hashes v.text ++ doc v.value
    else if is "ordered" v
    then keysOf v.keys v.value |> concatMap (k: entry k v.value.${k})
    else if isList v
    then concatMap item v
    else keysOf [] v |> concatMap (k: entry k v.${k});

  # What a parser reads back: comments gone, a raw value up to its comment.
  toData = v:
    if is "comment" v || is "ordered" v
    then toData v.value
    else if is "raw" v
    then head (splitString " #" v.text)
    else if isAttrs v
    then mapAttrs (_: toData) v
    else if isList v
    then map toData v
    else v;

  # Strings at the edges of `plain`, for the parser half of the check.
  corpus = {
    words = ["on" "On" "off" "yes" "no" "y" "n" "true" "null" "~" ""];
    numbers = ["1" "1.0" "-1" "0x1f" "0x1F" "0o17" "0b101" "017" "1_000" "1e3" "+1" ".5" "1.2.3" "1:20" "2026-09-26" ".inf" "-.Inf" ".NaN" "200%" "1x" "7zip"];
    indicators = ["-x" "- x" "?x" ":x" "#x" "&x" "*x" "!x" "|x" ">x" "'x" "\"x" "%x" "@x" "`x" "{x" "[x" ",x"];
    inside = ["a: b" "a:b" "a #b" "a#b" "a:" "a " " a" "it's" "say \"hi\"" "x{y}[z]" "\${{ matrix.check }}" "v*" "release/**"];
    blocks = ["a\nb\n" "a\nb" "a\n\n  b\n" " lead\nx"];
  };

  tests = lib.runTests {
    testIndentSkipsBlank = {
      expr = indent ["a" "" "b"];
      expected = ["  a" "" "  b"];
    };
    testHang = {
      expr = hang "- " ["a: 1" "b: 2"];
      expected = ["- a: 1" "  b: 2"];
    };
    testListOfMaps = {
      expr = doc {
        steps = [
          {
            uses = "x";
            "with".a = 1;
          }
          {run = "a\nb\n";}
        ];
      };
      expected = ["steps:" "- uses: x" "  with:" "    a: 1" "- run: |" "    a" "    b"];
    };
    testOrderAndComment = {
      expr = doc (nodes.ordered ["b"] {
        a = 1;
        b = nodes.comment "first" 2;
        name = "x";
      });
      expected = ["# first" "b: 2" "name: x" "a: 1"];
    };
    testDocumentComment = {
      expr = doc (nodes.comment "top" (nodes.ordered ["z" "missing"] {
        a = 1;
        z = 2;
      }));
      expected = ["# top" "z: 2" "a: 1"];
    };
    testRaw = {
      expr = doc {uses = nodes.raw "a@b # v1";};
      expected = ["uses: a@b # v1"];
    };
    testQuoting = {
      expr = map scalar ["on" "1.0" "a: b" "x #y" "-x" "a#b" "it's" "\${{ x }}"];
      expected = [''"on"'' ''"1.0"'' ''"a: b"'' ''"x #y"'' ''"-x"'' "a#b" "it's" "\${{ x }}"];
    };
    testToData = {
      expr = toData (nodes.ordered ["b"] {
        a = nodes.raw "a@b # v1";
        b = nodes.comment "c" [1];
      });
      expected = {
        a = "a@b";
        b = [1];
      };
    };
  };
in
  nodes
  // {
    inherit toData corpus tests;
    # A document, with `header` as `#` lines at the top.
    toYAML = header: v: hashes header ++ doc v |> concatLines;
  }
