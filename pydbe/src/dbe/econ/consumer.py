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
    class Preference:
        @classmethod
        def set(cls, **kwargs: object) -> type["Config.Preference"]:
            set_standard_config(preference=kwargs)
            return cls

        @classmethod
        def state(cls) -> dict[str, object]:
            return dict(get_standard_config()["preference"])

        @classmethod
        def restore_defaults(cls) -> type["Config.Preference"]:
            defaults = Config.defaults()["preference"]
            set_standard_config(preference=defaults)
            return cls

    class Indifference:
        @classmethod
        def set(cls, **kwargs: object) -> type["Config.Indifference"]:
            set_standard_config(indifference=kwargs)
            return cls

        @classmethod
        def state(cls) -> dict[str, object]:
            return dict(get_standard_config()["indifference"])

        @classmethod
        def restore_defaults(cls) -> type["Config.Indifference"]:
            defaults = Config.defaults()["indifference"]
            set_standard_config(indifference=defaults)
            return cls

    class Optimization:
        @classmethod
        def set(cls, **kwargs: object) -> type["Config.Optimization"]:
            set_standard_config(optimization=kwargs)
            return cls

        @classmethod
        def state(cls) -> dict[str, object]:
            return dict(get_standard_config()["optimization"])

        @classmethod
        def restore_defaults(cls) -> type["Config.Optimization"]:
            defaults = Config.defaults()["optimization"]
            set_standard_config(optimization=defaults)
            return cls

    @classmethod
    def set(cls, **kwargs: object) -> type["Config"]:
        set_standard_config(**kwargs)
        return cls

    @classmethod
    def state(cls) -> dict[str, object]:
        return dict(get_standard_config())

    @classmethod
    def defaults(cls) -> dict[str, object]:
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
            "optimization": {
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


__all__ = [
    "Config",
    "Preference",
    "get_standard_config",
    "optimal_bundle",
    "restore_standard_config",
    "set_standard_config",
    "trace_indifference_curve",
]
