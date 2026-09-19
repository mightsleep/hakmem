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
