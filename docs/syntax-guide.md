# TypeSymbol Syntax Guide (MVP)

This guide covers the shorthand formats currently supported by TypeSymbol, including flexible forms.

## Trigger behavior

- Default trigger: `enter`
- Alternative trigger: `ctrl-space`

Set via config:

```toml
trigger_key = "enter"
```

## Greek aliases

- `alpha` -> `α`
- `beta` -> `β`
- `gamma` -> `γ`
- `theta` -> `θ`
- `lambda` -> `λ`
- `pi` -> `π`
- `inf`, `infinity` -> `∞`

## Operators

- `->` -> `→`
- `<-` -> `←`
- `<->` -> `↔`
- `!=` -> `≠`
- `<=` -> `≤`
- `>=` -> `≥`
- `+-` -> `±`

## Superscripts/subscripts

- `x^2` -> `x²`
- `x^10` -> `x¹⁰`
- `a_1` -> `a₁`
- `x_i` -> `xᵢ`

## Square roots

- `sqrt(x)` -> `√(x)`
- `sqrt x` -> `√x`

## Summation

- `sum_(i=1)^n i^2` -> `∑ᵢ₌₁ⁿ i²`

## Integrals (strict forms)

- `int_0^inf x^2 dx` -> `∫₀^∞ x² dx`
- `integral0-infinity(x/x^2)` -> `∫₀^∞ x/x² dx`

## Integrals (flexible forms)

These phrases are normalized and interpreted:

- `integral from 0 to infinity of x over x^2` -> `∫₀^∞ x/x² dx`
- `int 0 -> inf x/x^2` -> `∫₀^∞ x/x² dx`

Normalization examples:

- `over` becomes `/`
- `to infinity` becomes `to inf`

## Current limitations

- Parsing is still rule-based, not full natural language understanding.
- Some phrases may require slight rewrites into supported forms.
- `dx` is currently auto-appended for flexible integral patterns.
