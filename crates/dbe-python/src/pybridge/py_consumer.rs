use dbe_ct::{
    indifference::{self, IndiffConfig},
    marshallian::{self, BudgetConstraint, OptimConfig},
    pref::{FalliblePreference, PreferenceError, StandardConfig},
};
use pyo3::exceptions::{PyIndexError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn set_nested_bool(dict: &Bound<'_, PyDict>, key: &str, current: &mut bool) -> PyResult<()> {
    if let Some(value) = dict.get_item(key)? {
        *current = value.extract()?;
    }
    Ok(())
}

fn set_nested_u32(dict: &Bound<'_, PyDict>, key: &str, current: &mut u32) -> PyResult<()> {
    if let Some(value) = dict.get_item(key)? {
        *current = value.extract()?;
    }
    Ok(())
}

fn set_nested_usize(dict: &Bound<'_, PyDict>, key: &str, current: &mut usize) -> PyResult<()> {
    if let Some(value) = dict.get_item(key)? {
        *current = value.extract()?;
    }
    Ok(())
}

fn set_nested_f64(dict: &Bound<'_, PyDict>, key: &str, current: &mut f64) -> PyResult<()> {
    if let Some(value) = dict.get_item(key)? {
        *current = value.extract()?;
    }
    Ok(())
}

fn validate_bundle(bundle: &[f64], dims: usize) -> PyResult<()> {
    if bundle.len() != dims {
        return Err(PyErr::new::<PyValueError, _>(format!(
            "bundle length {} does not match the number of goods {}",
            bundle.len(),
            dims
        )));
    }
    Ok(())
}

fn validate_good_index(good: usize, dims: usize, name: &str) -> PyResult<()> {
    if good >= dims {
        return Err(PyErr::new::<PyIndexError, _>(format!(
            "{name} index {good} is out of bounds for {dims} goods"
        )));
    }
    Ok(())
}

fn python_utility_eval(func: &Py<PyAny>, bundle: &[f64]) -> PyResult<f64> {
    match Python::try_attach(|py| func.call1(py, (bundle.to_vec(),))?.extract::<f64>(py)) {
        Some(result) => result,
        None => Err(PyErr::new::<PyRuntimeError, _>(
            "failed to attach Python interpreter for utility callback",
        )),
    }
}

type PyUtilityFn = Box<dyn Fn(&[f64]) -> PyResult<f64>>;
type PyBridgePreference = FalliblePreference<PyUtilityFn, PyErr>;

fn build_bridge_preference(
    py: Python<'_>,
    pref: &PyPreference,
    validate: bool,
) -> PyResult<PyBridgePreference> {
    let func = pref.utility_func.clone_ref(py);
    FalliblePreference::with_validation(
        Box::new(move |bundle: &[f64]| python_utility_eval(&func, bundle)) as PyUtilityFn,
        pref.min_bounds.clone(),
        pref.max_bounds.clone(),
        validate,
    )
    .map_err(map_preference_error)
}

fn map_preference_error(error: PreferenceError<PyErr>) -> PyErr {
    match error {
        PreferenceError::Config(message) => PyErr::new::<PyValueError, _>(message),
        PreferenceError::Eval(err) => err,
    }
}

/// Represents a consumer's preference over bundles of goods.
///
/// Accepts a Python callable as the utility function. Validation of the
/// axioms of rationality is run on construction unless `validate=False`.
#[pyclass(name = "Preference")]
pub struct PyPreference {
    pub(crate) utility_func: Py<PyAny>,
    pub(crate) min_bounds: Vec<f64>,
    pub(crate) max_bounds: Vec<f64>,
}

#[pymethods]
impl PyPreference {
    /// Create a validated consumer preference.
    #[new]
    #[pyo3(signature = (
        utility_func,
        min_bounds,
        max_bounds,
        *,
        validate = true,
    ))]
    pub fn new(
        py: Python<'_>,
        utility_func: Py<PyAny>,
        min_bounds: Vec<f64>,
        max_bounds: Vec<f64>,
        validate: bool,
    ) -> PyResult<Self> {
        let probe = utility_func.clone_ref(py);
        FalliblePreference::with_validation(
            move |bundle: &[f64]| python_utility_eval(&probe, bundle),
            min_bounds.clone(),
            max_bounds.clone(),
            validate,
        )
        .map_err(map_preference_error)?;

        Ok(PyPreference {
            utility_func,
            min_bounds,
            max_bounds,
        })
    }

    /// Evaluate the utility function at the given bundle.
    fn get_utility(&self, py: Python<'_>, bundle: Vec<f64>) -> PyResult<f64> {
        validate_bundle(&bundle, self.min_bounds.len())?;
        self.utility_func.call1(py, (bundle,))?.extract(py)
    }

    /// Compute marginal utility for a specific good via central differences.
    fn get_mu(&self, py: Python<'_>, bundle: Vec<f64>, good: usize) -> PyResult<f64> {
        validate_bundle(&bundle, self.min_bounds.len())?;
        validate_good_index(good, self.min_bounds.len(), "good")?;
        let ep = StandardConfig::get().preference.calc_epsilon;
        let mut inc = bundle.clone();
        let mut dec = bundle;
        inc[good] += ep;
        dec[good] -= ep;
        let u_inc: f64 = self.utility_func.call1(py, (inc,))?.extract(py)?;
        let u_dec: f64 = self.utility_func.call1(py, (dec,))?.extract(py)?;
        Ok((u_inc - u_dec) / (2.0 * ep))
    }

    /// Compute the marginal rate of substitution between good_i and good_j.
    fn get_mrs(
        &self,
        py: Python<'_>,
        bundle: Vec<f64>,
        good_i: usize,
        good_j: usize,
    ) -> PyResult<f64> {
        validate_bundle(&bundle, self.min_bounds.len())?;
        validate_good_index(good_i, self.min_bounds.len(), "good_i")?;
        validate_good_index(good_j, self.min_bounds.len(), "good_j")?;
        let mu_j = self.get_mu(py, bundle.clone(), good_j)?;
        if mu_j.abs() < StandardConfig::get().preference.calc_tolerance {
            return Err(PyErr::new::<PyValueError, _>(format!(
                "MRS undefined: MU of good {} is zero at {:?}",
                good_j, bundle
            )));
        }
        Ok(self.get_mu(py, bundle, good_i)? / mu_j)
    }
}

