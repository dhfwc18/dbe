// src/econ/cashflow.rs
#[inline]
pub fn cal_pv(val: f64, rate: f64, step: f64) -> f64 {
    val / (1.0 + rate).powf(step)
}

pub fn cal_pv_from_cf(cf: &[f64], rate: f64) -> f64 {
    cf.iter()
        .enumerate()
        .map(|(i, &val)| cal_pv(val, rate, i as f64))
        .sum()
}

/// Spreading a target PV across selected future steps.
///
/// This function takes a target PV and returns the uniform nominal value that needs to
/// be achieved at each timestep in order to aggregate to the target PV.
///
/// # Arguments
/// * `t_pv` - The target present value to be achieved
/// * `t_steps` - Number of steps (excluding step 0) into the future in which the target
///               is expected to be achieved
/// * `rate` - The discount rate against which the cashflow is evaluated at
/// * `w_init` - Whether to account for an undiscounted year 0 value
pub fn pv_unispread(t_pv: f64, t_steps: i32, rate: f64, w_init: bool) -> f64 {
    let mut start_idx: i32 = 0;
    if !w_init {
        start_idx = 1;
    }

    let fact: f64 = (start_idx..=t_steps)
        .map(|t_step| (1.0 + rate).powf(t_step as f64))
        .sum();
    t_pv / fact
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cal_pv_with_std_inputs_return_expected_val() {
        let out = cal_pv(100.00, 0.12, 1.0);
        assert!(
            (out - 89.2857).abs() < 0.1,
            "Result: {out}, Expected: 89.2857",
        );
    }

    #[test]
    fn test_cal_pv_from_cf_with_std_inputs_return_expected_val() {
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
}
