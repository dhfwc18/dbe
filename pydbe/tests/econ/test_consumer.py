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


def test_get_mu_with_out_of_bounds_good_raises_index_error():
    pref = consumer.Preference(linear_utility, [0.0, 0.0], [10.0, 10.0])
    with pytest.raises(IndexError):
        pref.get_mu([5.0, 5.0], 3)


def test_preference_with_raising_callback_raises_python_error():
    def raising_utility(bundle: list[float]) -> float:
        raise RuntimeError("utility failure")

    with pytest.raises(RuntimeError, match="utility failure"):
        consumer.Preference(raising_utility, [0.0, 0.0], [10.0, 10.0])


def test_optimal_bundle_with_raising_callback_raises_python_error():
    def raising_utility(bundle: list[float]) -> float:
        raise RuntimeError("utility failure")

    pref = consumer.Preference(
        raising_utility,
        [0.0, 0.0],
        [10.0, 10.0],
        validate=False,
    )

    with pytest.raises(RuntimeError, match="utility failure"):
        consumer.optimal_bundle(pref, [1.0, 1.0], 10.0)


def test_optimal_bundle_with_std_inputs_return_expected_val():
    pref = consumer.Preference(cobb_douglas_utility, [0.0, 0.0], [20.0, 20.0])
    out = consumer.optimal_bundle(pref, [1.0, 1.0], 10.0)
    assert out[0] == pytest.approx(5.0, abs=0.1)
    assert out[1] == pytest.approx(5.0, abs=0.1)


def test_trace_indifference_curve_with_std_inputs_return_expected_val():
    pref = consumer.Preference(cobb_douglas_utility, [0.1, 0.1], [10.0, 10.0])
    out = consumer.trace_indifference_curve(pref, 2.0, 0, 1, n_points=50)
    assert len(out) >= 40


def test_standard_config_with_std_inputs_return_expected_val():
    original = consumer.Config.state()
    consumer.Config.Preference.set(samples=100, seed=42, calc_epsilon=1e-5)
    consumer.Config.Indifference.set(n_points=75)
    consumer.Config.Optimization.set(outer_iters=12)
    out = consumer.Config.state()

    consumer.Config.restore_defaults()
    consumer.Config.set(**original)

    assert out["preference"]["samples"] == 100
    assert out["preference"]["seed"] == 42
    assert out["preference"]["calc_epsilon"] == pytest.approx(1e-5, abs=1e-12)
    assert out["indifference"]["n_points"] == 75
    assert out["optimization"]["outer_iters"] == 12
