from pydbe._core import rs_cal_pv, rs_cal_pv_from_cf, rs_pv_unispread


def calculate_pv(cashflow: float, discount_rate: float, timestep: float) -> float:
    """
    Calculate the present value of cashflow.

    Parameters
    ----------
    cashflow : float
        The cashflow to be discounted.
    discount_rate : float
        The discount rate to be applied to the cashflow.
    timestep : float
        The future time step at which the cashflow occurs.

    Returns
    -------
    float
        The present value of the cashflow.
    """
    return rs_cal_pv(cashflow, discount_rate, timestep)


def calculate_pv_from_cashflows(cashflows: list[float], discount_rate: float) -> float:
    """
    Calculate the present value of a series of cashflows.

    Parameters
    ----------
    cashflows : list[float]
        A list of cashflows to be discounted.
    discount_rate : float
        The discount rate to be applied to the cashflows.

    Returns
    -------
    float
        The present value of the cashflows.
    """
    return rs_cal_pv_from_cf(cashflows, discount_rate)


def calculate_pv_unispread(
    t_pv: float, t_steps: int, discount_rate: float, w_init: bool = False
) -> float:
    """
    Calculate the uniform nominal value that will need to be pay out at each time step
    to achieve the targeted present value at the present time step.

    Parameters
    ----------
    t_pv : float
        The targeted present value at the present time step.
    t_steps : int
        The number of time steps over which the cashflow will be distributed.
    rate : float
        The discount rate to be applied to the cashflow.
    w_init : bool, optional
        Whether to include step 0 in the calculation. The default is False.

    Returns
    -------
    float
        The uniform nominal value that will need to be pay out at each time step to
        achieve the targeted present value at the present time step.
    """
    return rs_pv_unispread(t_pv, t_steps, discount_rate, w_init)
