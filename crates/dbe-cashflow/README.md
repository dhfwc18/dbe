# Cashflow Crate

Provides core utilities for basic discounted-cashflow analysis.

## Present Value Calculations

### Single Cashflow

The `cal_pv` function evaluates the present value of a single cashflow observed at
time `t`, where `t = 0` represents the present period.

```math
PV = \frac{cashflow_t}{(1 + rate)^t}
```

**Variables**
- `PV` is the present value of the evaluated cashflow at `t = 0`
- `cashflow_t` is the cashflow received at timestep `t`
- `rate` is the discount rate applied to future cashflows

## Multi-period Discounted Cashflow

The `cal_pv_from_cf` function evaluates the present value of a series of cashflows
using a fixed discount rate.

```math
PV = \sum_{t=0}^{T}{\frac{cashflow_t}{(1 + rate)^t}}
```

**Variables**
- `PV` is the total present value of the cashflow series at `t = 0`
- `T` is the final timestep in the series
- `rate` is the discount rate applied uniformly across the full horizon

The series is zero-indexed. The first element is therefore treated as the cashflow at
`t = 0`.

## Uniform Cashflow Required to Reach a Target Present Value

The `pv_unispread` function solves for a fixed nominal cashflow that produces a target
present value across a payment horizon.

```math
cashflow = target_PV_{t = 0} \times \left(\sum_{t = 0}^{T}{\frac{1}{(1 + rate)^t}}\right)^{-1}
```

**Variables**
- `target_PV_{t = 0}` is the desired present value at `t = 0`
- `cashflow` is the fixed nominal payment made at each timestep in the horizon
- `T` is the final timestep in the payment schedule
- `rate` is the discount rate applied to each payment

## Notes

When calculating a uniform cashflow, an important modelling choice is whether to
include an undiscounted payment at `t = 0`. The `pv_unispread` function exposes this
as an option so the caller can decide whether the first period is included.