/// Find the consumption bundle that maximises utility subject to a
/// budget constraint.
///
/// Uses an interior-point log-barrier method with backtracking line search.
///
/// # Arguments
/// * `pref` - A `PyPreference` whose rationality axioms are already enforced
/// * `prices` - Price vector, must match the number of goods in `pref`
/// * `income` - Total income available to the consumer
#[pyfunction]
#[pyo3(name = "optimal_bundle")]
#[pyo3(signature = (pref, prices, income, options = None))]
#[pyo3(text_signature = "(pref, prices, income, options=None)")]
pub fn py_optimal_bundle(
    py: Python<'_>,
    pref: &PyPreference,
    prices: Vec<f64>,
    income: f64,
    options: Option<Bound<'_, PyDict>>,
) -> PyResult<Vec<f64>> {
    let rust_pref = build_bridge_preference(py, pref, false)?;
    let mut standard = StandardConfig::get();
    let constraint = BudgetConstraint { prices, income };

    if let Some(dict) = options {
        set_nested_f64(&dict, "mu_init", &mut standard.optimization.optim_mu_init)?;
        set_nested_f64(&dict, "mu_decay", &mut standard.optimization.optim_mu_decay)?;
        set_nested_usize(
            &dict,
            "outer_iters",
            &mut standard.optimization.optim_outer_iters,
        )?;
        set_nested_usize(
            &dict,
            "inner_iters",
            &mut standard.optimization.optim_inner_iters,
        )?;
        set_nested_f64(
            &dict,
            "step_size",
            &mut standard.optimization.optim_step_size,
        )?;
        set_nested_f64(&dict, "tol", &mut standard.optimization.optim_tol)?;
    }

    let optim_config = OptimConfig {
        mu_init: standard.optimization.optim_mu_init,
        mu_decay: standard.optimization.optim_mu_decay,
        outer_iters: standard.optimization.optim_outer_iters,
        inner_iters: standard.optimization.optim_inner_iters,
        step_size: standard.optimization.optim_step_size,
        tol: standard.optimization.optim_tol,
    };

    marshallian::optimal_bundle_fallible(&rust_pref, &constraint, optim_config)
        .map_err(map_preference_error)
}

/// Trace an indifference curve in the plane of two goods.
///
/// Grids `good_i` across its bounds and uses bisection to find the value of
/// `good_j` such that U(x) = `target_utility`. All other goods are held
/// fixed at the midpoint of their bounds.
///
/// # Arguments
/// * `pref` - A `PyPreference` whose rationality axioms are already enforced
/// * `target_utility` - The utility level U* to trace
/// * `good_i` - Index of the good on the x-axis (gridded)
/// * `good_j` - Index of the good on the y-axis (solved via bisection)
#[pyfunction]
#[pyo3(name = "trace_indifference_curve")]
#[pyo3(signature = (pref, target_utility, good_i, good_j, *, n_points = None, tol = None))]
#[pyo3(text_signature = "(pref, target_utility, good_i, good_j, *, n_points=None, tol=None)")]
pub fn py_trace_2d(
    py: Python<'_>,
    pref: &PyPreference,
    target_utility: f64,
    good_i: usize,
    good_j: usize,
    n_points: Option<usize>,
    tol: Option<f64>,
) -> PyResult<Vec<(f64, f64)>> {
    let rust_pref = build_bridge_preference(py, pref, false)?;
    let standard = StandardConfig::get();
    let indiff_config = IndiffConfig {
        n_points: n_points.unwrap_or(standard.indifference.indiff_n_points),
        tol: tol.unwrap_or(standard.indifference.indiff_tol),
    };
    indifference::trace_2d_fallible(&rust_pref, target_utility, good_i, good_j, indiff_config)
        .map_err(map_preference_error)
}

