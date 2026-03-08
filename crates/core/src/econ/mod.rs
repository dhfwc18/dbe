pub mod cashflow;

#[cfg(test)]
mod tests {
    use super::cashflow;

    #[test]
    fn test_cal_pv_with_std_inputs_return_expected_val() {
        let out = cashflow::cal_pv(100.00, 0.12, 1.0);
        assert!(
            (out - 89.2857).abs() < 0.1,
            "Result: {out}, Expected: 89.2857",
        );
    }

    #[test]
    fn test_cal_pv_from_cf_with_std_inputs_return_expected_val() {
        let _in: [f64; 2] = [100.00, 100.00];
        let out = cashflow::cal_pv_from_cf(&_in, 0.12);
        assert!(
            (out - 189.2857).abs() < 0.1,
            "Result: {out}, Expected: 189.2857",
        );
    }

    #[test]
    fn test_pv_unispread_with_init_returns_expected_val() {
        let out = cashflow::pv_unispread(100.00, 2, 0.12, true);
        assert!(
            (out - 29.6348).abs() < 0.1,
            "Result: {out}, Expected: 29.6348",
        );
    }

    #[test]
    fn test_pv_unispread_without_init_returns_expected_val() {
        let out = cashflow::pv_unispread(100.00, 2, 0.12, false);
        assert!(
            (out - 42.1159).abs() < 0.1,
            "Result: {out}, Expected: 42.1159",
        );
    }
}
