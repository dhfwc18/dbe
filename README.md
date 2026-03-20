# DBE

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
