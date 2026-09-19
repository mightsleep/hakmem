# gh-pages bench page (dev/bench/index.html). github-action-benchmark
# appends every run to data.js next to it and writes its own index.html
# only when none exists, so this page replaces it for good. Three views
# over the same data: the newest run per group, hakmem over the best
# incumbent per run (dimensionless, so it survives a change of runner),
# and hakmem's own times per run. The model is nix/bench-model.js, run
# with deno against a real data.js before it ships; palette from
# _palette.nix. Chart.js is the `chartjs` flake input (a semver-range URL,
# bumped by the weekly flake.lock PR, bytes pinned by the lock) and ships
# next to the page, so the browser loads nothing from a third party.
# Built in the pages workflow; not kept in the repository.
{inputs, ...}: {
  perSystem = {pkgs, ...}: let
    html = pkgs.writeText "bench-index.html" ''
      <!doctype html>
      <html lang="en">
      <meta charset="utf-8">
      <meta name="viewport" content="width=device-width, initial-scale=1">
      <title>hakmem benchmarks</title>
      <style>
      ${import ./_palette.nix}
        .chart { background: var(--mantle); border-radius: 0.5rem; padding: 0.75rem 1rem 1rem; margin: 0.75rem 0; }
        .chart h3 { margin: 0 0 0.5rem; }
        .chart .canvas { position: relative; height: 20rem; }
        .note { color: var(--subtext0); }
        td.msg { color: var(--subtext0); }
      </style>
      <h1>hakmem benchmarks</h1>
      <p>Criterion on a shared GitHub runner, after every push to <code>main</code>
         that touches <code>src/</code>, <code>benches/</code> or the Cargo files.
         Lower is better everywhere.</p>
      <nav>
        <a href="../../">checks</a>
        <a href="../../doc/hakmem/">docs</a>
        <a href="https://github.com/mightsleep/hakmem/blob/main/benches/incumbents.rs">bench source</a>
        <a href="data.js">data.js</a>
      </nav>

      <h2>Newest run</h2>
      <p id="newest" class="note">Loading data.js</p>
      <div id="latest"></div>

      <h2>hakmem over the best incumbent</h2>
      <p>Best hakmem variant divided by the best incumbent in the same run, per
         size. Both sides share the machine and the minute, so a slower runner
         cancels out. 0.1 means ten times faster; above 1 the incumbent wins.</p>
      <div id="ratio"></div>

      <h2>hakmem alone</h2>
      <p>Best hakmem variant per size, in time. This view moves with the
         runner's CPU: read it for trends across runs on the same hardware,
         not across hardware.</p>
      <div id="ours"></div>

      <h2>Runs</h2>
      <table id="runs">
        <thead><tr><th>commit</th><th>date</th><th>benches</th><th>message</th></tr></thead>
        <tbody></tbody>
      </table>

      <footer>Collected by github-action-benchmark into <code>data.js</code>; a 2×
        regression of any single bench comments on the commit. Numbers on a
        known machine are in the README.</footer>

      <script src="chart.umd.js"></script>
      <script src="data.js"></script>
      <script>
      ${builtins.readFile ./bench-model.js}
      </script>
      <script>
        (function () {
          var css = getComputedStyle(document.documentElement);
          var tok = function (n) { return css.getPropertyValue("--" + n).trim(); };
          var accents = ["blue", "mauve", "lavender", "sapphire", "teal", "pink", "sky", "peach"].map(tok);
          var greys = ["overlay2", "overlay1", "overlay0", "surface2", "subtext0"].map(tok);
          var pick = function (list, i) { return list[i % list.length]; };
          Chart.defaults.color = tok("subtext1");
          Chart.defaults.borderColor = tok("surface0");
          Chart.defaults.font.family = "system-ui, sans-serif";

          var chart = function (parent, title, config) {
            var box = document.createElement("div");
            box.className = "chart";
            var h = document.createElement("h3");
            h.textContent = title;
            var wrap = document.createElement("div");
            wrap.className = "canvas";
            var canvas = document.createElement("canvas");
            wrap.appendChild(canvas);
            box.appendChild(h);
            box.appendChild(wrap);
            parent.appendChild(box);
            config.options = Object.assign({ responsive: true, maintainAspectRatio: false }, config.options);
            new Chart(canvas, config);
          };
          var logNs = function (title) {
            return { type: "logarithmic", title: { display: true, text: title },
                     ticks: { callback: function (v) { return formatNs(v); } } };
          };
          var nsTooltip = { callbacks: { label: function (c) { return c.dataset.label + ": " + formatNs(c.parsed.y); } } };
          var lines = function (parent, series, key, yAxis, tooltip) {
            Object.keys(series).forEach(function (name) {
              var s = series[name];
              chart(parent, name, {
                type: "line",
                data: {
                  labels: s.points.map(function (p) { return p.sha; }),
                  datasets: s.params.map(function (p, k) {
                    var colour = pick(accents, k);
                    return { label: p || name, data: s.points.map(function (pt) { return pt[key][p] === undefined ? null : pt[key][p]; }),
                             borderColor: colour, backgroundColor: colour, tension: 0, spanGaps: true };
                  }),
                },
                options: { scales: { y: yAxis }, plugins: { tooltip: tooltip } },
              });
            });
          };

          var data = window.BENCHMARK_DATA;
          var newest = document.getElementById("newest");
          if (!data) {
            newest.textContent = "No data.js yet: the first bench run after this page was published creates it.";
            return;
          }
          var rs = runs(data);
          var last = rs[rs.length - 1];
          newest.innerHTML = "";
          var link = document.createElement("a");
          link.href = last.url;
          link.textContent = last.sha;
          newest.appendChild(link);
          newest.appendChild(document.createTextNode(", " + last.date.toISOString().slice(0, 10) + ", " + last.benches + " benches. " + last.message));

          var latest = document.getElementById("latest");
          Object.keys(last.groups).forEach(function (name) {
            var g = last.groups[name];
            var byParam = g.params.length > 1 || g.params[0] !== "";
            var anyOurs = g.impls.some(isOurs);
            var oi = 0, ti = 0;
            var colourFor = function (impl) {
              return (isOurs(impl) || !anyOurs) ? pick(accents, oi++) : pick(greys, ti++);
            };
            var config;
            if (byParam) {
              config = {
                type: "bar",
                data: {
                  labels: g.params,
                  datasets: g.impls.map(function (impl) {
                    return { label: impl, backgroundColor: colourFor(impl),
                             data: g.params.map(function (p) { var v = g.value[impl][p]; return v === undefined ? null : v; }) };
                  }),
                },
                options: { scales: { y: logNs("ns/iter, log") }, plugins: { tooltip: nsTooltip } },
              };
            } else {
              config = {
                type: "bar",
                data: {
                  labels: g.impls,
                  datasets: [{ label: "ns/iter", data: g.impls.map(function (impl) { return g.value[impl][""]; }),
                               backgroundColor: g.impls.map(colourFor) }],
                },
                options: { scales: { y: logNs("ns/iter, log") }, plugins: { legend: { display: false }, tooltip: nsTooltip } },
              };
            }
            chart(latest, name, config);
          });

          lines(document.getElementById("ratio"), ratioSeries(rs), "ratio",
                { type: "logarithmic", title: { display: true, text: "hakmem / best incumbent, log" } },
                { callbacks: { label: function (c) { return c.dataset.label + ": " + c.parsed.y.toFixed(3); } } });
          lines(document.getElementById("ours"), oursSeries(rs), "ns", logNs("ns/iter, log"), nsTooltip);

          var tbody = document.querySelector("#runs tbody");
          rs.slice().reverse().forEach(function (r) {
            var tr = document.createElement("tr");
            var td = document.createElement("td");
            var a = document.createElement("a");
            a.href = r.url;
            a.textContent = r.sha;
            td.appendChild(a);
            tr.appendChild(td);
            [r.date.toISOString().slice(0, 10), String(r.benches), r.message].forEach(function (text, i) {
              var cell = document.createElement("td");
              if (i === 2) cell.className = "msg";
              cell.textContent = text;
              tr.appendChild(cell);
            });
            tbody.appendChild(tr);
          });
        })();
      </script>
    '';
  in {
    # The page and its one dependency as a directory; the pages workflow
    # copies it over dev/bench/ on gh-pages, where data.js already lives.
    packages.bench-index = pkgs.runCommand "bench-index" {passthru = {inherit html;};} ''
      mkdir -p $out
      cp ${html} $out/index.html
      # A `file` input is the file itself; fail loudly if that ever changes.
      test -f ${inputs.chartjs}
      cp ${inputs.chartjs} $out/chart.umd.js
    '';
  };
}
