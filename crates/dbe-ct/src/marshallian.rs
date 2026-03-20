use super::pref::{FalliblePreference, Preference, PreferenceError, StandardConfig};

/// A linear budget constraint: p * x <= income
pub struct BudgetConstraint {
    /// Prices for each good, must match the dimensionality of the Preference bounds.
    pub prices: Vec<f64>,
    /// Total income available to the consumer.
    pub income: f64,
}

/// Configurable options for the interior-point bundle optimisation.
#[derive(Clone, Debug)]
pub struct OptimConfig {
    /// Initial barrier weight.
    pub mu_init: f64,
    /// Multiplicative decay applied to mu per outer iteration (e.g. `0.1`).
    pub mu_decay: f64,
    /// Number of outer iterations (mu reductions).
    pub outer_iters: usize,
    /// Number of gradient ascent steps per outer iteration.
    pub inner_iters: usize,
    /// Initial step size for gradient ascent.
    pub step_size: f64,
    /// Convergence tolerance: stops if the gradient norm falls below this.
    pub tol: f64,
}

impl Default for OptimConfig {
    fn default() -> Self {
        let standard = StandardConfig::get();
        Self {
            mu_init: standard.optimization.optim_mu_init,
            mu_decay: standard.optimization.optim_mu_decay,
            outer_iters: standard.optimization.optim_outer_iters,
            inner_iters: standard.optimization.optim_inner_iters,
            step_size: standard.optimization.optim_step_size,
            tol: standard.optimization.optim_tol,
        }
    }
}

/// Finds the optimal consumption bundle that maximises utility subject to a
/// budget constraint.
///
/// Uses an interior-point log-barrier method with backtracking line search.
/// The barrier objective is:
///
/// `B(x, mu) = U(x) + mu * log(I - p * x) + mu * SUM(log(x_i - lb_i) + mu * SUM(log(ub_i - x_i))`
///
/// As mu -> 0 across outer iterations the solution converges to the constrained
/// optimum.
///
/// # Arguments
/// * `pref` - A validated `Preference` whose rationality axioms are already
///   enforced
/// * `constraint` - The budget constraint `p * x <= income`
/// * `config` - Optimisation hyperparameters
pub fn optimal_bundle<F: Fn(&[f64]) -> f64>(
    pref: &Preference<F>,
    constraint: &BudgetConstraint,
    config: OptimConfig,
) -> Result<Vec<f64>, String> {
    let lb = pref.min_bounds();
    let ub = pref.max_bounds();
    let dims = lb.len();
    let prices = &constraint.prices;
    let income = constraint.income;

    if prices.len() != dims {
        return Err("prices length must match the number of goods in the Preference".into());
    }
    if income <= 0.0 {
        return Err("income must be positive".into());
    }
    if prices.iter().any(|&p| p <= 0.0) {
        return Err("all prices must be positive".into());
    }

    // Distribute income equally across goods (equal expenditure share), then
    // pull 50% toward the lower bound to ensure strict interior feasibility.
    let mut x: Vec<f64> = (0..dims)
        .map(|i| {
            let equal_share = income / (dims as f64 * prices[i]);
            lb[i] + 0.5 * (equal_share.min(ub[i]) - lb[i])
        })
        .collect();

    if !is_feasible(&x, lb, ub, prices, income) {
        return Err("Could not find a feasible interior starting point".into());
    }

    let mut mu = config.mu_init;

    for _ in 0..config.outer_iters {
        for _ in 0..config.inner_iters {
            let grad = barrier_gradient(pref, &x, lb, ub, prices, income, mu);

            // Check convergence: gradient norm
            let grad_norm: f64 = grad.iter().map(|g| g * g).sum::<f64>().sqrt();
            if grad_norm < config.tol {
                return Ok(x);
            }

            // Backtracking line search: halve step until the move stays feasible
            // and the barrier objective improves
            let b_current = barrier_value(pref, &x, lb, ub, prices, income, mu);
            let mut step = config.step_size;
            let x_new = loop {
                let candidate: Vec<f64> =
                    x.iter().zip(&grad).map(|(xi, gi)| xi + step * gi).collect();

                if is_feasible(&candidate, lb, ub, prices, income) {
                    let b_new = barrier_value(pref, &candidate, lb, ub, prices, income, mu);
                    if b_new > b_current {
                        break candidate;
                    }
                }

                step *= 0.5;

                // step approaches zero -> at a local maximum -> stop inner loop
                if step < 1e-15 {
                    break x.clone();
                }
            };

            x = x_new;
        }

        mu *= config.mu_decay;
    }

    Ok(x)
}

