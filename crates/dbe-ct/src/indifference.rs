use super::pref::{FalliblePreference, Preference, PreferenceError, StandardConfig};

/// Configurable options for indifference curve tracing.
#[derive(Clone, Debug)]
pub struct IndiffConfig {
    /// Number of points to trace along the curve (must be >= 2).
    pub n_points: usize,
    /// Bisection tolerance for solving U(x) = target_utility (e.g. `1e-10`).
    pub tol: f64,
}

impl Default for IndiffConfig {
    fn default() -> Self {
        let standard = StandardConfig::get();
        Self {
            n_points: standard.indifference.indiff_n_points,
            tol: standard.indifference.indiff_tol,
        }
    }
}

/// Traces an indifference curve in the plane of two goods.
///
/// Grids `good_i` across its bounds and for each value uses bisection to find
/// the value of `good_j` such that U(x) = `target_utility`. All other goods
/// are held fixed at the midpoint of their bounds.
///
/// Points where no solution exists within the bounds of `good_j` are silently
/// skipped. Returns an error only if no points at all could be found.
///
/// # Arguments
/// * `pref` - A validated `Preference` whose rationality axioms are already
///   enforced
/// * `target_utility` - The utility level U* to trace
/// * `good_i` - Index of the good on the x-axis (gridded)
/// * `good_j` - Index of the good on the y-axis (solved via bisection)
/// * `config` - Curve tracing options
pub fn trace_2d<F: Fn(&[f64]) -> f64>(
    pref: &Preference<F>,
    target_utility: f64,
    good_i: usize,
    good_j: usize,
    config: IndiffConfig,
) -> Result<Vec<(f64, f64)>, String> {
    let lb = pref.min_bounds();
    let ub = pref.max_bounds();
    let dims = lb.len();

    if good_i >= dims {
        return Err(format!(
            "good_i ({}) is out of bounds for a {}-good preference",
            good_i, dims
        ));
    }
    if good_j >= dims {
        return Err(format!(
            "good_j ({}) is out of bounds for a {}-good preference",
            good_j, dims
        ));
    }
    if good_i == good_j {
        return Err("good_i and good_j must be different goods".into());
    }
    if config.n_points < 2 {
        return Err("n_points must be at least 2".into());
    }

    // Fix all goods at their midpoints;
    // good_i and good_j are overridden per iteration
    let base: Vec<f64> = (0..dims).map(|d| (lb[d] + ub[d]) / 2.0).collect();

    let i_min = lb[good_i];
    let i_max = ub[good_i];
    let step = (i_max - i_min) / (config.n_points - 1) as f64;

    let mut points = Vec::with_capacity(config.n_points);

    for k in 0..config.n_points {
        let xi = i_min + k as f64 * step;
        let state = BisectState {
            base: &base,
            good_i,
            xi,
            good_j,
            target: target_utility,
        };

        if let Some(xj) = bisect(pref, &state, lb[good_j], ub[good_j], config.tol) {
            points.push((xi, xj));
        }
    }

    if points.is_empty() {
        return Err(format!(
            "No points found on the indifference curve for target utility {}. \
            Check that the target utility is reachable within the bounds.",
            target_utility
        ));
    }

    Ok(points)
}

pub fn trace_2d_fallible<F, E>(
    pref: &FalliblePreference<F, E>,
    target_utility: f64,
    good_i: usize,
    good_j: usize,
    config: IndiffConfig,
) -> Result<Vec<(f64, f64)>, PreferenceError<E>>
where
    F: Fn(&[f64]) -> Result<f64, E>,
{
    let lb = pref.min_bounds();
    let ub = pref.max_bounds();
    let dims = lb.len();

    if good_i >= dims {
        return Err(PreferenceError::Config(format!(
            "good_i ({}) is out of bounds for a {}-good preference",
            good_i, dims
        )));
    }
    if good_j >= dims {
        return Err(PreferenceError::Config(format!(
            "good_j ({}) is out of bounds for a {}-good preference",
            good_j, dims
        )));
    }
    if good_i == good_j {
        return Err(PreferenceError::Config(
            "good_i and good_j must be different goods".into(),
        ));
    }
    if config.n_points < 2 {
        return Err(PreferenceError::Config(
            "n_points must be at least 2".into(),
        ));
    }

    let base: Vec<f64> = (0..dims).map(|d| (lb[d] + ub[d]) / 2.0).collect();
    let i_min = lb[good_i];
    let i_max = ub[good_i];
    let step = (i_max - i_min) / (config.n_points - 1) as f64;
    let mut points = Vec::with_capacity(config.n_points);

    for k in 0..config.n_points {
        let xi = i_min + k as f64 * step;
        let state = BisectState {
            base: &base,
            good_i,
            xi,
            good_j,
            target: target_utility,
        };
        if let Some(xj) = bisect_fallible(pref, &state, lb[good_j], ub[good_j], config.tol)? {
            points.push((xi, xj));
        }
    }

    if points.is_empty() {
        return Err(PreferenceError::Config(format!(
            "No points found on the indifference curve for target utility {}. \
            Check that the target utility is reachable within the bounds.",
            target_utility
        )));
    }

    Ok(points)
}

struct BisectState<'a> {
    base: &'a [f64],
    good_i: usize,
    xi: f64,
    good_j: usize,
    target: f64,
}

