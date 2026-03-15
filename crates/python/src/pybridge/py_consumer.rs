use dbe::econ::consumer::{
    indifference::{self, IndiffConfig},
    marshallian::{self, BudgetConstraint, OptimConfig},
    pref::{CalcConfig, Preference, ValidationConfig},
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// PyO3 wrapper around `Preference<F>`.
///
/// Accepts a Python callable as the utility function. Validation of the
/// Axioms of Rationality is run on construction (unless `validate=False`).
#[pyclass(name = "PyPreference")]
pub struct PyPreference {
    pub(crate) utility_func: Py<PyAny>,
    pub(crate) min_bounds: Vec<f64>,
    pub(crate) max_bounds: Vec<f64>,
    pub(crate) config: ValidationConfig,
    pub(crate) calc_config: CalcConfig,
}

#[pymethods]
impl PyPreference {
    #[new]
    #[pyo3(signature = (
        utility_func,
        min_bounds,
        max_bounds,
        *,
        samples = 60_000,
        seed = 0,
        strict_monotonicity = false,
        strict_convexity = false,
        epsilon = 1e-6,
        tolerance = 1e-9,
        continuity_threshold = 1.0,
        validate = true,
        calc_epsilon = 1e-7,
        calc_tolerance = 1e-9,
    ))]
    pub fn new(
        py: Python<'_>,
        utility_func: Py<PyAny>,
        min_bounds: Vec<f64>,
        max_bounds: Vec<f64>,
        samples: usize,
        seed: u32,
        strict_monotonicity: bool,
        strict_convexity: bool,
        epsilon: f64,
        tolerance: f64,
        continuity_threshold: f64,
        validate: bool,
        calc_epsilon: f64,
        calc_tolerance: f64,
    ) -> PyResult<Self> {
        let config = ValidationConfig {
            samples,
            seed,
            strict_monotonicity,
            strict_convexity,
            epsilon,
            tolerance,
            continuity_threshold,
            validate,
        };
        let calc_config = CalcConfig {
            epsilon: calc_epsilon,
            tolerance: calc_tolerance,
        };

        // Build a temporary Preference to validate bounds and, if enabled,
        // the Axioms of Rationality. The Preference is dropped immediately after.
        let func = utility_func.clone_ref(py);
        let config_clone = config.clone();
        let calc_clone = calc_config.clone();
        Preference::with_config(
            move |b: &[f64]| {
                Python::try_attach(|py2| {
                    func.call1(py2, (b.to_vec(),))
                        .unwrap()
                        .extract::<f64>(py2)
                        .unwrap()
                }).unwrap()
            },
            min_bounds.clone(),
            max_bounds.clone(),
            config_clone,
            calc_clone,
        )
        .map_err(|e| PyErr::new::<PyValueError, _>(e))?;

        Ok(PyPreference {
            utility_func,
            min_bounds,
            max_bounds,
            config,
            calc_config,
        })
    }

    /// Evaluates the utility function at the given bundle.
    fn get_utility(&self, py: Python<'_>, bundle: Vec<f64>) -> PyResult<f64> {
        self.utility_func.call1(py, (bundle,))?.extract(py)
    }

    /// Computes marginal utility for a specific good via central differences.
    fn get_mu(&self, py: Python<'_>, bundle: Vec<f64>, good: usize) -> PyResult<f64> {
        let ep = self.calc_config.epsilon;
        let mut inc = bundle.clone();
        let mut dec = bundle;
        inc[good] += ep;
        dec[good] -= ep;
        let u_inc: f64 = self.utility_func.call1(py, (inc,))?.extract(py)?;
        let u_dec: f64 = self.utility_func.call1(py, (dec,))?.extract(py)?;
        Ok((u_inc - u_dec) / (2.0 * ep))
    }

    /// Computes the marginal rate of substitution between good_i and good_j.
    fn get_mrs(
        &self,
        py: Python<'_>,
        bundle: Vec<f64>,
        good_i: usize,
        good_j: usize,
    ) -> PyResult<f64> {
        let mu_j = self.get_mu(py, bundle.clone(), good_j)?;
        if mu_j.abs() < self.calc_config.tolerance {
            return Err(PyErr::new::<PyValueError, _>(format!(
                "MRS undefined: MU of good {} is zero at {:?}",
                good_j, bundle
            )));
        }
        Ok(self.get_mu(py, bundle, good_i)? / mu_j)
    }
}

