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
