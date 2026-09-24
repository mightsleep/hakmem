# gh-pages front page: the check matrix of `main` read live from the
# status JSON that CI writes (status/<system>/<check>.json, see
# .github/status.sh), links to rustdoc and the bench trend. Palette and
# base styles come from _palette.nix. Built in the pages workflow from the
# current matrix; not kept in the repository.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    repo = "mightsleep/hakmem";

    # Rows: the checks of this system (the workflow builds the index on
    # x86_64-linux, the superset) plus miri, which is a job, not a check.
    # Cells of the other system are looked up at run time; a missing file
    # means the cell is not in that system's matrix.
    labels = {
      hakmem-test-portable-default = "tests, portable";
      hakmem-test-bmi2-default = "tests, +bmi2,+pclmulqdq,+avx2";
      hakmem-test-bmi2-portable-feature = "tests, +bmi2 with feature portable";
      hakmem-doctest-portable = "doctests, portable";
      hakmem-doctest-bmi2 = "doctests, +bmi2";
      hakmem-clippy-portable-default = "clippy, portable";
      hakmem-clippy-bmi2-default = "clippy, +bmi2";
      hakmem-clippy-bmi2-portable-feature = "clippy, +bmi2 with feature portable";
      hakmem-doc = "rustdoc, warnings as errors";
      hakmem-msrv = "MSRV build of the packaged tarball";
      hakmem-deny = "cargo-deny";
      hakmem-audit = "cargo-audit";
      treefmt = "treefmt";
      miri = "Miri over the intrinsics, both paths";
    };
    order = builtins.attrNames labels;
    present = builtins.attrNames config.checks ++ ["miri"];
    rows = lib.filter (c: lib.elem c present) order ++ lib.filter (c: !(labels ? ${c})) present;
    systems = ["x86_64-linux" "aarch64-linux"];

    cell = system: check: ''<td data-system="${system}" data-check="${check}"><span class="dot"></span>pending</td>'';
    row = check: ''
      <tr><th scope="row">${labels.${check} or check}<br><code>${check}</code></th>${lib.concatMapStrings (s: cell s check) systems}</tr>
    '';
  in {
    packages.site-index = pkgs.writeText "index.html" ''
      <!doctype html>
      <html lang="en">
      <meta charset="utf-8">
      <meta name="viewport" content="width=device-width, initial-scale=1">
      <title>hakmem</title>
      <style>
      ${import ./_palette.nix}
        td[data-check] { white-space: nowrap; font-family: ui-monospace, monospace; font-size: 0.85em; }
        .dot { display: inline-block; width: 0.7em; height: 0.7em; border-radius: 50%; background: var(--surface1); margin-right: 0.5em; vertical-align: -0.05em; }
        .pass .dot { background: var(--green); }
        .fail .dot { background: var(--red); }
        .other .dot { background: var(--yellow); }
        .na { color: var(--overlay0); }
        .na .dot { background: transparent; border: 1px solid var(--surface1); }
      </style>
      <h1>hakmem</h1>
      <p>Bit tricks as a lawful algebra. A learning project; the README's Status section says what to expect.</p>
      <nav>
        <a href="doc/hakmem/">docs</a>
        <a href="https://github.com/${repo}/blob/main/docs/design.md">design</a>
        <a href="doc/hakmem/cookbook/index.html">cookbook</a>
        <a href="dev/bench/">benchmarks</a>
        <a href="https://github.com/${repo}">source</a>
      </nav>
      <h2>Checks on <code>main</code></h2>
      <p>Every cell is one Nix derivation built in a sandbox without network;
         the matrix lives in <code>nix/matrix.nix</code> and the workflow reads
         it at run time. Only cells whose output is not yet in the binary
         cache are built; a cell in the cache passed by construction. CI
         writes each result here after every push
         (<code>.github/status.sh</code>). Benchmarks are tracked over time,
         never pass/fail.</p>
      <table>
        <thead><tr><th>check</th>${lib.concatMapStrings (s: "<th>${s}</th>") systems}</tr></thead>
        <tbody>
      ${lib.concatMapStrings row rows}
        </tbody>
      </table>
      <footer>Bit tricks as a lawful algebra, MIT OR Apache-2.0. Named after HAKMEM, MIT AI Memo 239 (1972).</footer>
      <script>
        for (const td of document.querySelectorAll("td[data-check]")) {
          const url = "status/" + td.dataset.system + "/" + td.dataset.check + ".json";
          fetch(url, {cache: "no-cache"}).then(function (r) {
            if (!r.ok) { td.className = "na"; td.lastChild.textContent = "n/a"; return; }
            return r.json().then(function (s) {
              td.className = s.message === "pass" ? "pass" : s.message === "fail" ? "fail" : "other";
              td.lastChild.textContent = s.message;
            });
          }).catch(function () { td.className = "other"; td.lastChild.textContent = "unknown"; });
        }
      </script>
    '';
  };
}
