// src/econ/cashflow.rs
#[inline]
pub fn cal_pv(val: f64, rate: f64, step: f64) -> f64 {
    assert!(rate > -1.0, "Discount rate must be greater than -100%");
    assert!(step >= 0.0, "Step should generally be non-negative");

    val / (1.0 + rate).powf(step)
}

pub fn cal_pv_from_cf(cf: &[f64], rate: f64) -> f64 {
    if rate <= -1.0 {
        return f64::NAN;
    }

    let discount_multiplier = 1.0 / (1.0 + rate);
    let mut current_discount = 1.0;

    let mut total_pv = 0.0;
    for &val in cf {
        total_pv += val * current_discount;
        current_discount *= discount_multiplier;
    }

    total_pv
}

/// Spreading a target PV across selected future steps.
///
/// This function takes a target PV and returns the uniform nominal value that needs to
/// be achieved at each timestep in order to aggregate to the target PV.
///
/// # Arguments
/// * `t_pv` - The target present value to be achieved
/// * `t_steps` - Number of steps (excluding step 0) into the future in which the target
///   is expected to be achieved
/// * `rate` - The discount rate against which the cashflow is evaluated at
/// * `w_init` - Whether to account for an undiscounted year 0 value
pub fn pv_unispread(t_pv: f64, t_steps: i32, rate: f64, w_init: bool) -> f64 {
    assert!(t_steps >= 0, "Steps must be non-negative");
    if t_steps < 0 || rate <= -1.0 {
        return f64::NAN;
    }

    let start_idx = if w_init { 0 } else { 1 };

    if start_idx > t_steps {
        return f64::NAN;
    }

    let base = 1.0 + rate;
    let mut current_factor = base.powi(start_idx);
    let mut fact = 0.0;

    for _ in start_idx..=t_steps {
        fact += current_factor;
        current_factor *= base;
    }

    t_pv / fact
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cal_pv_with_std_inputs_returns_expected_val() {
        let out = cal_pv(100.00, 0.12, 1.0);
        assert!(
            (out - 89.2857).abs() < 0.1,
            "Result: {out}, Expected: 89.2857",
        );
    }

    #[test]
    fn test_cal_pv_from_cf_with_std_inputs_returns_expected_val() {
        let _in: [f64; 2] = [100.00, 100.00];
        let out = cal_pv_from_cf(&_in, 0.12);
        assert!(
            (out - 189.2857).abs() < 0.1,
            "Result: {out}, Expected: 189.2857",
        );
    }

    #[test]
    fn test_pv_unispread_with_init_returns_expected_val() {
        let out = pv_unispread(100.00, 2, 0.12, true);
        assert!(
            (out - 29.6348).abs() < 0.1,
            "Result: {out}, Expected: 29.6348",
        );
    }

    #[test]
    fn test_pv_unispread_without_init_returns_expected_val() {
        let out = pv_unispread(100.00, 2, 0.12, false);
        assert!(
            (out - 42.1159).abs() < 0.1,
            "Result: {out}, Expected: 42.1159",
        );
    }

    #[test]
    #[should_panic(expected = "Discount rate must be greater than -100%")]
    fn test_cal_pv_with_invalid_rate_panics() {
        let _ = cal_pv(100.0, -1.5, 1.0);
    }

    #[test]
    #[should_panic(expected = "Step should generally be non-negative")]
    fn test_cal_pv_with_negative_step_panics() {
        let _ = cal_pv(100.0, 0.12, -1.0);
    }

    #[test]
    fn test_cal_pv_from_cf_with_invalid_rate_returns_nan() {
        let _in: [f64; 2] = [100.00, 100.00];
        let out = cal_pv_from_cf(&_in, -1.5);
        assert!(out.is_nan());
    }

    #[test]
    fn test_pv_unispread_with_invalid_rate_returns_nan() {
        let out = pv_unispread(100.0, 2, -1.5, true);
        assert!(out.is_nan());
    }

    #[test]
    #[should_panic(expected = "Steps must be non-negative")]
    fn test_pv_unispread_with_negative_steps_panics() {
        let _ = pv_unispread(100.0, -2, 0.12, true);
    }

    #[test]
    fn test_pv_unispread_with_start_idx_greater_than_t_steps_returns_nan() {
        // start_idx is 1 (w_init is false), t_steps is 0. start_idx > t_steps.
        let out = pv_unispread(100.0, 0, 0.12, false);
        assert!(out.is_nan());
    }
}
