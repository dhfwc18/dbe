use std::sync::{LazyLock, RwLock};

/// Configurable options for numerical calculations (MU, MRS, etc.).
#[derive(Clone, Debug)]
pub struct CalcConfig {
    /// Step size for central diff numerical differentiation (e.g. `1e-7`).
    pub epsilon: f64,
    /// Tolerance for f64 comparisons e.g. MRS zero denominator (e.g. `1e-9`).
    pub tolerance: f64,
}

impl Default for CalcConfig {
    fn default() -> Self {
        Self {
            epsilon: 1e-7,
            tolerance: 1e-9,
        }
    }
}

/// Standard consumer-theory configuration shared across default constructors.
#[derive(Clone, Debug)]
pub struct StandardConfig {
    pub preference: PreferenceConfig,
    pub indifference: IndifferenceConfig,
    pub optimization: OptimizationConfig,
}

#[derive(Clone, Debug)]
pub struct PreferenceConfig {
    /// Number of Sobol sequence points to sample for the rationality checks.
    pub samples: usize,
    /// Seed for the Sobol sequence scrambling.
    pub seed: u32,
    /// Whether to enforce strict monotonicity (`U(x + ep) > U(x)`).
    pub strict_monotonicity: bool,
    /// Whether to enforce strict convexity (`U(midpoint) > min(U(A), U(B))`).
    pub strict_convexity: bool,
    /// The small increment used to compute directional changes (e.g. `1e-6`).
    pub epsilon: f64,
    /// Numerical tolerance for floating-point comparisons (e.g. `1e-9`).
    pub tolerance: f64,
    /// The threshold marginal utility (d_epsilon -> d_U) for a continuous function.
    pub continuity_threshold: f64,
    /// Step size for numerical differentiation (e.g. `1e-7`).
    pub calc_epsilon: f64,
    /// Tolerance for numerical calculations such as MU/MRS comparisons.
    pub calc_tolerance: f64,
}

#[derive(Clone, Debug)]
pub struct IndifferenceConfig {
    /// Number of points to trace along an indifference curve.
    pub indiff_n_points: usize,
    /// Bisection tolerance for indifference tracing.
    pub indiff_tol: f64,
}

#[derive(Clone, Debug)]
pub struct OptimizationConfig {
    /// Initial barrier weight for bundle optimisation.
    pub optim_mu_init: f64,
    /// Barrier decay for bundle optimisation.
    pub optim_mu_decay: f64,
    /// Number of outer iterations for bundle optimisation.
    pub optim_outer_iters: usize,
    /// Number of inner iterations for bundle optimisation.
    pub optim_inner_iters: usize,
    /// Initial step size for bundle optimisation.
    pub optim_step_size: f64,
    /// Convergence tolerance for bundle optimisation.
    pub optim_tol: f64,
}

impl Default for StandardConfig {
    fn default() -> Self {
        Self {
            preference: PreferenceConfig {
                samples: 60_000,
                seed: 0,
                strict_monotonicity: false,
                strict_convexity: false,
                epsilon: 1e-6,
                tolerance: 1e-9,
                continuity_threshold: 1.0,
                calc_epsilon: 1e-7,
                calc_tolerance: 1e-9,
            },
            indifference: IndifferenceConfig {
                indiff_n_points: 200,
                indiff_tol: 1e-10,
            },
            optimization: OptimizationConfig {
                optim_mu_init: 1.0,
                optim_mu_decay: 0.1,
                optim_outer_iters: 10,
                optim_inner_iters: 500,
                optim_step_size: 1e-2,
                optim_tol: 1e-8,
            },
        }
    }
}

static STANDARD_CONFIG: LazyLock<RwLock<StandardConfig>> =
    LazyLock::new(|| RwLock::new(StandardConfig::default()));

/// Configurable options for the Axioms of Rationality validations.
#[derive(Clone, Debug)]
pub struct ValidationConfig {
    /// Number of Sobol sequence points to sample for the rationality checks.
    pub samples: usize,
    /// Seed for the Sobol sequence scrambling.
    pub seed: u32,
    /// Whether to enforce strict monotonicity (`U(x + ep) > U(x)`).
    /// If false, requires weak monotonicity (`U(x + ep) >= U(x)`).
    pub strict_monotonicity: bool,
    /// Whether to enforce strict convexity (`U(midpoint) > min(U(A), U(B))`).
    /// If false, requires weak convexity (`U(midpoint) >= min(U(A), U(B))`).
    pub strict_convexity: bool,
    /// The small increment used to compute directional changes (e.g. `1e-6`).
    pub epsilon: f64,
    /// Numerical tolerance for floating-point comparisons (e.g. `1e-9`).
    pub tolerance: f64,
    /// The threshold marginal utility (d_epsilon -> d_U) for a continuous function.
    pub continuity_threshold: f64,
    /// Whether to validate the Axioms of Rationality upon construction.
    pub validate: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        StandardConfig::get().validation_config(true)
    }
}