/// Get the shared standard consumer-theory configuration.
#[pyfunction]
#[pyo3(name = "get_standard_config")]
pub fn py_get_standard_config(py: Python<'_>) -> PyResult<Py<PyDict>> {
    let config = StandardConfig::get();
    let out = PyDict::new(py);
    let preference = PyDict::new(py);
    preference.set_item("samples", config.preference.samples)?;
    preference.set_item("seed", config.preference.seed)?;
    preference.set_item("strict_monotonicity", config.preference.strict_monotonicity)?;
    preference.set_item("strict_convexity", config.preference.strict_convexity)?;
    preference.set_item("epsilon", config.preference.epsilon)?;
    preference.set_item("tolerance", config.preference.tolerance)?;
    preference.set_item(
        "continuity_threshold",
        config.preference.continuity_threshold,
    )?;
    preference.set_item("calc_epsilon", config.preference.calc_epsilon)?;
    preference.set_item("calc_tolerance", config.preference.calc_tolerance)?;
    out.set_item("preference", preference)?;

    let indifference = PyDict::new(py);
    indifference.set_item("n_points", config.indifference.indiff_n_points)?;
    indifference.set_item("tol", config.indifference.indiff_tol)?;
    out.set_item("indifference", indifference)?;

    let optimization = PyDict::new(py);
    optimization.set_item("mu_init", config.optimization.optim_mu_init)?;
    optimization.set_item("mu_decay", config.optimization.optim_mu_decay)?;
    optimization.set_item("outer_iters", config.optimization.optim_outer_iters)?;
    optimization.set_item("inner_iters", config.optimization.optim_inner_iters)?;
    optimization.set_item("step_size", config.optimization.optim_step_size)?;
    optimization.set_item("tol", config.optimization.optim_tol)?;
    out.set_item("optimization", optimization)?;
    Ok(out.unbind())
}

/// Update the shared standard consumer-theory configuration.
#[pyfunction]
#[pyo3(name = "set_standard_config")]
#[pyo3(signature = (*, preference = None, indifference = None, optimization = None))]
pub fn py_set_standard_config(
    preference: Option<Bound<'_, PyDict>>,
    indifference: Option<Bound<'_, PyDict>>,
    optimization: Option<Bound<'_, PyDict>>,
) -> PyResult<()> {
    let mut config = StandardConfig::get();

    if let Some(dict) = preference {
        set_nested_usize(&dict, "samples", &mut config.preference.samples)?;
        set_nested_u32(&dict, "seed", &mut config.preference.seed)?;
        set_nested_bool(
            &dict,
            "strict_monotonicity",
            &mut config.preference.strict_monotonicity,
        )?;
        set_nested_bool(
            &dict,
            "strict_convexity",
            &mut config.preference.strict_convexity,
        )?;
        set_nested_f64(&dict, "epsilon", &mut config.preference.epsilon)?;
        set_nested_f64(&dict, "tolerance", &mut config.preference.tolerance)?;
        set_nested_f64(
            &dict,
            "continuity_threshold",
            &mut config.preference.continuity_threshold,
        )?;
        set_nested_f64(&dict, "calc_epsilon", &mut config.preference.calc_epsilon)?;
        set_nested_f64(
            &dict,
            "calc_tolerance",
            &mut config.preference.calc_tolerance,
        )?;
    }

    if let Some(dict) = indifference {
        set_nested_usize(&dict, "n_points", &mut config.indifference.indiff_n_points)?;
        set_nested_f64(&dict, "tol", &mut config.indifference.indiff_tol)?;
    }

    if let Some(dict) = optimization {
        set_nested_f64(&dict, "mu_init", &mut config.optimization.optim_mu_init)?;
        set_nested_f64(&dict, "mu_decay", &mut config.optimization.optim_mu_decay)?;
        set_nested_usize(
            &dict,
            "outer_iters",
            &mut config.optimization.optim_outer_iters,
        )?;
        set_nested_usize(
            &dict,
            "inner_iters",
            &mut config.optimization.optim_inner_iters,
        )?;
        set_nested_f64(&dict, "step_size", &mut config.optimization.optim_step_size)?;
        set_nested_f64(&dict, "tol", &mut config.optimization.optim_tol)?;
    }

    StandardConfig::set(config);
    Ok(())
}

/// Restore the shared standard consumer-theory configuration to its defaults.
#[pyfunction]
#[pyo3(name = "restore_standard_config")]
pub fn py_restore_standard_config() -> PyResult<()> {
    StandardConfig::set(StandardConfig::default());
    Ok(())
}
