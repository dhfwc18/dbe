use super::super::econ::cashflow;
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(name = "rs_cal_pv")]
pub fn py_cal_pv(val: f64, rate: f64, step: f64) -> PyResult<f64> {
    Ok(cashflow::cal_pv(val, rate, step))
}

#[pyfunction]
#[pyo3(name = "rs_cal_pv_from_cf")]
pub fn py_cal_pv_from_cf(cf: Vec<f64>, rate: f64) -> PyResult<f64> {
    Ok(cashflow::cal_pv_from_cf(&cf, rate))
}

#[pyfunction]
#[pyo3(name = "rs_pv_unispread")]
pub fn py_pv_unispread(t_pv: f64, t_steps: i32, rate: f64, w_init: bool) -> PyResult<f64> {
    Ok(cashflow::pv_unispread(t_pv, t_steps, rate, w_init))
}
