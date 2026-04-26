# PRD: TypeSymbol

## 1. Product Overview

**Product name:** TypeSymbol

**Product category:** System-wide math text input utility

**Vision:**  
TypeSymbol lets users write plain-text math shorthand anywhere they can type, then converts it into clean mathematical symbols that look as close to LaTeX as possible while remaining portable across normal text fields. The product should work system-wide, similar in spirit to spellcheck or text replacement, but specifically for mathematical notation.

**Core value proposition:**  
Users should be able to type shorthand such as:

`integral0-infinity(x/x^2)`

and have TypeSymbol detect it as a convertible math expression, suggest a replacement, and convert it into a clean symbol-based output such as:

`∫₀^∞ x/x² dx`

The output must work in ordinary text contexts such as:
- Word processors
- Notes apps
- Browsers
- Chat apps
- Social media inputs
- Messaging fields
- Code comments
- Email clients

This is **not** a LaTeX renderer.  
This is a **system-wide math shorthand input engine** that outputs portable text, primarily using Unicode.

## 2. Product Goals

### Primary goals
1. Allow users to type math shorthand in plain text anywhere on the OS.
2. Detect shorthand patterns system-wide.
3. Offer conversion into math-symbol output that is portable across apps.
4. Be configurable entirely through a CLI and config file.
5. Keep core logic modular and maintainable for long-term evolution.

### Secondary goals
1. Feel similar to spellcheck/autocorrect in interaction quality.
2. Support both simple aliases and structured math expressions.
3. Be lightweight, fast, and unobtrusive.
4. Be safe and predictable, never destructively replacing text without user control.

## 3. Non-Goals

These are explicitly out of scope for v1:
- Full LaTeX rendering
- Arbitrary mathematical layout equal to TeX typesetting
- OCR
- Equation image generation
- Graphing
- Cloud sync
- Mobile keyboard support
- Linux support
- Deep IDE/plugin-specific integrations
- Full symbolic math evaluation
- Parsing every valid LaTeX command
- Exact native spellcheck API integration

## 4. Target Users

### Primary users
- Students who write math frequently
- Engineers
- STEM researchers
- Technical writers
- Developers writing math in plain text environments
- Users who want math symbols in apps that do not support LaTeX

### Secondary users
- Teachers
- Tutors
- Discord/Reddit/forum users
- Users posting equations in social captions or comments
- Productivity / CLI / power users

## 5. User Problem

Typing mathematical notation in ordinary text fields is frustrating because:
- LaTeX is unsupported in most places
- Unicode symbols are hard to remember and enter
- OS text replacement tools are too limited for structured math
- Copy-pasting equations is slow and breaks workflow
- Users want “type naturally, get nice math” everywhere

Users need a way to type math shorthand quickly and consistently across the whole system.

## 6. Core Product Concept

TypeSymbol runs as a background daemon and monitors typed text in editable text contexts. It keeps a rolling local buffer of recent text near the cursor and checks whether the user has typed a recognizable math shorthand pattern.

When it detects a valid pattern, it can:
- suggest a replacement
- wait for a trigger key
- replace the text with a Unicode math-symbol version
- optionally offer multiple candidate outputs

The product is configured through:
- a CLI
- a config file
- optional future settings UI

## 7. Core Principles

1. **System-wide first**  
   Must work in as many text input contexts as possible.

2. **Unicode-first output**  
   Output should prioritize portability over perfect typesetting.

3. **CLI-first configuration**  
   Advanced users must be able to configure rules, triggers, and behavior from terminal/config.

4. **Modular architecture**  
   Core logic must remain reusable across internal components.

5. **Safe interaction model**  
   Suggestions should be non-destructive and user-controlled.

6. **Low friction**  
   Fast, lightweight, and invisible until needed.

## 8. Functional Requirements

### 8.1 System-wide input monitoring
The app must:
- run in the background
- detect typed input in standard editable text fields
- track a rolling buffer of recent typed text near the insertion point
- ignore unsupported or inaccessible contexts gracefully

### 8.2 Math shorthand detection
The app must:
- identify predefined alias patterns
- identify operator replacements
- identify structured shorthand expressions
- distinguish probable math from ordinary text as much as possible

### 8.3 Suggestion and replacement flow
The app must support at least these modes:

#### Manual trigger mode
- User types shorthand
- User presses a configured hotkey/trigger
- TypeSymbol replaces the most recent valid expression

#### Suggest mode
- User types shorthand
- TypeSymbol detects a likely expression
- TypeSymbol presents a lightweight suggestion
- User accepts or dismisses it

#### Auto-replace mode
- Only for safe, low-risk patterns
- e.g. `->` to `→`, `!=` to `≠`
- Must be configurable and off by default for risky expressions

### 8.4 Output formatting
The app must output:
- portable Unicode symbols
- superscripts/subscripts where possible
- structured symbol expressions when feasible