pub fn optimal_bundle_fallible<F, E>(
    pref: &FalliblePreference<F, E>,
    constraint: &BudgetConstraint,
    config: OptimConfig,
) -> Result<Vec<f64>, PreferenceError<E>>
where
    F: Fn(&[f64]) -> Result<f64, E>,
{
    let lb = pref.min_bounds();
    let ub = pref.max_bounds();
    let dims = lb.len();
    let prices = &constraint.prices;
    let income = constraint.income;

    if prices.len() != dims {
        return Err(PreferenceError::Config(
            "prices length must match the number of goods in the Preference".into(),
        ));
    }
    if income <= 0.0 {
        return Err(PreferenceError::Config("income must be positive".into()));
    }
    if prices.iter().any(|&p| p <= 0.0) {
        return Err(PreferenceError::Config(
            "all prices must be positive".into(),
        ));
    }

    let mut x: Vec<f64> = (0..dims)
        .map(|i| {
            let equal_share = income / (dims as f64 * prices[i]);
            lb[i] + 0.5 * (equal_share.min(ub[i]) - lb[i])
        })
        .collect();

    if !is_feasible(&x, lb, ub, prices, income) {
        return Err(PreferenceError::Config(
            "Could not find a feasible interior starting point".into(),
        ));
    }

    let mut mu = config.mu_init;

    for _ in 0..config.outer_iters {
        for _ in 0..config.inner_iters {
            let grad = barrier_gradient_fallible(pref, &x, lb, ub, prices, income, mu)?;
            let grad_norm: f64 = grad.iter().map(|g| g * g).sum::<f64>().sqrt();
            if grad_norm < config.tol {
                return Ok(x);
            }

            let b_current = barrier_value_fallible(pref, &x, lb, ub, prices, income, mu)?;
            let mut step = config.step_size;
            let x_new = loop {
                let candidate: Vec<f64> =
                    x.iter().zip(&grad).map(|(xi, gi)| xi + step * gi).collect();

                if is_feasible(&candidate, lb, ub, prices, income) {
                    let b_new =
                        barrier_value_fallible(pref, &candidate, lb, ub, prices, income, mu)?;
                    if b_new > b_current {
                        break candidate;
                    }
                }

                step *= 0.5;
                if step < 1e-15 {
                    break x.clone();
                }
            };

            x = x_new;
        }

        mu *= config.mu_decay;
    }

    Ok(x)
}

/// Evaluates the log-barrier objective at a given point.
fn barrier_value<F: Fn(&[f64]) -> f64>(
    pref: &Preference<F>,
    x: &[f64],
    lb: &[f64],
    ub: &[f64],
    prices: &[f64],
    income: f64,
    mu: f64,
) -> f64 {
    let budget_slack = income - dot(prices, x);
    let lb_slack: f64 = x.iter().zip(lb).map(|(xi, li)| (xi - li).ln()).sum();
    let ub_slack: f64 = x.iter().zip(ub).map(|(xi, ui)| (ui - xi).ln()).sum();
    pref.get_utility(x) + mu * (budget_slack.ln() + lb_slack + ub_slack)
}

/// Computes the gradient of the barrier objective via numerical MU +
/// analytical constraint terms.
fn barrier_gradient<F: Fn(&[f64]) -> f64>(
    pref: &Preference<F>,
    x: &[f64],
    lb: &[f64],
    ub: &[f64],
    prices: &[f64],
    income: f64,
    mu: f64,
) -> Vec<f64> {
    let budget_slack = income - dot(prices, x);
    (0..x.len())
        .map(|i| {
            let du = pref.get_mu(x, i);
            let budget_term = -mu * prices[i] / budget_slack;
            let lb_term = mu / (x[i] - lb[i]);
            let ub_term = -mu / (ub[i] - x[i]);
            du + budget_term + lb_term + ub_term
        })
        .collect()
}

