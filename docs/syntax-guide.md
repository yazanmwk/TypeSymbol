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
- `sum from n=0 to inf of n^2` -> `∑ₙ₌₀^∞ n²`
- `summation from i = 1 to n of i` -> `∑ᵢ₌₁ⁿ i`

## Product

- `product from i = 1 to n of i` -> `∏ᵢ₌₁ⁿ i`
- `prod from k = 0 to inf of a_k` -> `∏_{k}₌₀^∞ aₖ`

## Limits

- `limit x to 0 of sin(x)/x` -> `lim_{x}→0 sin(x)/x`
- `lim (t->inf) e^(-t)` -> `lim_{t}→∞ e^(-t)`

## Transform notation

- `laplace of f(t)` -> `ℒ{f(t)}`
- `inverse laplace of F(s)` -> `ℒ⁻¹{F(s)}`
- `fourier transform of f(t)` -> `ℱ{f(t)}`
- `inv fourier of X(w)` -> `ℱ⁻¹{X(w)}`

## Partial derivatives

- `partial/partial x f(x,y)` -> `∂/∂x f(x,y)`

## Logic and set notation

- `for all x in A` -> `∀ x ∈ A`
- `there exists y not in B` -> `∃ y ∉ B`
- `A subseteq B` -> `A ⊆ B`
- `A union B` -> `A ∪ B`
- `A intersection B` -> `A ∩ B`

## Probability and statistics

- `probability of A|B` -> `P(A|B)`
- `expected value of X` -> `E[X]`
- `variance of X` -> `Var(X)`

## Power phrase normalization

- `x power of 3x` -> `x³ˣ`
- `y power of 10` -> `y¹⁰`

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
