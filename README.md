# DBE

[![crates.io: dbe-cashflow](https://img.shields.io/crates/v/dbe-cashflow.svg)](https://crates.io/crates/dbe-cashflow)
[![crates.io: dbe-ct](https://img.shields.io/crates/v/dbe-ct.svg)](https://crates.io/crates/dbe-ct)
[![PyPI: dbe](https://img.shields.io/pypi/v/dbe.svg)](https://pypi.org/project/dbe/)

This library is a Rust crate that provides a collection of core economic
functionalities, with optional Python bindings exposed through a native extension.

The Python package now lives under `pydbe/` with a `src` layout and its own test
suite.

From `pydbe/`, run:

```
uv sync
```

That installs the package and builds the Rust extension through the configured
build backend.

To run the Python tests from `pydbe/`:

```
uv run pytest
```
