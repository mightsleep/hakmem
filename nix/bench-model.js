// Data model of the bench page: github-action-benchmark's data.js
// (window.BENCHMARK_DATA) turned into the three views. No DOM here, so
// `deno run` can exercise it against a real data.js; nix/bench.nix embeds
// it into the page. Bench names are "group/impl/param" (criterion group,
// BenchmarkId function, BenchmarkId parameter); an impl whose name starts
// with "hakmem" is ours, everything else is an incumbent.

function parseName(name) {
  const parts = name.split("/");
  return { group: parts[0], impl: parts[1] ?? "", param: parts.slice(2).join("/") };
}

function isOurs(impl) {
  return impl.startsWith("hakmem");
}

function pushUnique(list, x) {
  if (!list.includes(x)) list.push(x);
}

// One run: { group: { params, impls, value: { impl: { param: ns } } } },
// params and impls in first-seen order, which is the order in the bench.
function tabulate(run) {
  const groups = {};
  for (const b of run.benches) {
    const { group, impl, param } = parseName(b.name);
    const g = (groups[group] ??= { params: [], impls: [], value: {} });
    pushUnique(g.params, param);
    pushUnique(g.impls, impl);
    (g.value[impl] ??= {})[param] = b.value;
  }
  return groups;
}

// All runs of all suites, oldest first.
function runs(data) {
  return Object.values(data.entries)
    .flat()
    .sort((a, b) => a.date - b.date)
    .map((r) => ({
      sha: r.commit.id.slice(0, 7),
      url: r.commit.url,
      date: new Date(r.date),
      message: r.commit.message.split("\n")[0],
      benches: r.benches.length,
      groups: tabulate(r),
    }));
}

function minOver(value, impls, param) {
  let m = Infinity;
  for (const i of impls) {
    const v = value[i]?.[param];
    if (v !== undefined && v < m) m = v;
  }
  return m === Infinity ? undefined : m;
}

// The implementations a group compares are whatever the newest run has.
// Older runs may carry cells the bench has since dropped, and those must
// not shape the history. { group: { ours, theirs } }
function currentImpls(rs) {
  const out = {};
  const last = rs.at(-1);
  if (!last) return out;
  for (const [name, g] of Object.entries(last.groups)) {
    out[name] = { ours: g.impls.filter(isOurs), theirs: g.impls.filter((i) => !isOurs(i)) };
  }
  return out;
}

// Per group that has both sides: hakmem's best variant over the best
// incumbent, per param, per run. Dimensionless, so a change of runner
// leaves it alone. { group: { params, points: [{ sha, ratio: { param } }] } }
function ratioSeries(rs) {
  const out = {};
  const current = currentImpls(rs);
  for (const r of rs) {
    for (const [name, g] of Object.entries(r.groups)) {
      if (!current[name]) continue;
      const { ours, theirs } = current[name];
      if (!ours.length || !theirs.length) continue;
      const o = (out[name] ??= { params: [], points: [] });
      const ratio = {};
      for (const p of g.params) {
        pushUnique(o.params, p);
        const a = minOver(g.value, ours, p);
        const b = minOver(g.value, theirs, p);
        if (a !== undefined && b !== undefined) ratio[p] = a / b;
      }
      o.points.push({ sha: r.sha, ratio });
    }
  }
  return out;
}

// Per group with a hakmem side: its best variant in ns, per param, per run.
// This one moves with the runner's CPU. { group: { params, points: [{ sha, ns: { param } }] } }
function oursSeries(rs) {
  const out = {};
  const current = currentImpls(rs);
  for (const r of rs) {
    for (const [name, g] of Object.entries(r.groups)) {
      if (!current[name] || !current[name].ours.length) continue;
      const { ours } = current[name];
      const o = (out[name] ??= { params: [], points: [] });
      const ns = {};
      for (const p of g.params) {
        pushUnique(o.params, p);
        const v = minOver(g.value, ours, p);
        if (v !== undefined) ns[p] = v;
      }
      o.points.push({ sha: r.sha, ns });
    }
  }
  return out;
}

function formatNs(ns) {
  if (ns >= 1e6) return (ns / 1e6).toFixed(2) + " ms";
  if (ns >= 1e3) return (ns / 1e3).toFixed(2) + " µs";
  return Math.round(ns) + " ns";
}

// Bounds for a log axis: whole decades around the data with a margin, so a
// few percent of noise draws flat instead of filling the chart, and every
// chart of a view can share the same bounds. `anchor` is forced into the
// range (1 for ratios, so parity is always on the axis).
function decadeRange(values, anchor) {
  const v = values.filter((x) => typeof x === "number" && x > 0);
  if (anchor !== undefined) v.push(anchor);
  if (!v.length) return { min: 1, max: 10 };
  const lo = Math.min(...v);
  const hi = Math.max(...v);
  let min = 10 ** Math.floor(Math.log10(lo));
  let max = 10 ** Math.ceil(Math.log10(hi));
  if (min === lo) min /= 10;
  if (max === hi) max *= 10;
  return { min, max };
}

function isDecade(v) {
  const l = Math.log10(v);
  return Math.abs(l - Math.round(l)) < 1e-9;
}

// A history as percent of its first run, per param: 100 is the first run
// that had the value. Linear and dimensionless, so one axis serves every
// size and every group, and a 20 % regression is the same height at 262 ns
// and at 28 µs. `abs` keeps the original values for tooltips.
// { group: { params, points: [{ sha, pct: { param }, abs: { param } }] } }
function relativeSeries(series, key) {
  const out = {};
  for (const [name, s] of Object.entries(series)) {
    const base = {};
    out[name] = {
      params: s.params,
      points: s.points.map((pt) => {
        const pct = {};
        for (const p of s.params) {
          const v = pt[key][p];
          if (v === undefined) continue;
          base[p] ??= v;
          pct[p] = (v / base[p]) * 100;
        }
        return { sha: pt.sha, pct, abs: pt[key] };
      }),
    };
  }
  return out;
}

// Bounds for the percent axis: tens, at least 90 to 110, with a margin.
function percentRange(values) {
  const v = values.filter((x) => typeof x === "number" && Number.isFinite(x));
  let min = 90;
  let max = 110;
  for (const x of v) {
    if (x - 5 < min) min = Math.floor((x - 5) / 10) * 10;
    if (x + 5 > max) max = Math.ceil((x + 5) / 10) * 10;
  }
  return { min: Math.max(0, min), max };
}
