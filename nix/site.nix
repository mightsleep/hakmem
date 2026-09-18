# gh-pages front page: badges, links to rustdoc and the bench trend, the
# table of checks. Built in the pages workflow from the current matrix;
# not kept in the repository.
{lib, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    ...
  }: let
    repo = "mightsleep/hakmem";
    badge = name: ''<a href="https://github.com/${repo}/actions/workflows/${name}.yml"><img src="https://github.com/${repo}/actions/workflows/${name}.yml/badge.svg" alt="${name}"></a>'';
    rows = lib.concatMapStrings (c: "<tr><td><code>${c}</code></td></tr>\n") (builtins.attrNames config.checks);
  in {
    packages.site-index = pkgs.writeText "index.html" ''
      <!doctype html>
      <meta charset="utf-8">
      <title>hakmem</title>
      <style>
        body { font: 16px/1.5 system-ui, sans-serif; max-width: 52rem; margin: 3rem auto; padding: 0 1rem; color: #222; }
        table { border-collapse: collapse; }
        td, th { border-bottom: 1px solid #ddd; padding: 0.3rem 0.8rem; text-align: left; }
        .badges img { vertical-align: middle; margin-right: 0.4rem; }
      </style>
      <h1>hakmem</h1>
      <p>Bit tricks as a lawful algebra; a learning project, see the README's Status section.
         <a href="doc/hakmem/">API docs (main)</a> ·
         <a href="dev/bench/">benchmarks over time</a> ·
         <a href="https://github.com/${repo}">source</a></p>
      <p class="badges">${badge "ci-x86_64"} ${badge "ci-aarch64"} ${badge "pages"}</p>
      <h2>What CI checks on ${system}</h2>
      <p>Every row is one Nix derivation built in a sandbox without network;
         the matrix lives in <code>nix/matrix.nix</code> and the workflow reads
         it at run time. Miri runs as a separate job; benchmarks are tracked
         over time, never pass/fail.</p>
      <table><tr><th>check</th></tr>
      ${rows}</table>
    '';
  };
}
