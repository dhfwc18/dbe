"""Consumer-theory helpers and configuration controls.

Classes
-------
Preference
    Consumer preference wrapper backed by the native core.
Config
    Shared default configuration for preference, indifference, and
    optimisation components.

Functions
---------
optimal_bundle
    Solve for the utility-maximising bundle under a budget constraint.
trace_indifference_curve
    Trace an indifference curve between two goods.
"""

from __future__ import annotations

from dbe._core import (
    Preference,
    get_standard_config,
    restore_standard_config,
    set_standard_config,
    trace_indifference_curve,
)
from dbe._core import (
    optimal_bundle as _optimal_bundle,
)


class Config:
    """Shared consumer-theory defaults grouped by component."""

    class Preference:
        """Preference-construction and numerical defaults."""

        @classmethod
        def set(cls, **kwargs: object) -> type["Config.Preference"]:
            """Update shared preference defaults.

            Parameters
            ----------
            **kwargs
                Preference configuration fields to override.
            """
            set_standard_config(preference=kwargs)
            return cls

        @classmethod
        def state(cls) -> dict[str, object]:
            """Return the current preference default state."""
            return dict(get_standard_config()["preference"])

        @classmethod
        def restore_defaults(cls) -> type["Config.Preference"]:
            """Restore preference defaults to the package defaults."""
            defaults = Config.defaults()["preference"]
            set_standard_config(preference=defaults)
            return cls

    class Indifference:
        """Indifference-curve tracing defaults."""

        @classmethod
        def set(cls, **kwargs: object) -> type["Config.Indifference"]:
            """Update shared indifference-tracing defaults.

            Parameters
            ----------
            **kwargs
                Indifference configuration fields to override.
            """
            set_standard_config(indifference=kwargs)
            return cls

        @classmethod
        def state(cls) -> dict[str, object]:
            """Return the current indifference default state."""
            return dict(get_standard_config()["indifference"])

        @classmethod
        def restore_defaults(cls) -> type["Config.Indifference"]:
            """Restore indifference defaults to the package defaults."""
            defaults = Config.defaults()["indifference"]
            set_standard_config(indifference=defaults)
            return cls

    class Optimisation:
        """Bundle-optimisation defaults."""

        @classmethod
        def set(cls, **kwargs: object) -> type["Config.Optimisation"]:
            """Update shared optimisation defaults.

            Parameters
            ----------
            **kwargs
                Optimisation configuration fields to override.
            """
            set_standard_config(optimisation=kwargs)
            return cls

        @classmethod
        def state(cls) -> dict[str, object]:
            """Return the current optimisation default state."""
            return dict(get_standard_config()["optimisation"])

        @classmethod
        def restore_defaults(cls) -> type["Config.Optimisation"]:
            """Restore optimisation defaults to the package defaults."""
            defaults = Config.defaults()["optimisation"]
            set_standard_config(optimisation=defaults)
            return cls

    @classmethod
    def set(cls, **kwargs: object) -> type["Config"]:
        """Update grouped shared defaults.

        Parameters
        ----------
        **kwargs
            Nested groups keyed by ``preference``, ``indifference``, or
            ``optimisation``.
        """
        set_standard_config(**kwargs)
        return cls

    @classmethod
    def state(cls) -> dict[str, object]:
        """Return the full nested configuration state."""
        return dict(get_standard_config())

    @classmethod
    def defaults(cls) -> dict[str, object]:
        """Return the package default configuration state."""
        defaults = {
            "preference": {
                "samples": 60000,
                "seed": 0,
                "strict_monotonicity": False,
                "strict_convexity": False,
                "epsilon": 1e-6,
                "tolerance": 1e-9,
                "continuity_threshold": 1.0,
                "calc_epsilon": 1e-7,
                "calc_tolerance": 1e-9,
            },
            "indifference": {
                "n_points": 200,
                "tol": 1e-10,
            },
            "optimisation": {
                "mu_init": 1.0,
                "mu_decay": 0.1,
                "outer_iters": 10,
                "inner_iters": 500,
                "step_size": 1e-2,
                "tol": 1e-8,
            },
        }
        return defaults

    @classmethod
    def restore_defaults(cls) -> type["Config"]:
        """Restore all shared defaults to the package defaults."""
        restore_standard_config()
        return cls


def optimal_bundle(
    pref: Preference,
    prices: list[float],
    income: float,
    **kwargs: object,
) -> list[float]:
    if kwargs:
        return _optimal_bundle(pref, prices, income, kwargs)
    return _optimal_bundle(pref, prices, income)


optimal_bundle.__doc__ = _optimal_bundle.__doc__


__all__ = [
    "Config",
    "Preference",
    "get_standard_config",
    "optimal_bundle",
    "restore_standard_config",
    "set_standard_config",
    "trace_indifference_curve",
]