/// Finds the optimal consumption bundle that maximises utility subject to a
/// budget constraint.
///
/// Uses an interior-point log-barrier method with backtracking line search.
///
/// # Arguments
/// * `pref` - A `PyPreference` whose rationality axioms are already enforced
/// * `prices` - Price vector, must match the number of goods in `pref`
/// * `income` - Total income available to the consumer
#[pyfunction]
#[pyo3(name = "rs_optimal_bundle")]
#[pyo3(signature = (
    pref,
    prices,
    income,
    *,
    mu_init = 1.0,
    mu_decay = 0.1,
    outer_iters = 10,
    inner_iters = 500,
    step_size = 1e-2,
    tol = 1e-8,
))]
pub fn py_optimal_bundle(
    py: Python<'_>,
    pref: &PyPreference,
    prices: Vec<f64>,
    income: f64,
    mu_init: f64,
    mu_decay: f64,
    outer_iters: usize,
    inner_iters: usize,
    step_size: f64,
    tol: f64,
) -> PyResult<Vec<f64>> {
    let func = pref.utility_func.clone_ref(py);
    let mut config = pref.config.clone();
    config.validate = false;

    let rust_pref = Preference::with_config(
        move |b: &[f64]| {
            Python::try_attach(|py2| {
                func.call1(py2, (b.to_vec(),))
                    .unwrap()
                    .extract::<f64>(py2)
                    .unwrap()
            }).unwrap()
        },
        pref.min_bounds.clone(),
        pref.max_bounds.clone(),
        config,
        pref.calc_config.clone(),
    )
    .map_err(|e| PyErr::new::<PyValueError, _>(e))?;

    let constraint = BudgetConstraint { prices, income };
    let optim_config = OptimConfig {
        mu_init,
        mu_decay,
        outer_iters,
        inner_iters,
        step_size,
        tol,
    };

    marshallian::optimal_bundle(&rust_pref, &constraint, optim_config)
        .map_err(|e| PyErr::new::<PyValueError, _>(e))
}

/// Traces an indifference curve in the plane of two goods.
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
#[pyo3(name = "rs_trace_2d")]
#[pyo3(signature = (pref, target_utility, good_i, good_j, *, n_points = 200, tol = 1e-10))]
pub fn py_trace_2d(
    py: Python<'_>,
    pref: &PyPreference,
    target_utility: f64,
    good_i: usize,
    good_j: usize,
    n_points: usize,
    tol: f64,
) -> PyResult<Vec<(f64, f64)>> {
    let func = pref.utility_func.clone_ref(py);
    let mut config = pref.config.clone();
    config.validate = false;

    let rust_pref = Preference::with_config(
        move |b: &[f64]| {
            Python::try_attach(|py2| {
                func.call1(py2, (b.to_vec(),))
                    .unwrap()
                    .extract::<f64>(py2)
                    .unwrap()
            }).unwrap()
        },
        pref.min_bounds.clone(),
        pref.max_bounds.clone(),
        config,
        pref.calc_config.clone(),
    )
    .map_err(|e| PyErr::new::<PyValueError, _>(e))?;

    let indiff_config = IndiffConfig { n_points, tol };
    indifference::trace_2d(&rust_pref, target_utility, good_i, good_j, indiff_config)
        .map_err(|e| PyErr::new::<PyValueError, _>(e))
}