fn barrier_value_fallible<F, E>(
    pref: &FalliblePreference<F, E>,
    x: &[f64],
    lb: &[f64],
    ub: &[f64],
    prices: &[f64],
    income: f64,
    mu: f64,
) -> Result<f64, PreferenceError<E>>
where
    F: Fn(&[f64]) -> Result<f64, E>,
{
    let budget_slack = income - dot(prices, x);
    let lb_slack: f64 = x.iter().zip(lb).map(|(xi, li)| (xi - li).ln()).sum();
    let ub_slack: f64 = x.iter().zip(ub).map(|(xi, ui)| (ui - xi).ln()).sum();
    Ok(pref.get_utility(x)? + mu * (budget_slack.ln() + lb_slack + ub_slack))
}

fn barrier_gradient_fallible<F, E>(
    pref: &FalliblePreference<F, E>,
    x: &[f64],
    lb: &[f64],
    ub: &[f64],
    prices: &[f64],
    income: f64,
    mu: f64,
) -> Result<Vec<f64>, PreferenceError<E>>
where
    F: Fn(&[f64]) -> Result<f64, E>,
{
    let budget_slack = income - dot(prices, x);
    (0..x.len())
        .map(|i| {
            let du = pref.get_mu(x, i)?;
            let budget_term = -mu * prices[i] / budget_slack;
            let lb_term = mu / (x[i] - lb[i]);
            let ub_term = -mu / (ub[i] - x[i]);
            Ok(du + budget_term + lb_term + ub_term)
        })
        .collect()
}

/// Returns true if x is strictly interior to all constraints.
fn is_feasible(x: &[f64], lb: &[f64], ub: &[f64], prices: &[f64], income: f64) -> bool {
    let budget_ok = dot(prices, x) < income;
    let bounds_ok = x
        .iter()
        .zip(lb)
        .zip(ub)
        .all(|((xi, li), ui)| xi > li && xi < ui);
    budget_ok && bounds_ok
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(ai, bi)| ai * bi).sum()
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
    fn test_optimal_bundle_cobb_douglas_returns_ok() {
        // For U = sqrt(xy), p = [1, 1], I = 10:
        // Analytical solution: x* = y* = I / (2p) = 5.0
        let pref = Preference::new(cobb_douglas, vec![0.0, 0.0], vec![20.0, 20.0]).unwrap();
        let constraint = BudgetConstraint {
            prices: vec![1.0, 1.0],
            income: 10.0,
        };

        let result = optimal_bundle(&pref, &constraint, OptimConfig::default());

        assert!(result.is_ok(), "Optimisation failed: {:?}", result);
        let x = result.unwrap();
        assert!((x[0] - 5.0).abs() < 0.1, "Expected x ~= 5.0, got {}", x[0]);
        assert!((x[1] - 5.0).abs() < 0.1, "Expected y ~= 5.0, got {}", x[1]);
    }

    #[test]
    fn test_optimal_bundle_mismatched_prices_raises_err() {
        let pref = Preference::new(linear, vec![0.0, 0.0], vec![10.0, 10.0]).unwrap();
        let constraint = BudgetConstraint {
            prices: vec![1.0],
            income: 10.0,
        };

        let result = optimal_bundle(&pref, &constraint, OptimConfig::default());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("prices length"));
    }

    #[test]
    fn test_optimal_bundle_zero_income_raises_err() {
        let pref = Preference::new(linear, vec![0.0, 0.0], vec![10.0, 10.0]).unwrap();
        let constraint = BudgetConstraint {
            prices: vec![1.0, 1.0],
            income: 0.0,
        };

        let result = optimal_bundle(&pref, &constraint, OptimConfig::default());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("income must be positive"));
    }

    #[test]
    fn test_optimal_bundle_solution_satisfies_budget_constraint() {
        let pref = Preference::new(cobb_douglas, vec![0.0, 0.0], vec![20.0, 20.0]).unwrap();
        let prices = vec![2.0, 3.0];
        let income = 12.0;
        let constraint = BudgetConstraint {
            prices: prices.clone(),
            income,
        };

        let x = optimal_bundle(&pref, &constraint, OptimConfig::default()).unwrap();

        let expenditure: f64 = x.iter().zip(&prices).map(|(xi, pi)| xi * pi).sum();
        assert!(
            expenditure <= income + 1e-6,
            "Budget constraint violated: expenditure {} > income {}",
            expenditure,
            income
        );
    }
}