impl StandardConfig {
    pub fn set(config: StandardConfig) {
        *STANDARD_CONFIG.write().unwrap() = config;
    }

    pub fn get() -> StandardConfig {
        STANDARD_CONFIG.read().unwrap().clone()
    }

    pub fn validation_config(&self, validate: bool) -> ValidationConfig {
        ValidationConfig {
            samples: self.preference.samples,
            seed: self.preference.seed,
            strict_monotonicity: self.preference.strict_monotonicity,
            strict_convexity: self.preference.strict_convexity,
            epsilon: self.preference.epsilon,
            tolerance: self.preference.tolerance,
            continuity_threshold: self.preference.continuity_threshold,
            validate,
        }
    }

    pub fn calc_config(&self) -> CalcConfig {
        CalcConfig {
            epsilon: self.preference.calc_epsilon,
            tolerance: self.preference.calc_tolerance,
        }
    }
}

#[derive(Debug)]
pub enum PreferenceError<E> {
    Config(String),
    Eval(E),
}

impl<E> PreferenceError<E> {
    fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
}

/// Data structure to represent the preference of a consumer
pub struct Preference<F>
where
    F: Fn(&[f64]) -> f64,
{
    utility_func: F,      // Utility function U(x)
    min_bounds: Vec<f64>, // Min limit (usually 0)
    max_bounds: Vec<f64>, // Max limit (satiation)
    config: ValidationConfig,
    calc_config: CalcConfig,
}

/// Fallible preference representation for callback-driven frontends.
pub struct FalliblePreference<F, E>
where
    F: Fn(&[f64]) -> Result<f64, E>,
{
    utility_func: F,
    min_bounds: Vec<f64>,
    max_bounds: Vec<f64>,
    config: ValidationConfig,
    calc_config: CalcConfig,
}

