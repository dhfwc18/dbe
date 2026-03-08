import pytest

from pydbe.econ import cashflow


def test_cal_pv_with_std_inputs_return_expected_val():
    out = cashflow.calculate_pv(100.00, 0.12, 1.0)
    assert out == pytest.approx(89.2857, abs=0.1)


def test_cal_pv_from_cf_with_std_inputs_return_expected_val():
    _in = [100.00, 100.00]
    out = cashflow.calculate_pv_from_cashflows(_in, 0.12)
    assert out == pytest.approx(189.2857, abs=0.1)


def test_pv_unispread_with_init_returns_expected_val():
    out = cashflow.calculate_pv_unispread(100.00, 2, 0.12, True)
    assert out == pytest.approx(29.6348, abs=0.1)


def test_pv_unispread_without_init_returns_expected_val():
    out = cashflow.calculate_pv_unispread(100.00, 2, 0.12, False)
    assert out == pytest.approx(42.1159, abs=0.1)
