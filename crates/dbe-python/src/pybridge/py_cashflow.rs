use dbe_cashflow as cashflow;
use pyo3::prelude::*;

/// Discount a single cashflow to present value.
///
/// Parameters
/// ----------
/// cashflow : float
///     The cashflow to be discounted.
/// discount_rate : float
///     The discount rate to apply to the cashflow.
/// timestep : float
///     The future time step at which the cashflow occurs.
///
/// Returns
/// -------
/// float
///     The present value of the cashflow.
#[pyfunction]
#[pyo3(name = "calculate_pv")]
#[pyo3(text_signature = "(cashflow, discount_rate, timestep, /)")]
pub fn py_cal_pv(cashflow_value: f64, discount_rate: f64, timestep: f64) -> PyResult<f64> {
    Ok(cashflow::cal_pv(cashflow_value, discount_rate, timestep))
}

/// Discount a sequence of cashflows to present value.
///
/// Parameters
/// ----------
/// cashflows : list[float]
///     A list of cashflows to be discounted.
/// discount_rate : float
///     The discount rate to apply to the cashflows.
///
/// Returns
/// -------
/// float
///     The present value of the cashflows.
#[pyfunction]
#[pyo3(name = "calculate_pv_from_cashflows")]
#[pyo3(text_signature = "(cashflows, discount_rate, /)")]
pub fn py_cal_pv_from_cf(cashflows: Vec<f64>, discount_rate: f64) -> PyResult<f64> {
    Ok(cashflow::cal_pv_from_cf(&cashflows, discount_rate))
}

/// Spread a target present value uniformly across future periods.
///
/// Parameters
/// ----------
/// t_pv : float
///     The target present value at the present time step.
/// t_steps : int
///     The number of time steps over which the cashflow will be distributed.
/// discount_rate : float
///     The discount rate to apply to the cashflow.
/// w_init : bool, optional
///     Whether to include step 0 in the calculation. Defaults to ``False``.
///
/// Returns
/// -------
/// float
///     The uniform nominal value to pay out at each time step.
#[pyfunction]
#[pyo3(name = "calculate_pv_unispread")]
#[pyo3(signature = (t_pv, t_steps, discount_rate, w_init = false))]
#[pyo3(text_signature = "(t_pv, t_steps, discount_rate, w_init=False, /)")]
pub fn py_pv_unispread(t_pv: f64, t_steps: i32, discount_rate: f64, w_init: bool) -> PyResult<f64> {
    Ok(cashflow::pv_unispread(t_pv, t_steps, discount_rate, w_init))
}
