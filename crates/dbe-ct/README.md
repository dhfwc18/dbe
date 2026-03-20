# Consumer Theory Crate

Provides core tools for consumer-theory analysis over bounded consumption bundles.

## Overview

The crate is organised around three main tasks:

- validating and evaluating consumer preferences
- tracing indifference curves in two dimensions
- solving for utility-maximising bundles under a budget constraint

These capabilities are exposed through the `pref`, `indifference`, and `marshallian`
modules.

## Preference Modelling

The `pref` module defines `Preference`, which represents a utility function over a
bounded bundle of goods.

A preference instance supports:

- utility evaluation with `get_utility`
- marginal utility estimation with `get_mu`
- marginal rate of substitution evaluation with `get_mrs`
- optional validation against continuity, monotonicity, and convexity checks at
  construction time

Validation uses deterministic boundary points together with Sobol-sequence sampling to
test the supplied utility function over the admissible region.

## Indifference Curve Tracing

The `indifference` module provides `trace_2d`, which traces an indifference curve for a
target utility level in the plane of two selected goods.

The procedure:

- fixes all non-plotted goods at the midpoint of their bounds
- grids one selected good across its bounds
- uses bisection to solve for the second selected good

This is useful for inspection, plotting, and sanity-checking preference behaviour in
two-good slices of a higher-dimensional problem.

## Marshallian Bundle Optimisation

The `marshallian` module provides `optimal_bundle`, which solves for the
utility-maximising bundle subject to a linear budget constraint.

The optimisation uses an interior-point log-barrier method with:

- a strictly interior starting point
- numerical marginal utility estimation
- backtracking line search
- configurable outer and inner iteration limits

This produces a constrained optimum for problems of the form `p * x <= income`.

## Shared Standard Configuration

The crate maintains a shared standard configuration that supplies default settings for:

- preference validation and numerical evaluation
- indifference-curve tracing
- bundle optimisation

These defaults are grouped under `StandardConfig` as:

- `preference`
- `indifference`
- `optimisation`

This allows callers to adjust standard behaviour centrally while keeping per-call
configuration available where needed.

## Main Types

- `Preference` in `pref`
- `ValidationConfig` in `pref`
- `StandardConfig` in `pref`
- `IndiffConfig` in `indifference`
- `BudgetConstraint` in `marshallian`
- `OptimConfig` in `marshallian`

## Typical Use Cases

- check whether a utility specification behaves like a rational consumer preference
- compute marginal utility and marginal rates of substitution at a bundle
- trace indifference curves for visual analysis
- solve for Marshallian demand under prices and income
