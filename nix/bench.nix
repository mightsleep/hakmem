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
         cancels out. The charts show each ratio as percent of its first run,
         100 being where it started. The table holds the newest values: 0.1
         means ten times faster, above 1 the incumbent wins.</p>
      <div id="ratio"></div>
      <table id="ratios">
        <thead><tr><th>group</th><th>size</th><th>ratio</th><th>hakmem</th><th>best incumbent</th></tr></thead>
        <tbody></tbody>
      </table>

      <h2>hakmem alone</h2>
      <p>Best hakmem variant per size as percent of its first run. This view
         moves with the runner's CPU: a step in every size at once is a
         different runner, a step in one size is the code.</p>
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
          // Newest run: a log axis in whole decades, shared by its charts, with
          // labels only at the decades. It is the only way to have 260 ns and
          // 656 µs in one picture; a centimetre is a factor, not a difference.
          var logAxis = function (title, range, fmt) {
            return {
              type: "logarithmic", min: range.min, max: range.max,
              title: { display: true, text: title },
              ticks: { autoSkip: false, callback: function (v) { return isDecade(v) ? fmt(v) : null; } },
            };
          };
          // Histories: linear percent of the first run, one range for all of
          // them, so a centimetre is the same ten percent in every chart.
          var pctAxis = function (range) {
            return {
              type: "linear", min: range.min, max: range.max,
              title: { display: true, text: "% of first run" },
              ticks: { stepSize: 10, callback: function (v) { return v + " %"; } },
              grid: { color: function (ctx) { return ctx.tick.value === 100 ? tok("overlay0") : tok("surface0"); } },
            };
          };
          var nsTooltip = { callbacks: { label: function (c) { return c.dataset.label + ": " + formatNs(c.parsed.y); } } };
          var pctTooltip = function (fmt) {
            return { callbacks: { label: function (c) {
              return c.dataset.label + ": " + c.parsed.y.toFixed(1) + " % (" + fmt(c.dataset.abs[c.dataIndex]) + ")";
            } } };
          };
          var pctValues = function (series) {
            var out = [];
            Object.keys(series).forEach(function (name) {
              series[name].points.forEach(function (pt) {
                Object.keys(pt.pct).forEach(function (p) { out.push(pt.pct[p]); });
              });
            });
            return out;
          };
          var histories = function (parent, series, range, fmt) {
            Object.keys(series).forEach(function (name) {
              var s = series[name];
              chart(parent, name, {
                type: "line",
                data: {
                  labels: s.points.map(function (p) { return p.sha; }),
                  datasets: s.params.map(function (p, k) {
                    var colour = pick(accents, k);
                    return { label: p || name,
                             data: s.points.map(function (pt) { return pt.pct[p] === undefined ? null : pt.pct[p]; }),
                             abs: s.points.map(function (pt) { return pt.abs[p]; }),
                             borderColor: colour, backgroundColor: colour, tension: 0, spanGaps: true };
                  }),
                },
                options: { scales: { y: pctAxis(range) }, plugins: { tooltip: pctTooltip(fmt) } },
              });
            });
          };
          var cell = function (tr, text) {
            var td = document.createElement("td");
            td.textContent = text;
            tr.appendChild(td);
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
          var latestValues = [];
          Object.keys(last.groups).forEach(function (name) {
            var g = last.groups[name];
            g.impls.forEach(function (impl) { g.params.forEach(function (p) { latestValues.push(g.value[impl][p]); }); });
          });
          var latestRange = decadeRange(latestValues);
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
                options: { scales: { y: logAxis("ns/iter, log", latestRange, formatNs) }, plugins: { tooltip: nsTooltip } },
              };
            } else {
              config = {
                type: "bar",
                data: {
                  labels: g.impls,
                  datasets: [{ label: "ns/iter", data: g.impls.map(function (impl) { return g.value[impl][""]; }),
                               backgroundColor: g.impls.map(colourFor) }],
                },
                options: { scales: { y: logAxis("ns/iter, log", latestRange, formatNs) }, plugins: { legend: { display: false }, tooltip: nsTooltip } },
              };
            }
            chart(latest, name, config);
          });

          var ratios = ratioSeries(rs);
          var relRatios = relativeSeries(ratios, "ratio");
          var relOurs = relativeSeries(oursSeries(rs), "ns");
          var range = percentRange(pctValues(relRatios).concat(pctValues(relOurs)));
          histories(document.getElementById("ratio"), relRatios, range, function (v) { return v.toFixed(3); });
          histories(document.getElementById("ours"), relOurs, range, formatNs);

          var sides = currentImpls(rs);
          var rbody = document.querySelector("#ratios tbody");
          Object.keys(ratios).forEach(function (name) {
            var g = last.groups[name];
            var newestRatio = ratios[name].points[ratios[name].points.length - 1].ratio;
            g.params.forEach(function (p) {
              if (newestRatio[p] === undefined) return;
              var tr = document.createElement("tr");
              cell(tr, name);
              cell(tr, p || "-");
              cell(tr, newestRatio[p].toFixed(3));
              cell(tr, formatNs(minOver(g.value, sides[name].ours, p)));
              cell(tr, formatNs(minOver(g.value, sides[name].theirs, p)));
              rbody.appendChild(tr);
            });
          });

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
              var c = document.createElement("td");
              if (i === 2) c.className = "msg";
              c.textContent = text;
              tr.appendChild(c);
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
