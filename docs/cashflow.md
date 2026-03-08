# Cashflow Module
Provides functionalities for basic cashflow analysis.

## Present value calculation for cashflow at a selected future time step `cal_pv`

$$
PV = \frac{cashflow_t}{(1 + rate)^{t}}
$$

Where $cashflow_t$ is the cashflow at time $t$, $t$ is the future timestep at which the
evaluation takes place, and $rate$ is the rate at which the future cashflows are
discounted.

## Multi-period discounted cashflow analysis `cal_pv_from_cf`

$$
PV = \sum_{t=0}^{T}{\frac{cashflow_t}{(1 + rate)^{t}}}
$$

Where $T$ is the total period of future timesteps to be accounted for.

## Nominal uniform cashflow to achieve a target present value in time `pv_unispread`

$$
Cashflow = PV_{target} \times \left({\sum_{t=\{0,\,1\}}^{T}{\frac{1}{(1 + rate)^{t}}}}
\right)^{-1}
$$
