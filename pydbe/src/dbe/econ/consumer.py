from __future__ import annotations

from collections.abc import Callable

from dbe._core import PyPreference, rs_optimal_bundle, rs_trace_2d


class Preference:
    """
    Represents a consumer's preference over bundles of goods.

    Validates the Axioms of Rationality (continuity, monotonicity, convexity)
    on construction using Sobol quasi-random sampling over the given bounds.

    Parameters
    ----------
    utility_func : callable
        A function mapping a list of floats (consumption bundle) to a float
        (utility). Must accept a single list argument.
    min_bounds : list[float]
        Minimum quantity for each good (typically 0).
    max_bounds : list[float]
        Maximum quantity for each good (satiation point).
    samples : int, optional
        Number of Sobol sequence points used for axiom validation.
        Default is 60000.
    seed : int, optional
        Seed for Sobol sequence scrambling. Default is 0.
    strict_monotonicity : bool, optional
        If True, enforces strict monotonicity. Default is False.
    strict_convexity : bool, optional
        If True, enforces strict convexity. Default is False.
    epsilon : float, optional
        Step size for axiom validation perturbations. Default is 1e-6.
    tolerance : float, optional
        Numerical tolerance for axiom validation comparisons. Default is 1e-9.
    continuity_threshold : float, optional
        Maximum permitted utility jump for continuity. Default is 1.0.
    validate : bool, optional
        Whether to validate the Axioms of Rationality on construction.
        Default is True.
    calc_epsilon : float, optional
        Step size for numerical differentiation (MU, MRS). Default is 1e-7.
    calc_tolerance : float, optional
        Tolerance for numerical comparisons in MU/MRS calculations.
        Default is 1e-9.

    Raises
    ------
    ValueError
        If bounds are invalid or any rationality axiom is violated.
    """

    def __init__(
        self,
        utility_func: Callable[[list[float]], float],
        min_bounds: list[float],
        max_bounds: list[float],
        *,
        samples: int = 60_000,
        seed: int = 0,
        strict_monotonicity: bool = False,
        strict_convexity: bool = False,
        epsilon: float = 1e-6,
        tolerance: float = 1e-9,
        continuity_threshold: float = 1.0,
        validate: bool = True,
        calc_epsilon: float = 1e-7,
        calc_tolerance: float = 1e-9,
    ) -> None:
        self._inner = PyPreference(
            utility_func,
            min_bounds,
            max_bounds,
            samples=samples,
            seed=seed,
            strict_monotonicity=strict_monotonicity,
            strict_convexity=strict_convexity,
            epsilon=epsilon,
            tolerance=tolerance,
            continuity_threshold=continuity_threshold,
            validate=validate,
            calc_epsilon=calc_epsilon,
            calc_tolerance=calc_tolerance,
        )

    def get_utility(self, bundle: list[float]) -> float:
        """
        Evaluate the utility function at the given bundle.

        Parameters
        ----------
        bundle : list[float]
            Consumption bundle, one value per good.

        Returns
        -------
        float
            Utility of the bundle.
        """
        return self._inner.get_utility(bundle)

    def get_mu(self, bundle: list[float], good: int) -> float:
        """
        Compute the marginal utility of a specific good via central differences.

        Parameters
        ----------
        bundle : list[float]
            Consumption bundle, one value per good.
        good : int
            Index of the good to differentiate with respect to.

        Returns
        -------
        float
            Marginal utility of the specified good at the given bundle.
        """
        return self._inner.get_mu(bundle, good)

    def get_mrs(self, bundle: list[float], good_i: int, good_j: int) -> float:
        """
        Compute the marginal rate of substitution between good_i and good_j.

        The MRS is defined as MU(good_i) / MU(good_j), holding all other
        goods fixed.

        Parameters
        ----------
        bundle : list[float]
            Consumption bundle, one value per good.
        good_i : int
            Index of the good in the numerator.
        good_j : int
            Index of the good in the denominator.

        Returns
        -------
        float
            MRS of good_i with respect to good_j.

        Raises
        ------
        ValueError
            If the marginal utility of good_j is zero at the given bundle.
        """
        return self._inner.get_mrs(bundle, good_i, good_j)


def optimal_bundle(
    pref: Preference,
    prices: list[float],
    income: float,
    *,
    mu_init: float = 1.0,
    mu_decay: float = 0.1,
    outer_iters: int = 10,
    inner_iters: int = 500,
    step_size: float = 1e-2,
    tol: float = 1e-8,
) -> list[float]:
    """
    Find the consumption bundle that maximises utility subject to a budget
    constraint.

    Uses an interior-point log-barrier method with backtracking line search.

    Parameters
    ----------
    pref : Preference
        A validated consumer preference.
    prices : list[float]
        Price for each good. Must match the number of goods in `pref`.
    income : float
        Total income available to the consumer. Must be positive.
    mu_init : float, optional
        Initial barrier weight. Default is 1.0.
    mu_decay : float, optional
        Multiplicative decay applied to the barrier weight per outer
        iteration. Default is 0.1.
    outer_iters : int, optional
        Number of outer iterations (barrier weight reductions). Default is 10.
    inner_iters : int, optional
        Number of gradient ascent steps per outer iteration. Default is 500.
    step_size : float, optional
        Initial step size for gradient ascent. Default is 0.01.
    tol : float, optional
        Convergence tolerance on the gradient norm. Default is 1e-8.

    Returns
    -------
    list[float]
        The optimal consumption bundle.

    Raises
    ------
    ValueError
        If prices or income are invalid, or no feasible starting point exists.
    """
    return rs_optimal_bundle(
        pref._inner,
        prices,
        income,
        mu_init=mu_init,
        mu_decay=mu_decay,
        outer_iters=outer_iters,
        inner_iters=inner_iters,
        step_size=step_size,
        tol=tol,
    )


def trace_indifference_curve(
    pref: Preference,
    target_utility: float,
    good_i: int,
    good_j: int,
    *,
    n_points: int = 200,
    tol: float = 1e-10,
) -> list[tuple[float, float]]:
    """
    Trace an indifference curve in the plane of two goods.

    Grids good_i across its bounds and for each value uses bisection to find
    the value of good_j such that U(x) = target_utility. All other goods are
    held fixed at the midpoint of their bounds.

    Parameters
    ----------
    pref : Preference
        A validated consumer preference.
    target_utility : float
        The utility level to trace.
    good_i : int
        Index of the good on the x-axis (gridded).
    good_j : int
        Index of the good on the y-axis (solved via bisection).
    n_points : int, optional
        Number of points to trace along the curve. Default is 200.
    tol : float, optional
        Bisection tolerance for finding each point. Default is 1e-10.

    Returns
    -------
    list[tuple[float, float]]
        List of (x_i, x_j) coordinate pairs lying on the indifference curve.

    Raises
    ------
    ValueError
        If good indices are invalid or the target utility is unreachable
        within the given bounds.
    """
    return rs_trace_2d(
        pref._inner,
        target_utility,
        good_i,
        good_j,
        n_points=n_points,
        tol=tol,
    )
