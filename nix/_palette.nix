# Catppuccin Latte (default) and Mocha (dark colour scheme) as CSS custom
# properties, plus the base styles both gh-pages pages share. Not a
# flake-parts module: import-tree skips files starting with `_`, and
# nix/site.nix and nix/bench.nix `import` this string.
''
  :root {
    --base: #eff1f5; --mantle: #e6e9ef; --crust: #dce0e8;
    --surface0: #ccd0da; --surface1: #bcc0cc; --surface2: #acb0be;
    --overlay0: #9ca0b0; --overlay1: #8c8fa1; --overlay2: #7c7f93;
    --subtext0: #6c6f85; --subtext1: #5c5f77; --text: #4c4f69;
    --blue: #1e66f5; --lavender: #7287fd; --sapphire: #209fb5; --sky: #04a5e5;
    --teal: #179299; --green: #40a02b; --yellow: #df8e1d; --peach: #fe640b;
    --maroon: #e64553; --red: #d20f39; --mauve: #8839ef; --pink: #ea76cb;
  }
  @media (prefers-color-scheme: dark) {
    :root {
      --base: #1e1e2e; --mantle: #181825; --crust: #11111b;
      --surface0: #313244; --surface1: #45475a; --surface2: #585b70;
      --overlay0: #6c7086; --overlay1: #7f849c; --overlay2: #9399b2;
      --subtext0: #a6adc8; --subtext1: #bac2de; --text: #cdd6f4;
      --blue: #89b4fa; --lavender: #b4befe; --sapphire: #74c7ec; --sky: #89dceb;
      --teal: #94e2d5; --green: #a6e3a1; --yellow: #f9e2af; --peach: #fab387;
      --maroon: #eba0ac; --red: #f38ba8; --mauve: #cba6f7; --pink: #f5c2e7;
    }
  }
  html { background: var(--base); color: var(--text); }
  body { font: 16px/1.5 system-ui, sans-serif; max-width: 52rem; margin: 3rem auto; padding: 0 1rem; }
  h1 { font-size: 2rem; margin: 0; }
  h1 + p { color: var(--subtext0); margin-top: 0.25rem; }
  h2 { font-size: 1.25rem; margin-top: 2.5rem; }
  h3 { font-size: 1rem; margin: 1.5rem 0 0.5rem; color: var(--subtext1); }
  a { color: var(--blue); text-decoration: none; }
  a:hover { text-decoration: underline; }
  nav a + a::before { content: "·"; color: var(--overlay0); margin: 0 0.6rem; }
  code { font-family: ui-monospace, monospace; font-size: 0.85em; color: var(--mauve); }
  table { border-collapse: collapse; width: 100%; background: var(--mantle); border-radius: 0.5rem; overflow: hidden; }
  th, td { padding: 0.5rem 0.9rem; text-align: left; border-top: 1px solid var(--surface0); }
  thead th { background: var(--surface0); border-top: 0; font-weight: 600; }
  th[scope=row] { font-weight: 400; }
  th[scope=row] code { color: var(--subtext0); }
  footer { margin-top: 3rem; color: var(--subtext0); font-size: 0.9rem; border-top: 1px solid var(--surface0); padding-top: 1rem; }
''