Optional future output modes:
- LaTeX string export
- ASCII fallback
- pretty-preview-only mode

### 8.5 CLI configuration
The CLI must support:
- starting/stopping daemon
- testing expressions
- enabling/disabling features
- managing aliases/rules
- setting trigger keys
- toggling suggestion/auto-replace modes
- reading/writing config

### 8.6 Config file
The app must load config from a local file, likely TOML.

The config should support:
- trigger mode
- trigger keys
- enabled feature categories
- aliases
- operator rules
- formatting mode
- safety settings
- per-app exclusions in future

### 8.7 Logging/debugging
Developer/debug functionality should include:
- verbose mode
- dry-run/test mode
- rule match diagnostics
- parser diagnostics
- replacement failure logs

## 9. Supported Syntax for v1

V1 should intentionally support a constrained but useful syntax.

### 9.1 Aliases
Examples:
- `alpha` → `α`
- `beta` → `β`
- `gamma` → `γ`
- `theta` → `θ`
- `lambda` → `λ`
- `pi` → `π`
- `inf` or `infinity` → `∞`

### 9.2 Operators
Examples:
- `->` → `→`
- `<-` → `←`
- `<->` → `↔`
- `!=` → `≠`
- `<=` → `≤`
- `>=` → `≥`
- `+-` → `±`

### 9.3 Superscripts and subscripts
Examples:
- `x^2` → `x²`
- `x^n` → keep best-effort representation if direct Unicode unavailable
- `a_1` → `a₁`
- `x_i` → `xᵢ` when supported, otherwise fallback gracefully

### 9.4 Square roots
Examples:
- `sqrt(x)` → `√(x)`
- `sqrt x` → `√x` when unambiguous

### 9.5 Integrals
Examples:
- `int_0^inf f(x) dx` → `∫₀^∞ f(x) dx`
- `integral0-infinity(x/x^2)` → best normalized equivalent
- app may normalize missing spacing/parens internally

### 9.6 Summation
Examples:
- `sum_(i=1)^n i^2` → `∑ᵢ₌₁ⁿ i²`

### 9.7 Limits
Examples:
- `lim_(x->0) sin(x)/x` → `limₓ→0 sin(x)/x`

### 9.8 Basic grouping
Must support:
- parentheses
- nested parentheses to a reasonable depth
- best-effort parsing of grouped terms

## 10. Syntax Behavior Rules

1. Input syntax should be forgiving, not overly formal.
2. The parser should accept both compact and spaced forms where reasonable.
3. When Unicode cannot faithfully represent a structure, the formatter should degrade gracefully.
4. Output should prefer readability and portability over visual perfection.
5. The system must avoid converting plain English words unintentionally.

Example:  
Typing “alpha” in a mathematical context may suggest `α`, but the system should avoid excessive false positives in ordinary prose.

## 11. User Experience Requirements

### 11.1 Interaction model
The UX should feel similar to a text utility, not a full application.

The user should experience:
- low latency
- minimal UI
- no disruption to normal typing
- clear accept/reject behavior
- predictable replacements

### 11.2 Suggestion UI
For v1, suggestion UI can be minimal. Options:
- lightweight popup near cursor
- small inline overlay
- hotkey-only replacement without popup

A hotkey-only MVP is acceptable.

### 11.3 Safety
The app must:
- never replace text in password/secure fields
- fail safely
- avoid repeated replacement loops
- allow immediate undo via system undo when possible

## 12. Technical Architecture Requirements

### 12.1 Cross-platform architecture
The codebase must be designed so that:
- core parsing/logic is platform-agnostic
- OS-specific integrations are thin adapters
- platform-specific logic is isolated from the core

### 12.2 Recommended architecture
Use:
- **Rust** for the core engine
- Rust for CLI
- Rust for daemon/service logic
- thin OS-specific modules for:
  - input hooks
  - accessibility/input context
  - text replacement
  - suggestion window integration

Avoid a Swift-heavy architecture so the core remains portable and testable.

### 12.3 Layered architecture
The codebase should be divided into:

#### Core layer
Responsible for:
- tokenizer
- parser
- AST
- formatter
- rule engine
- config model
- suggestion logic

#### Daemon layer
Responsible for:
- background runtime
- event loop
- recent text buffer management
- calling core logic
- coordinating replacement actions

#### Platform adapter layer
Responsible for:
- reading typed events
- reading text context when possible
- replacing text in target app
- showing suggestion UI
- platform permissions

#### CLI layer
Responsible for:
- user configuration
- testing expressions
- daemon control
- diagnostics

## 13. Proposed Repository Structure

```text
typesymbol/
  crates/
    core/
    cli/
    daemon/
    config/
    platform/
    platform-macos/
```

Alternative acceptable structure:

```text
typesymbol-core
typesymbol-cli
typesymbol-daemon
typesymbol-platform-macos
```

## 14. CLI Requirements

The CLI should support commands such as:

```bash
typesymbol daemon start
typesymbol daemon stop
typesymbol test "alpha + beta -> gamma"
typesymbol config show
typesymbol config set trigger ctrl-space
typesymbol rule add "alpha" "α"
typesymbol rule remove "alpha"
typesymbol features enable integrals
typesymbol features disable limits
```

### Required CLI features
- test an expression
- inspect parse/output
- start/stop/status daemon
- add/remove/list aliases
- toggle features
- set trigger mode
- set output mode
- export/import config
- verbose debug mode

## 15. Config File Requirements

Use TOML unless there is a strong reason otherwise.

Example:

```toml
mode = "unicode"
trigger_mode = "manual"
trigger_key = "ctrl-space"
live_suggestions = false
auto_replace_safe_rules = true

[features]
greek_letters = true
operators = true
superscripts = true
subscripts = true
sqrt = true
integrals = true
summations = true
limits = true

[aliases]
alpha = "α"
beta = "β"
gamma = "γ"
theta = "θ"
pi = "π"
infinity = "∞"

[operators]
"->" = "→"
"<-" = "←"
"<->" = "↔"
"!=" = "≠"
"<=" = "≤"
">=" = "≥"
```

## 16. Platform Support Plan

### v1 target platform
- macOS only

### v1 macOS requirements
- background daemon process
- permission handling for accessibility/input monitoring as needed
- reliable replacement in common text fields
- graceful degradation in unsupported contexts

## 17. MVP Scope

The MVP should include:

### Must-have
- background daemon
- CLI
- config file
- alias replacements
- operator replacements
- superscript/subscript basics
- sqrt support
- basic integral and summation parsing
- manual trigger replacement flow
- macOS support

### Nice-to-have
- suggestion popup
- live suggestion mode
- multiple output candidates
- app-specific exclusions
- undo-aware replacement

### Excluded from MVP
- settings GUI
- deep native spellcheck-style underline
- full advanced equation parser
- mobile support

## 18. Example User Flows

### Flow 1: simple symbol alias
1. User types `alpha + beta`
2. User presses trigger key
3. TypeSymbol replaces with `α + β`

### Flow 2: operator replacement
1. User types `x != y`
2. Safe auto-replace enabled
3. TypeSymbol converts to `x ≠ y`

### Flow 3: structured math shorthand
1. User types `int_0^inf x^2 dx`
2. User presses trigger key
3. TypeSymbol parses and replaces with `∫₀^∞ x² dx`

### Flow 4: CLI test mode
1. User runs:
   ```bash
   typesymbol test "sum_(i=1)^n i^2"
   ```
2. CLI outputs:
   ```text
   ∑ᵢ₌₁ⁿ i²
   ```

## 19. Acceptance Criteria

The product is successful for MVP if:

1. User can install and run a daemon on macOS.
2. User can configure behavior through CLI and TOML config.
3. User can type supported shorthand into common text fields and convert it with a trigger key.
4. Conversion works reliably for the supported v1 syntax categories.
5. Core parsing/formatting logic is clearly separated from platform-specific code.
6. Codebase is structured so future expansion can happen without rewriting parser/config/CLI/core logic.

## 20. Engineering Constraints

1. Core logic must not depend on macOS-specific frameworks.
2. Platform code must be isolated behind clear interfaces/traits.
3. The daemon must be lightweight and low-latency.
4. Replacement logic must be robust against partial parse failures.
5. All risky replacements must remain user-controlled by default.
6. The system must degrade gracefully when a target app cannot be manipulated safely.

## 21. Open Questions for Implementation

These should be resolved during design/engineering:
1. Exact trigger-key default
2. How much context to buffer
3. Whether to support inline popup in MVP or hotkey only
4. How to determine “math-like context” vs ordinary prose
5. Best fallback behavior for unsupported superscripts/subscripts
6. Whether to preserve original text in a suggestion candidate menu
7. App blacklist/whitelist strategy
8. Best undo integration approach
9. Whether structured shorthand should normalize missing `dx` in certain integral cases or require explicit input

## 22. Suggested Delivery Order

### Phase 1
- parser prototype
- formatter prototype
- CLI `test` command
- config loading

### Phase 2
- daemon runtime
- rolling input buffer
- hotkey-triggered replacement
- basic macOS adapter

### Phase 3
- structured expression support
- more rules
- safety improvements
- logs and diagnostics

### Phase 4
- optional suggestion UI
- app exclusions
- performance polish

## 23. Summary

TypeSymbol is a system-wide, CLI-configured math shorthand input engine that allows users to type plain-text expressions anywhere and convert them into portable Unicode math notation. The MVP should focus on macOS, hotkey-triggered replacement, and a Rust-first modular architecture.
