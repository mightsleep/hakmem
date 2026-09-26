# The GitHub workflows, the status action, dependabot's and zizmor's
# config, as Nix values rendered to .github/ (nix/_yaml.nix). GitHub reads
# only committed files, so they stay in the repository; this is where they
# are written.
#
#   nix run .#snapshots -- workflows   write them (a snapshot, nix/hakmem.nix)
#   checks.workflows                   the committed files are the rendered ones,
#                                      parse back to what they were rendered
#                                      from, and pass actionlint and zizmor
#
# Actions are pinned by commit in nix/actions.json, with the tag as a
# comment; `nix run .#bump-actions` moves them. Dependabot no longer
# watches actions: it would edit files this module owns.
{lib, ...}: let
  yaml = import ./_yaml.nix {inherit lib;};
  inherit (yaml) comment raw ordered;
  pins = lib.importJSON ./actions.json;

  # `${{ expr }}`, which Nix would read as an interpolation.
  gh = expr: "\${{ ${expr} }}";
  use = repo: raw "${repo}@${pins.${repo}.sha} # ${pins.${repo}.ref}";

  # No credentials left in .git for later steps (zizmor: artipacked); a
  # step that pushes says with which token.
  checkout = {
    uses = use "actions/checkout";
    "with".persist-credentials = false;
  };
  # The flake is written with `|>`.
  installNix = {
    uses = use "DeterminateSystems/nix-installer-action";
    "with".extra-conf = "extra-experimental-features = pipe-operators";
  };
  cachix = {
    uses = use "cachix/cachix-action";
    "with" = {
      name = "hakmem";
      authToken = gh "secrets.CACHIX_AUTH_TOKEN";
    };
  };
  # The cache, read only.
  cachixRead = {
    uses = use "cachix/cachix-action";
    "with".name = "hakmem";
  };
  status = args: {
    uses = "./.github/actions/status";
    "with" = args;
  };
  # `>-` in the files these came from: lines joined by a space.
  fold = lib.replaceStrings ["\n"] [" "];
  onMainAndPRs = {
    push.branches = ["main"];
    pull_request = {};
  };
  bot = ''
    git config user.name "github-actions[bot]"
    git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
  '';

  ci = {
    name,
    system,
    runner,
    miri ? false,
  }:
    comment ''
      The matrix lives in the flake: `plan` reads the check names and asks the
      binary cache which outputs already exist, `check` builds only the rest
      (.github/plan.sh). A cell whose output is in the cache passed by
      construction; `status` records it as such.''
    {
      inherit name;
      on = onMainAndPRs;
      permissions = {};
      jobs = ordered ["plan" "check" "miri" "pending" "status"] (
        {
          plan = {
            permissions.contents = "read";
            runs-on = runner;
            outputs = {
              checks = gh "steps.plan.outputs.checks";
              cached = gh "steps.plan.outputs.cached";
            };
            steps = [
              checkout
              installNix
              {
                id = "plan";
                run = ".github/plan.sh ${system}";
              }
            ];
          };
          check = comment "GitHub rejects an empty matrix; nothing to build is a valid plan." {
            permissions.contents = "read";
            needs = "plan";
            "if" = "needs.plan.outputs.checks != '[]'";
            name = gh "matrix.check";
            runs-on = runner;
            strategy = {
              fail-fast = false;
              matrix.check = gh "fromJSON(needs.plan.outputs.checks)";
            };
            steps = [
              checkout
              installNix
              cachix
              {
                env.CHECK = gh "matrix.check";
                run = "nix build -L \".#checks.${system}.$CHECK\"";
              }
            ];
          };
          pending =
            comment ''
              A grey placeholder for every cell about to build that has no badge
              yet: a missing file is a red "resource not found" in the README, and a
              check new in this push has none until the run ends.''
            {
              needs = "plan";
              "if" = "github.event_name == 'push' && needs.plan.outputs.checks != '[]'";
              runs-on = "ubuntu-latest";
              permissions.contents = "write";
              steps = [
                checkout
                (status {
                  inherit system;
                  pending = gh "needs.plan.outputs.checks";
                })
              ];
            };
          status =
            comment ''
              Result of every cell to gh-pages for the README matrix and the site.
              Pushes to main only: a PR run must not overwrite the status of main.''
            {
              needs = ["plan" "pending" "check"] ++ lib.optional miri "miri";
              "if" = "always() && github.event_name == 'push'";
              runs-on = "ubuntu-latest";
              permissions = {
                actions = "read";
                contents = "write";
              };
              steps = [
                checkout
                (status {
                  inherit system;
                  cached = gh "needs.plan.outputs.cached";
                })
              ];
            };
        }
        // lib.optionalAttrs miri {
          miri = {
            permissions.contents = "read";
            runs-on = runner;
            steps = [checkout installNix cachix {run = "nix run .#miri-hakmem";}];
          };
        }
      );
    };

  # cargo-semver-checks from nix/semver.nix.
  semverSteps = [checkout installNix cachixRead {run = "nix run .#semver";}];

  workflows = {
    "workflows/ci-x86_64.yml" = ci {
      name = "x86_64";
      system = "x86_64-linux";
      runner = "ubuntu-latest";
      miri = true;
    };
    "workflows/ci-aarch64.yml" = ci {
      name = "aarch64";
      system = "aarch64-linux";
      runner = "ubuntu-24.04-arm";
    };

    "workflows/semver.yml" =
      comment ''
        The last release against this tree: cargo-semver-checks says whether
        the version in Cargo.toml went up enough for what changed. Outside the
        nix matrix: it needs the registry for the baseline, so it is an app
        (nix/semver.nix), not a check. `public-api/` (a nix check) sees every
        change; this one says which of them break.''
      {
        name = "semver";
        on = onMainAndPRs;
        permissions = {};
        jobs.semver = {
          runs-on = "ubuntu-latest";
          permissions.contents = "read";
          steps = semverSteps;
        };
      };

    "workflows/update-flake-lock.yml" =
      comment ''
        Weekly PRs: the flake inputs (nixpkgs, fenix, crane, advisory-db; CI on
        that PR runs hakmem-audit against the fresh RustSec DB) and the action
        pins.''
      {
        name = "update-flake-lock";
        on = {
          schedule = [{cron = "23 5 * * 1";}];
          workflow_dispatch = {};
        };
        permissions = {};
        jobs = ordered ["update" "actions"] {
          update = {
            runs-on = "ubuntu-latest";
            permissions = {
              contents = "write";
              pull-requests = "write";
            };
            steps = [
              checkout
              installNix
              {
                uses = use "DeterminateSystems/update-flake-lock";
                "with" =
                  comment ''
                    A PR opened with the default GITHUB_TOKEN triggers no workflows, so
                    CI would never run on it. FLAKE_LOCK_TOKEN is a fine-grained PAT
                    (contents + pull requests, this repo only); until it exists the
                    default token still opens the PR, just without CI.''
                  {
                    token = gh "secrets.FLAKE_LOCK_TOKEN || github.token";
                    pr-title = "flake.lock: weekly update";
                    pr-labels = "dependencies";
                  };
              }
            ];
          };
          actions =
            comment ''
              The action pins, which dependabot no longer watches: nix/actions.json
              moved by `nix run .#bump-actions`, the files rendered again, one PR.
              A push that changes .github/workflows needs a token with the
              Workflows permission: FLAKE_LOCK_TOKEN must have it, the default
              token never does.''
            {
              runs-on = "ubuntu-latest";
              permissions = {
                contents = "write";
                pull-requests = "write";
              };
              steps = [
                checkout
                installNix
                {
                  env = {
                    GH_TOKEN = gh "secrets.FLAKE_LOCK_TOKEN || github.token";
                    TOKEN = gh "secrets.FLAKE_LOCK_TOKEN || github.token";
                  };
                  run = ''
                    nix run .#bump-actions | tee changes.txt
                    [ -s changes.txt ] || exit 0
                    nix run .#snapshots -- workflows
                    ${bot}git switch -c bump-actions
                    git commit -qam "Bump action pins" -m "$(cat changes.txt)"
                    git push -f "https://x-access-token:$TOKEN@github.com/$GITHUB_REPOSITORY" bump-actions
                    gh pr view bump-actions >/dev/null 2>&1 \
                      || gh pr create --head bump-actions --title "Bump action pins" --body-file changes.txt --label dependencies
                  '';
                }
              ];
            };
        };
      };

    "workflows/pages.yml" = {
      name = "pages";
      on = {
        push.branches = ["main"];
        schedule = [{cron = "17 4 * * 1";}];
        workflow_dispatch = {};
      };
      permissions = {};
      jobs = ordered ["changes" "docs" "bench"] {
        changes =
          comment ''
            What the pushed range needs. rustdoc and the index always; the bench
            only when a file that can move a number changed. Schedule and manual
            runs have no `before` and bench always.''
          {
            permissions.contents = "read";
            runs-on = "ubuntu-latest";
            outputs.bench = gh "steps.diff.outputs.bench";
            steps = [
              {
                id = "diff";
                env = {
                  GH_TOKEN = gh "github.token";
                  BEFORE = gh "github.event.before";
                };
                run = ''
                  bench=true
                  if [ -n "$BEFORE" ] && [ "$BEFORE" != "0000000000000000000000000000000000000000" ]; then
                    # A force push can leave `before` unreachable; then bench rather than guess.
                    if files=$(gh api "repos/$GITHUB_REPOSITORY/compare/$BEFORE...$GITHUB_SHA" --jq '.files[].filename'); then
                      bench=false
                      echo "$files" | grep -qE '^(src/|benches/|Cargo\.toml$|Cargo\.lock$|\.github/workflows/pages\.yml$)' && bench=true
                    fi
                  fi
                  echo "bench=$bench" >> "$GITHUB_OUTPUT"
                  echo "bench=$bench"
                '';
              }
            ];
          };
        docs = {
          name = "rustdoc + index";
          permissions.contents = "write";
          runs-on = "ubuntu-latest";
          steps = [
            checkout
            installNix
            cachix
            {run = "nix build -L .#checks.x86_64-linux.hakmem-doc -o doc-result";}
            {run = "nix build -L .#site-index -o index-result";}
            {run = "nix build -L .#bench-index -o bench-index-result";}
            {
              run = ''
                mkdir -p site/dev/bench
                cp -r doc-result/share/doc site/doc
                cp index-result site/index.html
                # Replaces the default page of github-action-benchmark and ships
                # Chart.js next to it; data.js stays.
                cp -r bench-index-result/. site/dev/bench/
              '';
            }
            {
              uses = use "peaceiris/actions-gh-pages";
              "with" = comment "Bench data and the status JSON live on the same branch; keep them." {
                github_token = gh "secrets.GITHUB_TOKEN";
                publish_dir = "./site";
                keep_files = true;
              };
            }
          ];
        };
        bench =
          comment ''
            docs pushes to gh-pages too; serialise the two pushes. Timed outside
            the Nix sandbox on a shared runner: a trend, never pass/fail.''
          {
            name = "bench (criterion, tracked on gh-pages)";
            permissions.contents = "write";
            runs-on = "ubuntu-latest";
            needs = ["docs" "changes"];
            "if" = "needs.changes.outputs.bench == 'true'";
            steps = [
              checkout
              {uses = use "dtolnay/rust-toolchain";}
              (comment ''
                  Two seconds per bench: run-to-run noise on one runner type is a few
                  percent, five seconds bought nothing but fourteen-minute jobs.''
                {
                  run = "cargo bench --bench incumbents --bench select --bench compact --bench rank9 -- --output-format bencher --warm-up-time 1 --measurement-time 2 | tee bench.txt";
                  env.RUSTFLAGS = "-C target-cpu=native";
                })
              {
                uses = use "benchmark-action/github-action-benchmark";
                "with" = comment "Comment on the commit only at a 2× regression; a shared runner is noisy." {
                  tool = "cargo";
                  output-file-path = "bench.txt";
                  github-token = gh "secrets.GITHUB_TOKEN";
                  auto-push = true;
                  benchmark-data-dir-path = "dev/bench";
                  alert-threshold = "200%";
                  comment-on-alert = true;
                };
              }
            ];
          };
      };
    };

    "workflows/release.yml" = let
      tag = "github.ref_type == 'tag'";
    in
      comment ''
        crates.io from a tag, by trusted publishing: no stored token. The job
        waits in the crates-io environment for a maintainer's approval, then
        runs the checks `nix run .#publish` insists on (the tag is the version,
        the changelog dates it, the package's file list, the MSRV build of the
        tarball) and uploads the tarball those passed. Semver is checked first,
        against what crates.io has: it needs the registry, so it is not a nix
        check.

        The GitHub release is a job apart: the one holding the OIDC token cannot
        write to the repository, and "Re-run failed jobs" after a failed GitHub
        release does not upload again (and if it does run again, publish finds
        the same tarball on crates.io and passes).

        Outside a tag the same jobs run as a rehearsal: no environment, no
        token, `nix run .#publish` as a dry run that reports what only a tag
        can satisfy, and the GitHub release printed instead of made. The 0.2.0
        release is why: its first run stopped on a tree the semver action had
        left dirty, a step order no local dry run had.''
      {
        name = "release";
        on = {
          push.tags = ["v*"];
          pull_request.paths = [
            ".github/workflows/release.yml"
            "nix/workflows.nix"
            "nix/_yaml.nix"
            "nix/actions.json"
            "nix/semver.nix"
            "nix/release.nix"
            "nix/hakmem.nix"
            "nix/msrv.nix"
            "Cargo.toml"
            "Cargo.lock"
            "CHANGELOG.md"
            "release/**"
          ];
          workflow_dispatch = {};
        };
        permissions = {};
        jobs = ordered ["semver" "publish" "github-release"] {
          semver =
            comment ''
              Before the approval is asked for. A job of its own since the 0.2.0
              release, whose semver action left semver-checks/ in the checkout
              and publish refused the tree; `nix run .#semver` writes under
              target/, but the order is worth keeping anyway.''
            {
              runs-on = "ubuntu-latest";
              permissions.contents = "read";
              steps = semverSteps;
            };
          publish = {
            needs = "semver";
            runs-on = "ubuntu-latest";
            environment = gh "${tag} && 'crates-io' || ''";
            permissions = {
              contents = "read";
              id-token = "write";
            };
            steps = [
              checkout
              installNix
              cachixRead
              {
                id = "auth";
                "if" = tag;
                uses = use "rust-lang/crates-io-auth-action";
              }
              {
                run = "nix run .#publish";
                env = {
                  CARGO_REGISTRY_TOKEN = gh "steps.auth.outputs.token";
                  HAKMEM_PUBLISH_REHEARSAL = gh "github.ref_type != 'tag' && '1' || ''";
                };
              }
            ];
          };
          github-release =
            comment ''
              The release carries the tarball crates.io serves, with a signed
              SLSA provenance (Sigstore, through actions/attest): anyone can check
              that this workflow, at the tagged commit, released those bytes with
              `gh attestation verify hakmem-<version>.crate --repo <this repo>`.''
            {
              needs = "publish";
              runs-on = "ubuntu-latest";
              permissions = {
                contents = "write";
                id-token = "write";
                attestations = "write";
              };
              steps = [
                checkout
                {
                  name = "The tarball crates.io serves";
                  id = "crate";
                  "if" = tag;
                  env.TAG = gh "github.ref_name";
                  run = ''
                    v=''${TAG#v}
                    crate=hakmem-$v.crate
                    curl -sfL -A hakmem-release -o "$crate" "https://static.crates.io/crates/hakmem/$crate"
                    want=$(curl -sf -A hakmem-release "https://crates.io/api/v1/crates/hakmem/$v" | jq -r .version.checksum)
                    echo "$want  $crate" | sha256sum -c -
                    echo "path=$crate" >> "$GITHUB_OUTPUT"
                  '';
                }
                {
                  "if" = tag;
                  uses = use "actions/attest";
                  "with".subject-path = gh "steps.crate.outputs.path";
                }
                {
                  name = "GitHub release from the changelog section";
                  env = {
                    GH_TOKEN = gh "github.token";
                    TAG = gh "${tag} && github.ref_name || ''";
                    CRATE = gh "steps.crate.outputs.path";
                  };
                  run = ''
                    tag=''${TAG:-v$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)}
                    v=''${tag#v}
                    awk -v v="$v" '$0 ~ "^## "v"," {p = 1; next} /^## / {p = 0} p' CHANGELOG.md > notes.md
                    [ -s notes.md ] || { echo "CHANGELOG.md has no section for $v" >&2; exit 1; }
                    # `## 0.2.0, 2026-09-26: A Nocturne of Curves` names the release.
                    name=$(sed -n "s/^## $v, [0-9-]*: //p" CHANGELOG.md | head -n1)
                    title="$tag''${name:+: $name}"
                    if [ -z "$TAG" ]; then
                      printf 'rehearsal, the release would be:\n%s\n\n' "$title"
                      cat notes.md
                    elif gh release view "$tag" >/dev/null 2>&1; then
                      echo "release $tag exists already"
                      gh release upload "$tag" "$CRATE" --clobber
                    else
                      gh release create "$tag" "$CRATE" --notes-file notes.md --verify-tag --title "$title"
                    fi
                  '';
                }
              ];
            };
        };
      };

    "actions/status/action.yml" = {
      name = "status";
      description = fold ''
        Write the result of every job of this run to gh-pages as shields.io
        endpoint JSON (status/<system>/<check>.json). The README matrix and the
        site index read those files. With `pending`, before the build, write a
        grey placeholder for each listed check that has no file yet.'';
      inputs = {
        system = {
          description = "Nix system of the matrix this run built (x86_64-linux, aarch64-linux).";
          required = true;
        };
        cached = {
          description = "JSON array of checks the plan skipped as cache hits (recorded as pass).";
          required = false;
          default = "[]";
        };
        pending = {
          description = fold ''
            JSON array of checks about to build. When set, only placeholders are
            written, for the ones without a file, and no results.'';
          required = false;
          default = "";
        };
      };
      runs = {
        using = "composite";
        steps = [
          (lib.recursiveUpdate checkout {
            "with" = {
              ref = "gh-pages";
              path = "site";
            };
          })
          {
            shell = "bash";
            env = {
              GH_TOKEN = gh "github.token";
              SYSTEM = gh "inputs.system";
              CACHED = gh "inputs.cached";
              PENDING = gh "inputs.pending";
            };
            run = ''
              if [ -n "$PENDING" ]; then
                printf '%s' "$PENDING" > pending.json
                .github/status.sh --pending "$SYSTEM" pending.json site/status
                echo "what=pending" >> "$GITHUB_ENV"
              else
                gh api "repos/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/jobs?per_page=100" > jobs.json
                printf '%s' "$CACHED" > cached.json
                .github/status.sh "$SYSTEM" jobs.json site/status cached.json
                echo "what=status" >> "$GITHUB_ENV"
              fi
            '';
          }
          {
            shell = "bash";
            working-directory = "site";
            env = {
              GH_TOKEN = gh "github.token";
              SYSTEM = gh "inputs.system";
            };
            run = ''
              ${bot}git add status
              git commit -qm "$what: $SYSTEM ''${GITHUB_SHA::8}" || exit 0
              # gh-pages is shared with the other CI workflow and with pages.yml:
              # rebase on a lost race and retry. The checkout kept no token.
              remote="https://x-access-token:$GH_TOKEN@github.com/$GITHUB_REPOSITORY"
              for _ in 1 2 3 4 5; do
                git push "$remote" HEAD:gh-pages && exit 0
                git pull --rebase -q "$remote" gh-pages
              done
              exit 1
            '';
          }
        ];
      };
    };

    "dependabot.yml" =
      comment ''
        Dependabot: alerts over Cargo.lock are enabled under Settings, Security;
        these are the weekly bump PRs. Nix inputs are bumped by update-flake-lock.yml,
        actions by `nix run .#bump-actions` (nix/actions.json).''
      {
        version = 2;
        updates = [
          {
            package-ecosystem = "cargo";
            directory = "/";
            schedule.interval = "weekly";
            cooldown = comment ''
              A release a week old before it is proposed: time for a
              compromised one to be yanked (zizmor: dependabot-cooldown).''
            {default-days = 7;};
          }
        ];
      };

    "zizmor.yml" = comment "zizmor's configuration, for checks.workflows." {
      rules.self-repository = comment ''
        `$/.github/actions/status` instead of `./`: actionlint 1.7.12 does not
        know the syntax yet and rejects it. Again when it does.''
      {disable = true;};
    };
  };

  header = file: "Generated from nix/workflows.nix (${file}): edit there, then `nix run .#snapshots -- workflows`.";
in {
  perSystem = {pkgs, ...}: let
    rendered =
      workflows
      |> lib.mapAttrsToList (file: v: ''
        mkdir -p "$out/$(dirname ${file})"
        cp ${pkgs.writeText (baseNameOf file) (yaml.toYAML (header file) v)} "$out/${file}"
      '')
      |> lib.concatStrings
      |> pkgs.runCommand "hakmem-workflows" {};

    # What the files and the corpus mean, for the parser half of the check.
    json = name: v: builtins.toJSON v |> pkgs.writeText "${name}.json";
    meaning = workflows |> lib.mapAttrs (_: yaml.toData) |> json "hakmem-workflows";
    corpus = {
      yaml = yaml.toYAML "The edges of `plain` in nix/_yaml.nix." yaml.corpus |> pkgs.writeText "corpus.yml";
      json = json "corpus" yaml.corpus;
    };
    # A rendered file against what it was rendered from, through a parser.
    parsesBack = yml: want: ''
      remarshal -if yaml -of json ${yml} | jq -S . > got.json
      jq -S ${lib.escapeShellArg want.filter} ${want.file} > want.json
      diff -u want.json got.json || { echo "nix/_yaml.nix: ${yml} parses differently" >&2; fail=1; }
    '';
    roundTrips =
      (workflows
        |> lib.attrNames
        |> map (f:
          parsesBack "${rendered}/${f}" {
            file = meaning;
            filter = ".[${builtins.toJSON f}]";
          }))
      ++ [
        (parsesBack corpus.yaml {
          file = corpus.json;
          filter = ".";
        })
      ]
      |> lib.concatStrings;
  in {
    packages.workflows = rendered;

    # The files in sync, then: the emitter's own tests (at eval time), every
    # file and the corpus parsed back to what was meant, and actionlint (with
    # shellcheck over every `run:`) and zizmor clean, offline.
    hakmem.snapshots.workflows = {
      files = workflows |> lib.mapAttrs' (f: _: lib.nameValuePair ".github/${f}" "${rendered}/${f}");
      why = ".github/ is not what nix/workflows.nix renders.";
      description = ".github/ is what nix/workflows.nix renders: parsed back, actionlint, shellcheck, zizmor";
      inputs = [pkgs.remarshal pkgs.jq pkgs.actionlint pkgs.shellcheck pkgs.zizmor];
      before = assert yaml.tests == [] || throw "nix/_yaml.nix tests: ${builtins.toJSON yaml.tests}"; "";
      after = ''
        export HOME=$TMPDIR
        fail=0
        ${roundTrips}
        mkdir -p repo
        cp -r ${rendered} repo/.github
        chmod -R u+w repo
        cd repo
        actionlint .github/workflows/*.yml || fail=1
        zizmor --offline --no-progress --config .github/zizmor.yml .github || fail=1
        [ "$fail" = 0 ]
      '';
    };

    # Each pin to the newest release tag of its action (a branch pin to
    # the branch's head), by the commit the tag names: `git ls-remote`, no
    # token. Rewrites nix/actions.json; `nix run .#snapshots -- workflows`
    # then renders.
    apps.bump-actions = {
      type = "app";
      program = lib.getExe (pkgs.writeShellApplication {
        name = "hakmem-bump-actions";
        runtimeInputs = [pkgs.git pkgs.jq pkgs.gawk pkgs.coreutils pkgs.gnugrep pkgs.gnused];
        text = ''
          pins=nix/actions.json
          [ -f "$pins" ] || { echo "run it from the repository root" >&2; exit 1; }
          next=$(mktemp)
          trap 'rm -f "$next"' EXIT
          cp "$pins" "$next"
          for repo in $(jq -r 'keys[]' "$pins"); do
            ref=$(jq -r --arg r "$repo" '.[$r].ref' "$pins")
            refs=$(git ls-remote "https://github.com/$repo")
            if [[ $ref =~ ^v[0-9] ]]; then
              new=$(grep -oE 'refs/tags/v[0-9]+(\.[0-9]+){0,2}$' <<<"$refs" | sed 's|refs/tags/||' | sort -V | tail -n1)
              # An annotated tag names a tag object; the commit is its ^{}.
              sha=$(awk -v t="refs/tags/$new^{}" '$2 == t {print $1}' <<<"$refs")
              [ -n "$sha" ] || sha=$(awk -v t="refs/tags/$new" '$2 == t {print $1}' <<<"$refs")
            else
              new=$ref
              sha=$(awk -v t="refs/heads/$ref" '$2 == t {print $1}' <<<"$refs")
            fi
            [ -n "$sha" ] || { echo "$repo: no commit for $new" >&2; exit 1; }
            old=$(jq -r --arg r "$repo" '.[$r].sha' "$pins")
            [ "$sha" = "$old" ] || echo "$repo: $ref -> $new"
            jq --arg r "$repo" --arg ref "$new" --arg sha "$sha" '.[$r] = {ref: $ref, sha: $sha}' "$next" > "$next.tmp"
            mv "$next.tmp" "$next"
          done
          jq -r 'to_entries | map("  \(.key | tojson): { \"ref\": \(.value.ref | tojson), \"sha\": \(.value.sha | tojson) }") | "{\n" + join(",\n") + "\n}"' "$next" > "$pins"
        '';
      });
      meta.description = "Move the action pins in nix/actions.json to their newest releases";
    };
  };
}
