# dbe Python Package

Python bindings for the `dbe` basic economics library.

The package exposes:

- present-value utilities for discounted-cashflow analysis
- consumer-theory tools for preferences, indifference curves, and constrained bundle 
  optimisation

The native extension is built with PyO3 and maturin and is packaged under `dbe._core`.
