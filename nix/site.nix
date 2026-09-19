# gh-pages front page: the check matrix of `main` read live from the
# status JSON that CI writes (status/<system>/<check>.json, see
# .github/status.sh), links to rustdoc and the bench trend. Catppuccin
# Latte by default, Mocha for a dark colour scheme. Built in the pages
# workflow from the current matrix; not kept in the repository.
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
      hakmem-test-bmi2-default = "tests, +bmi2,+pclmulqdq";
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
        :root {
          --base: #eff1f5; --mantle: #e6e9ef; --crust: #dce0e8;
          --surface0: #ccd0da; --surface1: #bcc0cc;
          --text: #4c4f69; --subtext0: #6c6f85; --overlay0: #9ca0b0;
          --blue: #1e66f5; --green: #40a02b; --red: #d20f39; --yellow: #df8e1d;
          --mauve: #8839ef; --lavender: #7287fd;
        }
        @media (prefers-color-scheme: dark) {
          :root {
            --base: #1e1e2e; --mantle: #181825; --crust: #11111b;
            --surface0: #313244; --surface1: #45475a;
            --text: #cdd6f4; --subtext0: #a6adc8; --overlay0: #6c7086;
            --blue: #89b4fa; --green: #a6e3a1; --red: #f38ba8; --yellow: #f9e2af;
            --mauve: #cba6f7; --lavender: #b4befe;
          }
        }
        html { background: var(--base); color: var(--text); }
        body { font: 16px/1.5 system-ui, sans-serif; max-width: 52rem; margin: 3rem auto; padding: 0 1rem; }
        h1 { font-size: 2rem; margin: 0; }
        h1 + p { color: var(--subtext0); margin-top: 0.25rem; }
        h2 { font-size: 1.25rem; margin-top: 2.5rem; }
        a { color: var(--blue); text-decoration: none; }
        a:hover { text-decoration: underline; }
        nav a + a::before { content: "·"; color: var(--overlay0); margin: 0 0.6rem; }
        code { font-family: ui-monospace, monospace; font-size: 0.85em; color: var(--mauve); }
        table { border-collapse: collapse; width: 100%; background: var(--mantle); border-radius: 0.5rem; overflow: hidden; }
        th, td { padding: 0.5rem 0.9rem; text-align: left; border-top: 1px solid var(--surface0); }
        thead th { background: var(--surface0); border-top: 0; font-weight: 600; }
        th[scope=row] { font-weight: 400; }
        th[scope=row] code { color: var(--subtext0); }
        td[data-check] { white-space: nowrap; font-family: ui-monospace, monospace; font-size: 0.85em; }
        .dot { display: inline-block; width: 0.7em; height: 0.7em; border-radius: 50%; background: var(--surface1); margin-right: 0.5em; vertical-align: -0.05em; }
        .pass .dot { background: var(--green); }
        .fail .dot { background: var(--red); }
        .other .dot { background: var(--yellow); }
        .na { color: var(--overlay0); }
        .na .dot { background: transparent; border: 1px solid var(--surface1); }
        footer { margin-top: 3rem; color: var(--subtext0); font-size: 0.9rem; border-top: 1px solid var(--surface0); padding-top: 1rem; }
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
