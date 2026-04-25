<div align="center">

<img src="docs/assets/readme-hero.svg" width="100%" alt="TypeSymbol hero banner" />

<br/>

[![Release](https://img.shields.io/github/v/release/yazanmwk/TypeSymbol?style=for-the-badge&logo=github&logoColor=fff&label=release&labelColor=18181b&color=3178C6)](https://github.com/yazanmwk/TypeSymbol/releases)
[![License](https://img.shields.io/badge/License-MIT-16a34a?style=for-the-badge&logo=opensourceinitiative&logoColor=white&labelColor=18181b)](LICENSE)

<br/>

<sub>Rust core · global daemon · macOS & Windows · CLI + TUI: ∫ prompt, <code>╭╮╰╯</code> panels, violet borders, gradient header</sub>

</div>

<br/>

# TypeSymbol

Type mathematical shorthand system-wide—`alpha` becomes `α`, `->` becomes `→`, and your formulas read like real math.

---

## See it in one glance

| You type (shorthand) | You get (Unicode) |
| :---: | :---: |
| `alpha -> beta` | **α → β** |
| `for all x in A` | **∀ x ∈ A** |
| `int 0 -> inf x` | **∫₀^∞ x dx** |
| `sum_(i=1)^n i^2` | **∑ᵢ₌₁ⁿ i²** |

*Transforms follow your [config](docs/install.md) (Greek, operators, integrals, sums, products, limits, transforms, set logic, probability/statistics, scripts, and more). Use `typesymbol test "..."` to preview any string.*

---

## What this is

TypeSymbol is a local, system-wide typing engine for math notation.

Instead of switching tools or opening symbol pickers, you keep typing in plain text and TypeSymbol expands it into Unicode math where you already work: notes, chats, docs, editors, and browsers.

It is built for one goal: reduce friction between thinking in math and writing in software.

---

## How it works

```mermaid
%%{init: { "theme": "base", "themeVariables": { "primaryColor": "#1a1020", "primaryTextColor": "#e8e0f0", "lineColor": "#ba53e6", "tertiaryColor": "#1f1630" } } }%%
flowchart LR
    subgraph input [You]
        A[Keyboard: shorthand]
    end
    subgraph typesymbol [TypeSymbol]
        B[Core engine]
        C[Platform adapter]
    end
    subgraph out [System]
        D[Unicode in the focused app]
    end
    A --> B
    B --> C
    C --> D
```

1. A **cross-platform Rust engine** parses and expands your math shorthand.  
2. A **background daemon** watches input so replacement can happen globally (not just inside one app).  
3. **macOS and Windows** each have a native adapter for capture and injection.

### Control surface (why the controls exist)

| Control | What it does | Why it matters |
| --- | --- | --- |
| `typesymbol on` / `typesymbol off` | Enable or pause global replacement instantly | You can safely switch contexts without uninstalling or editing config |
| `typesymbol daemon status` | Shows whether the background service is running | Quick health check when symbols are not appearing |
| `typesymbol config init` | Creates a local config with defaults | Gives you an explicit, editable baseline instead of hidden behavior |
| `typesymbol config show` | Prints the active config | Confirms which symbol families and triggers are currently active |
| `typesymbol test "..."` | Previews transforms without injecting into apps | Lets you validate rules before using them in live text fields |
| Trigger setting (`enter` / `ctrl-space`) | Chooses when replacement is applied | Balances speed vs control based on your typing style |
| Excluded apps list | Prevents replacement in selected apps | Avoids accidental transforms in terminals, editors, or sensitive inputs |

---

## Install

### Recommended (single path)

Install the official release build for your platform:

```bash
# macOS
brew install yazanmwk/homebrew-tap/typesymbol
```

```powershell
# Windows
irm https://raw.githubusercontent.com/yazanmwk/TypeSymbol/main/scripts/install-windows-release.ps1 | iex
```

Verify:

```bash
typesymbol test "alpha -> beta"
typesymbol daemon status
```

All alternative install methods (manual assets, from source, troubleshooting) are in [docs/install.md](docs/install.md).

---

## Quick start

```bash
# Preview a transform without the daemon
typesymbol test "alpha -> beta"

# Config
typesymbol config init
typesymbol config show

# Daemon
typesymbol daemon status
```

## Syntax types supported

TypeSymbol can replace shorthand across all of these symbol families:

- Greek aliases (`alpha`, `theta`, `pi`, `inf`)
- Operators (`->`, `<-`, `<->`, `!=`, `<=`, `>=`, `+-`)
- Superscripts and subscripts (`x^10`, `x_i`, `a_1`)
- Square roots (`sqrt(x)`, `sqrt x`)
- Integrals (strict + phrase forms, including bounds and inferred differential)
- Summations (`sum_(i=1)^n`, `sum from i=1 to n of ...`)
- Products (`product from i = 1 to n of ...`)
- Limits (`limit x to 0 of ...`, `lim (t->inf) ...`)
- Transform notation (Laplace/Fourier + inverse forms)
- Partial derivatives (`partial/partial x ...`)
- Quantifiers and set membership (`for all`, `exists`, `in`, `not in`)
- Set operators (`subseteq`, `union`, `intersection`)
- Probability/statistics (`probability of ...`, `expected value of ...`, `variance of ...`)
- Natural-language power phrase (`x power of 2` -> `x²`)

Complete syntax examples: [docs/syntax-guide.md](docs/syntax-guide.md)

---

## Why TypeSymbol

- **Keep flow state:** type plain shorthand and get math symbols without leaving your current app.
- **Work everywhere:** applies system-wide on macOS and Windows, not only inside one editor.
- **Stay in control:** explicit triggers, quick on/off, test mode, and per-app exclusions.
- **Trust the output:** deterministic rule-based transforms with config you can inspect and edit.

---

## Repository map

Only the main areas most contributors need:

```text
TypeSymbol/
├── crates/
│   ├── typesymbol-core/              # Parsing + transform engine
│   ├── typesymbol-config/            # Config model + defaults
│   ├── typesymbol-daemon/            # Runtime + input event pipeline
│   ├── typesymbol-platform-macos/    # Native macOS adapter
│   ├── typesymbol-platform-windows/  # Native Windows adapter
│   └── typesymbol-cli/               # CLI + TUI entrypoint
├── docs/                             # Install, syntax, release, security
├── scripts/                          # Install and packaging scripts
└── .github/workflows/                # CI and release automation
```

---

## Documentation

Index of all guides: **[docs/README.md](docs/README.md)**.

| Doc | What it’s for |
| --- | --- |
| [docs/install.md](docs/install.md) | Detailed install, PATH, and platform notes |
| [docs/releasing.md](docs/releasing.md) | Cutting a version and release artifacts |
| [docs/homebrew-tap.md](docs/homebrew-tap.md) | Homebrew tap |
| [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) | Build from source, tests, packaging overrides for forks |
| [docs/PRD.md](docs/PRD.md) | Product requirements (vision and goals) |
| [docs/SECURITY.md](docs/SECURITY.md) | Responsible disclosure |
| [LICENSE](LICENSE) | MIT License |

---

## Security

Do not post suspected vulnerabilities in public issues first. See **[docs/SECURITY.md](docs/SECURITY.md)** for how to report them responsibly.

---

## Let's Connect

[![GitHub](https://img.shields.io/badge/GitHub-TypeSymbol-111827?style=for-the-badge&logo=github&logoColor=white)](https://github.com/yazanmwk/TypeSymbol)
[![LinkedIn](https://img.shields.io/badge/LinkedIn-yazanmwk-0A66C2?style=for-the-badge&logo=linkedin&logoColor=white)](https://www.linkedin.com/in/yazanmwk/)
[![Email](https://img.shields.io/badge/Email-yazan.mw.k%40gmail.com-EA4335?style=for-the-badge&logo=gmail&logoColor=white)](mailto:yazan.mw.k@gmail.com)
[![Issues](https://img.shields.io/badge/Issues-Open%20Tracker-DC2626?style=for-the-badge&logo=githubissues&logoColor=white)](https://github.com/yazanmwk/TypeSymbol/issues)

---

<div align="center">

**[Releases](https://github.com/yazanmwk/TypeSymbol/releases)** · **[Contributing](docs/CONTRIBUTING.md)** · **[Security](docs/SECURITY.md)**

</div>
