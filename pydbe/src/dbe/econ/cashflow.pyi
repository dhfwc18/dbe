def calculate_pv(cashflow: float, discount_rate: float, timestep: float) -> float: ...
def calculate_pv_from_cashflows(
    cashflows: list[float],
    discount_rate: float,
) -> float: ...
def calculate_pv_unispread(
    t_pv: float,
    t_steps: int,
    discount_rate: float,
    w_init: bool = False,
) -> float: ...