impl<F> Preference<F>
where
    F: Fn(&[f64]) -> f64,
{
    /// Constructor - enforce the Axioms of Rationality upon creation on default config
    pub fn new(
        utility_func: F,
        min_bounds: Vec<f64>,
        max_bounds: Vec<f64>,
    ) -> Result<Self, String> {
        Self::with_validation(utility_func, min_bounds, max_bounds, true)
    }

    /// Constructor using the shared standard config with per-instance validation control.
    pub fn with_validation(
        utility_func: F,
        min_bounds: Vec<f64>,
        max_bounds: Vec<f64>,
        validate: bool,
    ) -> Result<Self, String> {
        let standard = StandardConfig::get();
        Self::with_config(
            utility_func,
            min_bounds,
            max_bounds,
            standard.validation_config(validate),
            standard.calc_config(),
        )
    }

    /// Constructor - enforce or not enforce the Axioms of Rationality on
    /// customised config
    pub fn with_config(
        utility_func: F,
        min_bounds: Vec<f64>,
        max_bounds: Vec<f64>,
        config: ValidationConfig,
        calc_config: CalcConfig,
    ) -> Result<Self, String> {
        validate_bounds(&min_bounds, &max_bounds)?;

        let instance = Self {
            utility_func,
            min_bounds,
            max_bounds,
            config,
            calc_config,
        };

        if instance.config.validate {
            instance.validate_rationality()?;
        }

        Ok(instance)
    }

    /// Getter for the utility of a specific consumption bundle
    pub fn get_utility(&self, bundle: &[f64]) -> f64 {
        (self.utility_func)(bundle)
    }

    pub fn min_bounds(&self) -> &[f64] {
        &self.min_bounds
    }

    pub fn max_bounds(&self) -> &[f64] {
        &self.max_bounds
    }

    /// Getter for marginal utility
    ///
    /// This method allows user to access the marginal utility for a specified
    /// good, assuming consumption for all other goods in the bundle remains
    /// equal:
    ///
    /// mu = (U(x + ep) - U(x - ep)) / 2
    ///
    /// # Arguments
    /// * bundle - the bundle of goods in question
    /// * good - index for the good to be evaluated
    pub fn get_mu(&self, bundle: &[f64], good: usize) -> f64 {
        let ep = self.calc_config.epsilon;
        let mut increase = bundle.to_vec();
        let mut decrease = bundle.to_vec();
        increase[good] += ep;
        decrease[good] -= ep;
        (self.get_utility(&increase) - self.get_utility(&decrease)) / (2.0 * ep)
    }

    pub fn get_mrs(&self, bundle: &[f64], good_i: usize, good_j: usize) -> Result<f64, String> {
        let mu_j = self.get_mu(bundle, good_j);
        if mu_j.abs() < self.calc_config.tolerance {
            return Err(format!(
                "MRS undefined: MU of good {} is zero at {:?}",
                good_j, bundle
            ));
        }
        Ok(self.get_mu(bundle, good_i) / mu_j)
    }
    /// Collection of all Axioms of Rationality validations
    fn validate_rationality(&self) -> Result<(), String> {
        // Generate the sample points once and reuse them for all checks
        let points = self.generate_samples();

        self.check_continuity(&points)?;
        self.check_monotonicity(&points)?;
        self.check_convexity(&points)?;
        Ok(())
    }

    /// Generates deterministic corner/edge points and Sobol quasi-random samples
    fn generate_samples(&self) -> Vec<Vec<f64>> {
        let mut points = Vec::new();
        let dims = self.min_bounds.len();

        // Always include the exact midpoint as a deterministic baseline
        points.push(
            self.min_bounds
                .iter()
                .zip(&self.max_bounds)
                .map(|(l, h)| (l + h) / 2.0)
                .collect(),
        );

        // Generate corner/edge points
        // In highly dimensional spaces, generating *all* 2^N corners is prohibitive.
        // A core subset of critical extremes are generated instead:

        // E1: All minimum bounds
        points.push(self.min_bounds.clone());

        // E2: All maximum bounds
        points.push(self.max_bounds.clone());

        // E3: Points where exactly ONE dimension is max, and all others are min
        for d in 0..dims {
            let mut corner = self.min_bounds.clone();
            corner[d] = self.max_bounds[d];
            points.push(corner);
        }

        // E4: Points where exactly ONE dimension is min, and all others are max
        if dims > 1 {
            for d in 0..dims {
                let mut corner = self.max_bounds.clone();
                corner[d] = self.min_bounds[d];
                points.push(corner);
            }
        }

        if self.config.samples > 0 {
            self.generate_sobol_samples(&mut points, self.config.samples);
        }

        points
    }

    /// Sobol Sequence Sampling (Quasi-Random)
    fn generate_sobol_samples(&self, points: &mut Vec<Vec<f64>>, n: usize) {
        let dims = self.min_bounds.len();

        // sobol_burley supports up to 65535 dimensions
        if dims > 0 && dims <= 65535 {
            for i in 0..n {
                let mut p = Vec::with_capacity(dims);
                for d in 0..dims {
                    let min = self.min_bounds[d];
                    let max = self.max_bounds[d];

                    // Sample returns a value in [0, 1)
                    let sobol_val = sobol_burley::sample(i as u32, d as u32, self.config.seed);
                    p.push(min + sobol_val as f64 * (max - min));
                }
                points.push(p);
            }
        }
    }

    /// Axiom: Continuity
    fn check_continuity(&self, points: &[Vec<f64>]) -> Result<(), String> {
        for p in points {
            let u_start = self.get_utility(p);

            for i in 0..p.len() {
                let mut p_tiny = p.clone();
                // Add or subtract epsilon and ensure point stay within bounds
                if p_tiny[i] + self.config.epsilon <= self.max_bounds[i] {
                    p_tiny[i] += self.config.epsilon;
                } else if p_tiny[i] - self.config.epsilon >= self.min_bounds[i] {
                    p_tiny[i] -= self.config.epsilon;
                } else {
                    continue; // Skip if bounds are too tight to perturb
                }

                let u_end = self.get_utility(&p_tiny);
                if (u_end - u_start).abs() > self.config.continuity_threshold {
                    return Err(format!(
                        "Continuity Violated: Detected a jump in the utility function \
                        at index {} near {:?}",
                        i, p
                    ));
                }
            }
        }
        Ok(())
    }

    /// Axiom: Monotonicity
    fn check_monotonicity(&self, points: &[Vec<f64>]) -> Result<(), String> {
        for test_point in points {
            let u_base = self.get_utility(test_point);

            for i in 0..test_point.len() {
                let mut p_plus = test_point.clone();
                // Check if we have room to add epsilon
                if p_plus[i] + self.config.epsilon <= self.max_bounds[i] {
                    p_plus[i] += self.config.epsilon;
                    let u_plus = self.get_utility(&p_plus);

                    if self.config.strict_monotonicity {
                        if u_plus <= u_base + self.config.tolerance {
                            return Err(format!(
                                "Strict Monotonicity Violated: Utility failed to \
                                strictly increase at index {} near {:?}",
                                i, test_point
                            ));
                        }
                    } else {
                        if u_plus < u_base - self.config.tolerance {
                            return Err(format!(
                                "Weak Monotonicity Violated: Utility decreased at \
                                index {} near {:?}",
                                i, test_point
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Axiom: Convexity
    fn check_convexity(&self, points: &[Vec<f64>]) -> Result<(), String> {
        // Test pair points. Pair point `i` with point `len - 1 - i`
        let len = points.len();
        for i in 0..(len / 2) {
            let a = &points[i];
            let b = &points[len - 1 - i];

            let u_a = self.get_utility(a);
            let u_b = self.get_utility(b);

            let midpoint: Vec<f64> = a.iter().zip(b).map(|(x, y)| (x + y) / 2.0).collect();
            let u_mid = self.get_utility(&midpoint);

            let worst_utility = u_a.min(u_b);

            if self.config.strict_convexity {
                let distance_squared: f64 = a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum();
                if distance_squared > self.config.epsilon
                    && u_mid <= worst_utility + self.config.tolerance
                {
                    return Err(format!(
                        "Strict Convexity Violated: Averages not strictly \
                        preferred to extremes between {:?} and {:?}",
                        a, b
                    ));
                }
            } else {
                if u_mid < worst_utility - self.config.tolerance {
                    return Err(format!(
                        "Weak Convexity Violated: Preference for extremes detected \
                        between {:?} and {:?}",
                        a, b
                    ));
                }
            }
        }
        Ok(())
    }
}

impl<F, E> FalliblePreference<F, E>
where
    F: Fn(&[f64]) -> Result<f64, E>,
{
    pub fn with_validation(
        utility_func: F,
        min_bounds: Vec<f64>,
        max_bounds: Vec<f64>,
        validate: bool,
    ) -> Result<Self, PreferenceError<E>> {
        let standard = StandardConfig::get();
        Self::with_config(
            utility_func,
            min_bounds,
            max_bounds,
            standard.validation_config(validate),
            standard.calc_config(),
        )
    }

    pub fn with_config(
        utility_func: F,
        min_bounds: Vec<f64>,
        max_bounds: Vec<f64>,
        config: ValidationConfig,
        calc_config: CalcConfig,
    ) -> Result<Self, PreferenceError<E>> {
        validate_bounds(&min_bounds, &max_bounds).map_err(PreferenceError::config)?;

        let instance = Self {
            utility_func,
            min_bounds,
            max_bounds,
            config,
            calc_config,
        };

        if instance.config.validate {
            instance.validate_rationality()?;
        }

        Ok(instance)
    }

    pub fn get_utility(&self, bundle: &[f64]) -> Result<f64, PreferenceError<E>> {
        (self.utility_func)(bundle).map_err(PreferenceError::Eval)
    }

    pub fn min_bounds(&self) -> &[f64] {
        &self.min_bounds
    }

    pub fn max_bounds(&self) -> &[f64] {
        &self.max_bounds
    }

    pub fn get_mu(&self, bundle: &[f64], good: usize) -> Result<f64, PreferenceError<E>> {
        let ep = self.calc_config.epsilon;
        let mut increase = bundle.to_vec();
        let mut decrease = bundle.to_vec();
        increase[good] += ep;
        decrease[good] -= ep;
        Ok((self.get_utility(&increase)? - self.get_utility(&decrease)?) / (2.0 * ep))
    }

    pub fn get_mrs(
        &self,
        bundle: &[f64],
        good_i: usize,
        good_j: usize,
    ) -> Result<f64, PreferenceError<E>> {
        let mu_j = self.get_mu(bundle, good_j)?;
        if mu_j.abs() < self.calc_config.tolerance {
            return Err(PreferenceError::config(format!(
                "MRS undefined: MU of good {} is zero at {:?}",
                good_j, bundle
            )));
        }
        Ok(self.get_mu(bundle, good_i)? / mu_j)
    }

    fn validate_rationality(&self) -> Result<(), PreferenceError<E>> {
        let points = self.generate_samples();
        self.check_continuity(&points)?;
        self.check_monotonicity(&points)?;
        self.check_convexity(&points)?;
        Ok(())
    }

    fn generate_samples(&self) -> Vec<Vec<f64>> {
        let mut points = Vec::new();
        let dims = self.min_bounds.len();

        points.push(
            self.min_bounds
                .iter()
                .zip(&self.max_bounds)
                .map(|(l, h)| (l + h) / 2.0)
                .collect(),
        );
        points.push(self.min_bounds.clone());
        points.push(self.max_bounds.clone());

        for d in 0..dims {
            let mut corner = self.min_bounds.clone();
            corner[d] = self.max_bounds[d];
            points.push(corner);
        }

        if dims > 1 {
            for d in 0..dims {
                let mut corner = self.max_bounds.clone();
                corner[d] = self.min_bounds[d];
                points.push(corner);
            }
        }

        if self.config.samples > 0 {
            self.generate_sobol_samples(&mut points, self.config.samples);
        }

        points
    }

    fn generate_sobol_samples(&self, points: &mut Vec<Vec<f64>>, n: usize) {
        let dims = self.min_bounds.len();

        if dims > 0 && dims <= 65535 {
            for i in 0..n {
                let mut p = Vec::with_capacity(dims);
                for d in 0..dims {
                    let min = self.min_bounds[d];
                    let max = self.max_bounds[d];
                    let sobol_val = sobol_burley::sample(i as u32, d as u32, self.config.seed);
                    p.push(min + sobol_val as f64 * (max - min));
                }
                points.push(p);
            }
        }
    }

    fn check_continuity(&self, points: &[Vec<f64>]) -> Result<(), PreferenceError<E>> {
        for p in points {
            let u_start = self.get_utility(p)?;

            for i in 0..p.len() {
                let mut p_tiny = p.clone();
                if p_tiny[i] + self.config.epsilon <= self.max_bounds[i] {
                    p_tiny[i] += self.config.epsilon;
                } else if p_tiny[i] - self.config.epsilon >= self.min_bounds[i] {
                    p_tiny[i] -= self.config.epsilon;
                } else {
                    continue;
                }

                let u_end = self.get_utility(&p_tiny)?;
                if (u_end - u_start).abs() > self.config.continuity_threshold {
                    return Err(PreferenceError::config(format!(
                        "Continuity Violated: Detected a jump in the utility function \
                        at index {} near {:?}",
                        i, p
                    )));
                }
            }
        }
        Ok(())
    }

    fn check_monotonicity(&self, points: &[Vec<f64>]) -> Result<(), PreferenceError<E>> {
        for test_point in points {
            let u_base = self.get_utility(test_point)?;

            for i in 0..test_point.len() {
                let mut p_plus = test_point.clone();
                if p_plus[i] + self.config.epsilon <= self.max_bounds[i] {
                    p_plus[i] += self.config.epsilon;
                    let u_plus = self.get_utility(&p_plus)?;

                    if self.config.strict_monotonicity {
                        if u_plus <= u_base + self.config.tolerance {
                            return Err(PreferenceError::config(format!(
                                "Strict Monotonicity Violated: Utility failed to \
                                strictly increase at index {} near {:?}",
                                i, test_point
                            )));
                        }
                    } else if u_plus < u_base - self.config.tolerance {
                        return Err(PreferenceError::config(format!(
                            "Weak Monotonicity Violated: Utility decreased at \
                            index {} near {:?}",
                            i, test_point
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    fn check_convexity(&self, points: &[Vec<f64>]) -> Result<(), PreferenceError<E>> {
        let len = points.len();
        for i in 0..(len / 2) {
            let a = &points[i];
            let b = &points[len - 1 - i];

            let u_a = self.get_utility(a)?;
            let u_b = self.get_utility(b)?;
            let midpoint: Vec<f64> = a.iter().zip(b).map(|(x, y)| (x + y) / 2.0).collect();
            let u_mid = self.get_utility(&midpoint)?;
            let worst_utility = u_a.min(u_b);

            if self.config.strict_convexity {
                let distance_squared: f64 = a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum();
                if distance_squared > self.config.epsilon
                    && u_mid <= worst_utility + self.config.tolerance
                {
                    return Err(PreferenceError::config(format!(
                        "Strict Convexity Violated: Averages not strictly \
                        preferred to extremes between {:?} and {:?}",
                        a, b
                    )));
                }
            } else if u_mid < worst_utility - self.config.tolerance {
                return Err(PreferenceError::config(format!(
                    "Weak Convexity Violated: Preference for extremes detected \
                    between {:?} and {:?}",
                    a, b
                )));
            }
        }
        Ok(())
    }
}

fn validate_bounds(min_bounds: &[f64], max_bounds: &[f64]) -> Result<(), String> {
    if min_bounds.is_empty() || min_bounds.len() != max_bounds.len() {
        return Err("Bounds must be non-empty and of the same length".into());
    }

    for (min, max) in min_bounds.iter().zip(max_bounds) {
        if min > max {
            return Err("max_bounds must be greater than or equal to min_bounds".into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // A valid, simple linear utility function: U(x, y) = x + y
    fn valid_linear_utility(bundle: &[f64]) -> f64 {
        bundle.iter().sum()
    }

    // A valid Cobb-Douglas utility function: U(x, y) = x^0.5 * y^0.5
    fn valid_cobb_douglas(bundle: &[f64]) -> f64 {
        bundle.iter().product::<f64>().sqrt()
    }

    // A function that fails monotonicity (downward sloping)
    fn invalid_monotonicity_utility(bundle: &[f64]) -> f64 {
        -(bundle[0] + bundle[1]) // Utility drops as consumption increases
    }

    // A function that fails convexity (concave indifference curves like U = x^2 + y^2)
    fn invalid_convexity_utility(bundle: &[f64]) -> f64 {
        bundle[0].powi(2) + bundle[1].powi(2)
    }

    // A function that fails continuity (a jump in the middle)
    fn invalid_continuity_utility(bundle: &[f64]) -> f64 {
        let base = bundle[0] + bundle[1];
        if bundle[0] > 5.0 { base + 100.0 } else { base }
    }

    #[test]
    fn test_new_valid_bounds_returns_ok() {
        let min_bounds = vec![0.0, 0.0];
        let max_bounds = vec![10.0, 10.0];
        let pref = Preference::new(valid_linear_utility, min_bounds, max_bounds);

        assert!(
            pref.is_ok(),
            "Failed to create valid preference with defaults"
        );
    }

    #[test]
    fn test_new_mismatched_bounds_raises_err() {
        let min_bounds = vec![0.0, 0.0];
        let max_bounds = vec![10.0];
        let pref = Preference::new(valid_linear_utility, min_bounds, max_bounds);

        assert!(
            pref.is_err(),
            "Should fail when bounds lengths are different"
        );
        if let Err(e) = pref {
            assert!(e.contains("same length"));
        }
    }

    #[test]
    fn test_new_min_greater_than_max_raises_err() {
        let min_bounds = vec![10.0, 0.0];
        let max_bounds = vec![0.0, 10.0];
        let pref = Preference::new(valid_linear_utility, min_bounds, max_bounds);

        assert!(pref.is_err(), "Should fail when min bound > max bound");
    }

    #[test]
    fn test_new_invalid_monotonicity_utility_raises_err() {
        let min_bounds = vec![0.0, 0.0];
        let max_bounds = vec![10.0, 10.0];
        let pref = Preference::new(invalid_monotonicity_utility, min_bounds, max_bounds);

        assert!(pref.is_err());
        if let Err(e) = pref {
            assert!(e.contains("Monotonicity Violated"));
        }
    }

    #[test]
    fn test_new_invalid_convexity_utility_raises_err() {
        let min_bounds = vec![0.0, 0.0];
        let max_bounds = vec![10.0, 10.0];
        let pref = Preference::new(invalid_convexity_utility, min_bounds, max_bounds);

        assert!(pref.is_err());
        if let Err(e) = pref {
            assert!(e.contains("Convexity Violated"));
        }
    }

    #[test]
    fn test_new_invalid_continuity_utility_raises_err() {
        let min_bounds = vec![0.0, 0.0];
        let max_bounds = vec![10.0, 10.0];

        let pref = Preference::new(invalid_continuity_utility, min_bounds, max_bounds);

        assert!(pref.is_err());
        if let Err(e) = pref {
            assert!(
                e.contains("Continuity Violated"),
                "Expected Continuity Violated, got: {}",
                e
            );
        }
    }

    #[test]
    fn test_with_config_strict_axioms_returns_ok() {
        // Restrict away from 0 to avoid Cobb-Douglas edge behaviour when either one of
        // the goods in the bundle is set to 0 (relates more to mathematical behaviour
        // of the function than violation of rationality axioms)
        let min_bounds = vec![0.1, 0.1];
        let max_bounds = vec![10.0, 10.0];

        let config = ValidationConfig {
            strict_monotonicity: true,
            strict_convexity: true,
            ..ValidationConfig::default()
        };

        let pref = Preference::with_config(
            valid_cobb_douglas,
            min_bounds,
            max_bounds,
            config,
            CalcConfig::default(),
        );

        if let Err(e) = &pref {
            println!("Strict Axioms Failed With: {}", e);
        }
        assert!(pref.is_ok(), "Strict axioms failed on a valid function");
    }

    #[test]
    fn test_with_config_validation_disabled_invalid_utility_returns_ok() {
        let min_bounds = vec![0.0, 0.0];
        let max_bounds = vec![10.0, 10.0];

        let config = ValidationConfig {
            validate: false,
            ..ValidationConfig::default()
        };

        let pref = Preference::with_config(
            invalid_monotonicity_utility,
            min_bounds,
            max_bounds,
            config,
            CalcConfig::default(),
        );

        assert!(
            pref.is_ok(),
            "Should succeed when validation is disabled, even for an invalid utility function"
        );
    }

    #[test]
    fn test_with_config_custom_seed_valid_utility_returns_ok() {
        let min_bounds = vec![0.0, 0.0];
        let max_bounds = vec![10.0, 10.0];

        let config = ValidationConfig {
            seed: 42,
            samples: 100,
            ..ValidationConfig::default()
        };

        let pref = Preference::with_config(
            valid_linear_utility,
            min_bounds,
            max_bounds,
            config,
            CalcConfig::default(),
        );

        assert!(pref.is_ok(), "Should succeed with a custom Sobol seed");
    }

    #[test]
    fn test_get_mu_linear_utility_returns_expected_val() {
        // For U(x, y) = x + y, MU of any good is always 1.0
        let pref = Preference::new(valid_linear_utility, vec![0.0, 0.0], vec![10.0, 10.0]).unwrap();

        let bundle = vec![5.0, 5.0];
        let mu = pref.get_mu(&bundle, 0);
        assert!((mu - 1.0).abs() < 1e-5, "Expected MU ~= 1.0, got {}", mu);
    }

    #[test]
    fn test_get_mrs_linear_utility_returns_expected_val() {
        // For U(x, y) = x + y, MRS = MU_x / MU_y = 1 / 1 = 1.0
        let pref = Preference::new(valid_linear_utility, vec![0.0, 0.0], vec![10.0, 10.0]).unwrap();

        let bundle = vec![5.0, 5.0];
        let mrs = pref.get_mrs(&bundle, 0, 1).unwrap();
        assert!((mrs - 1.0).abs() < 1e-5, "Expected MRS ~= 1.0, got {}", mrs);
    }

    #[test]
    fn test_get_mrs_zero_mu_denominator_raises_err() {
        // U(x, y) = x only - MU of good 1 (y) is always 0
        let pref =
            Preference::new(|bundle: &[f64]| bundle[0], vec![0.0, 0.0], vec![10.0, 10.0]).unwrap();

        let bundle = vec![5.0, 5.0];
        let result = pref.get_mrs(&bundle, 0, 1);
        assert!(
            result.is_err(),
            "Expected error when MU of denominator good is zero"
        );
    }

    #[test]
    fn test_standard_config_set_updates_default_constructor_behaviour() {
        let original = StandardConfig::get();
        let mut standard = original.clone();
        standard.preference.samples = 100;
        standard.preference.seed = 42;
        standard.preference.calc_epsilon = 1e-5;
        StandardConfig::set(standard);

        let pref = Preference::with_validation(
            valid_linear_utility,
            vec![0.0, 0.0],
            vec![10.0, 10.0],
            false,
        );

        StandardConfig::set(original);

        assert!(
            pref.is_ok(),
            "Expected global standard config to remain usable"
        );
    }
}
