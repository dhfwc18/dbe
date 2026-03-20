"""Cashflow discounting utilities.

Functions
---------
calculate_pv
    Discount a single cashflow to present value.
calculate_pv_from_cashflows
    Discount a sequence of cashflows to present value.
calculate_pv_unispread
    Spread a target present value uniformly across future periods.
"""

from dbe._core import (
    calculate_pv as _calculate_pv,
)
from dbe._core import (
    calculate_pv_from_cashflows as _calculate_pv_from_cashflows,
)
from dbe._core import (
    calculate_pv_unispread as _calculate_pv_unispread,
)


def calculate_pv(cashflow: float, discount_rate: float, timestep: float) -> float:
    return _calculate_pv(cashflow, discount_rate, timestep)


def calculate_pv_from_cashflows(cashflows: list[float], discount_rate: float) -> float:
    return _calculate_pv_from_cashflows(cashflows, discount_rate)


def calculate_pv_unispread(
    t_pv: float,
    t_steps: int,
    discount_rate: float,
    w_init: bool = False,
) -> float:
    return _calculate_pv_unispread(t_pv, t_steps, discount_rate, w_init)


calculate_pv.__doc__ = _calculate_pv.__doc__
calculate_pv_from_cashflows.__doc__ = _calculate_pv_from_cashflows.__doc__
calculate_pv_unispread.__doc__ = _calculate_pv_unispread.__doc__

__all__ = [
    "calculate_pv",
    "calculate_pv_from_cashflows",
    "calculate_pv_unispread",
]
