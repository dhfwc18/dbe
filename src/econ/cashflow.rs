// src/econ/cashflow.rs

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
