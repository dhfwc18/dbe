use pyo3::prelude::*;
pub mod econ;
pub mod pybridge;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pybridge::py_cashflow::py_cal_pv, m)?)?;
    m.add_function(wrap_pyfunction!(
        pybridge::py_cashflow::py_cal_pv_from_cf,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(pybridge::py_cashflow::py_pv_unispread, m)?)?;
    Ok(())
}