/// Bisects on `good_j` in `[j_lo, j_hi]` to find xj such that
/// U(bundle) - target = 0. Returns None if the target is not bracketed.
fn bisect<F: Fn(&[f64]) -> f64>(
    pref: &Preference<F>,
    state: &BisectState<'_>,
    j_lo: f64,
    j_hi: f64,
    tol: f64,
) -> Option<f64> {
    let eval = |xj: f64| -> f64 {
        let mut bundle = state.base.to_vec();
        bundle[state.good_i] = state.xi;
        bundle[state.good_j] = xj;
        pref.get_utility(&bundle) - state.target
    };

    let mut lo = j_lo;
    let mut hi = j_hi;
    let mut f_lo = eval(lo);
    let f_hi = eval(hi);

    // Target must be bracketed between lo and hi
    if f_lo * f_hi > 0.0 {
        return None;
    }

    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let f_mid = eval(mid);

        if f_mid.abs() < tol || (hi - lo) / 2.0 < tol {
            return Some(mid);
        }

        if f_lo * f_mid < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            f_lo = f_mid;
        }
    }

    Some((lo + hi) / 2.0)
}

fn bisect_fallible<F, E>(
    pref: &FalliblePreference<F, E>,
    state: &BisectState<'_>,
    j_lo: f64,
    j_hi: f64,
    tol: f64,
) -> Result<Option<f64>, PreferenceError<E>>
where
    F: Fn(&[f64]) -> Result<f64, E>,
{
    let eval = |xj: f64| -> Result<f64, PreferenceError<E>> {
        let mut bundle = state.base.to_vec();
        bundle[state.good_i] = state.xi;
        bundle[state.good_j] = xj;
        Ok(pref.get_utility(&bundle)? - state.target)
    };

    let mut lo = j_lo;
    let mut hi = j_hi;
    let mut f_lo = eval(lo)?;
    let f_hi = eval(hi)?;

    if f_lo * f_hi > 0.0 {
        return Ok(None);
    }

    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let f_mid = eval(mid)?;

        if f_mid.abs() < tol || (hi - lo) / 2.0 < tol {
            return Ok(Some(mid));
        }

        if f_lo * f_mid < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            f_lo = f_mid;
        }
    }

    Ok(Some((lo + hi) / 2.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pref::Preference;

    // Cobb-Douglas: U(x, y) = sqrt(x * y)
    fn cobb_douglas(bundle: &[f64]) -> f64 {
        bundle.iter().product::<f64>().sqrt()
    }

    // Linear: U(x, y) = x + y
    fn linear(bundle: &[f64]) -> f64 {
        bundle.iter().sum()
    }

    #[test]
    fn test_trace_2d_cobb_douglas_points_lie_on_curve() {
        // For U = sqrt(xy) = U*, every returned point must satisfy U(xi, xj) ~= U*
        let pref = Preference::new(cobb_douglas, vec![0.1, 0.1], vec![10.0, 10.0]).unwrap();
        let target = 2.0;
        let config = IndiffConfig::default();

        let points = trace_2d(&pref, target, 0, 1, config).unwrap();

        assert!(!points.is_empty(), "Expected at least one point");
        for (xi, xj) in &points {
            let u = cobb_douglas(&[*xi, *xj]);
            assert!(
                (u - target).abs() < 1e-6,
                "Point ({}, {}) has utility {} ~= {}",
                xi,
                xj,
                u,
                target
            );
        }
    }

    #[test]
    fn test_trace_2d_linear_returns_expected_n_points() {
        // For U(x, y) = x + y = target, every xi in [0, target] has a valid xj
        let pref = Preference::new(linear, vec![0.0, 0.0], vec![10.0, 10.0]).unwrap();
        let config = IndiffConfig {
            n_points: 50,
            tol: 1e-10,
        };

        let points = trace_2d(&pref, 8.0, 0, 1, config).unwrap();

        assert!(
            points.len() >= 40,
            "Expected close to 50 points, got {}",
            points.len()
        );
    }

    #[test]
    fn test_trace_2d_same_good_raises_err() {
        let pref = Preference::new(cobb_douglas, vec![0.1, 0.1], vec![10.0, 10.0]).unwrap();

        let result = trace_2d(&pref, 2.0, 0, 0, IndiffConfig::default());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be different"));
    }

    #[test]
    fn test_trace_2d_out_of_bounds_good_raises_err() {
        let pref = Preference::new(cobb_douglas, vec![0.1, 0.1], vec![10.0, 10.0]).unwrap();

        let result = trace_2d(&pref, 2.0, 0, 5, IndiffConfig::default());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_trace_2d_unreachable_utility_raises_err() {
        // U = sqrt(xy) <= sqrt(10*10) = 10, so target 999 is unreachable
        let pref = Preference::new(cobb_douglas, vec![0.1, 0.1], vec![10.0, 10.0]).unwrap();

        let result = trace_2d(&pref, 999.0, 0, 1, IndiffConfig::default());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No points found"));
    }

    #[test]
    fn test_trace_2d_n_points_less_than_2_raises_err() {
        let pref = Preference::new(cobb_douglas, vec![0.1, 0.1], vec![10.0, 10.0]).unwrap();
        let config = IndiffConfig {
            n_points: 1,
            tol: 1e-10,
        };

        let result = trace_2d(&pref, 2.0, 0, 1, config);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("n_points must be at least 2"));
    }
}
