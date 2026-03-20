import pytest

from dbe.econ import consumer


def linear_utility(bundle: list[float]) -> float:
    return sum(bundle)


def cobb_douglas_utility(bundle: list[float]) -> float:
    return (bundle[0] * bundle[1]) ** 0.5


def test_preference_with_std_inputs_return_expected_val():
    pref = consumer.Preference(linear_utility, [0.0, 0.0], [10.0, 10.0])
    out = pref.get_utility([2.0, 3.0])
    assert out == pytest.approx(5.0, abs=1e-6)


def test_get_mu_with_std_inputs_return_expected_val():
    pref = consumer.Preference(linear_utility, [0.0, 0.0], [10.0, 10.0])
    out = pref.get_mu([5.0, 5.0], 0)
    assert out == pytest.approx(1.0, abs=1e-5)


def test_optimal_bundle_with_std_inputs_return_expected_val():
    pref = consumer.Preference(cobb_douglas_utility, [0.0, 0.0], [20.0, 20.0])
    out = consumer.optimal_bundle(pref, [1.0, 1.0], 10.0)
    assert out[0] == pytest.approx(5.0, abs=0.1)
    assert out[1] == pytest.approx(5.0, abs=0.1)


def test_trace_indifference_curve_with_std_inputs_return_expected_val():
    pref = consumer.Preference(cobb_douglas_utility, [0.1, 0.1], [10.0, 10.0])
    out = consumer.trace_indifference_curve(pref, 2.0, 0, 1, n_points=50)
    assert len(out) >= 40
