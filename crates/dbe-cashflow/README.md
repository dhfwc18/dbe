# Cashflow Crate
Provides functionalities for basic cashflow analysis.

## Present Value Calculations

### Single Value Calculation

The `cal_pv` function handles single value calculation. It allows the calculation of
cashflow at time $t$, where $t$ is some timestep in the present ($t = 0$) or future.

```math
PV = \frac{cashflow_t}{(1 + rate)^{t}}
```

**VARIABLES**
- $PV$ is the present value of the evaluated cashflow at the present ($t = 0$)
- $cashflow_t$ is the cashflow at timestep $t$
- $rate$ is the discount rate applied when evaluating future cashflow

## Multi-period Discounted Cashflow Analysis

The `cal_pv_from_cf` function allows the user to evaluate the present value of a series
of cashflow.


```math
PV = \sum_{t=0}^{T}{\frac{cashflow_t}{(1 + rate)^{t}}}
```



**VARIABLES**
- $PV$ is the total present value of the cashflow series evaluated at $t = 0$
- $T$ is the total period covered by the analysis -- it is important to note that the
  analysis is zero indexed, meaning that the first element of the series will be treated
  as the cashflow at $t = 0$
- $rate$ is the fixed discount rate applied to the cashflow across time. Note that the
  discount rate here is assumed to be uniform across time, which is an assumption that
  can potentially be relaxed and expanded upon in the future

## Nominal Uniform Cashflow to Achieve a Target Present Value

A common question to ask in the policy sphere is what is the fixed nominal payout that
would be required so that the stakeholders are being given the equivalent present value
for a certain present time trade off. The `pv_unispread` function answers this question
by giving user the ability to calculate the fixed uniform cashflow that would need to
occur at each timestep for the cashflow series to achieve a targeted $PV$ at $t = 0$.

```math
cashflow = target\_PV_{t = 0} \times \left({\sum_{t=\{0,\,1\}}^{T}{\frac{1}
{(1 + rate)^{t}}}} \right)^{-1}
```

**VARIABLES**
- $target\_PV_{t = 0}$ is the desired present value to be achieved at $t = 0$
- $cashflow$ is the fix nominal cashflow payout at each period in the payment period $T$

**NOTE**

When calculating the fixed nominal cashflow required, whether a present $t = 0$
undiscounted payout should be included in the calculation as it is in the case of
`cal_pv_from_cf` becomes an important question. For convenience, the `pv_unispread`
function include this as an optional variable where the user can decide whether the
first period should be included.
