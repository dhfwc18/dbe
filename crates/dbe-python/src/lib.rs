use pyo3::prelude::*;

pub mod pybridge;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pybridge::py_cashflow::py_cal_pv, m)?)?;
    m.add_function(wrap_pyfunction!(
        pybridge::py_cashflow::py_cal_pv_from_cf,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(pybridge::py_cashflow::py_pv_unispread, m)?)?;
    m.add_class::<pybridge::py_consumer::PyPreference>()?;
    m.add_function(wrap_pyfunction!(
        pybridge::py_consumer::py_optimal_bundle,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(pybridge::py_consumer::py_trace_2d, m)?)?;
    m.add_function(wrap_pyfunction!(
        pybridge::py_consumer::py_get_standard_config,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        pybridge::py_consumer::py_set_standard_config,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        pybridge::py_consumer::py_restore_standard_config,
        m
    )?)?;
    Ok(())
}
