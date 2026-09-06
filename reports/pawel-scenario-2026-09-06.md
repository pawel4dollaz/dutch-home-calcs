# Pawel apartment — screening calculation report

**Date:** 2026-09-06  
**Model:** NTA 8800 Home Calculator, transparent screening engine  
**Important:** this is not an official registered energy label.

## Known inputs

- 51 m² apartment
- construction year 1989
- top floor; roof/ceiling area exposed
- roof Rc = 2.5 m²K/W → U = 0.400 W/m²K
- HR++ glazing
- 5 cm floor insulation (screening U assumed 0.50 W/m²K; must be replaced by documented construction value)
- simple exhaust ventilation; no WTW
- heating by air/water heat pump in the scenarios below
- 3 radiators + bathroom underfloor heating
- no PV in the base comparison
- DHW: 2 occupants, 35 L/person/day

## Critical provisional assumptions

The following are **not established NTA inputs** and materially affect the result:

- external-wall area/U-value
- actual window area/U-value/g-value
- heated volume
- infiltration/ventilation defaults and actual flows
- thermal bridges
- exact adjacent-zone conditions
- monthly solar/shading factors
- exact heat-pump declaration interpolation and applicability
- DHW system configuration and losses
- current NTA primary-energy factors and all subsystem defaults

The model therefore reports an indication, not an official label.

## Scenario results

Using the same envelope and non-heating assumptions for all cases:

| Scenario | Heat-source screening input | EP2 screening | Indicative label | WWS energy points | Total if non-energy base = 134 |
|---|---:|---:|---|---:|---:|
| Generic heat pump | COP 3.50 | 88.0 | A+ | 43 | 177 |
| Vincent V45 | η ≈ 4.35 at ≤30°C, low-energy screening point | 77.3 | A+ | 43 | 177 |
| Flint P40 | η = 5.909 at ≤30°C, low-energy screening point | 65.7 | A++ | 48 | 182 |
| Flint P40 + 2.0 kWp PV | same | 47.7 | A+++ | 53 | **187** |

The last row is particularly relevant to the 187-point question: **under this screening model**, 2 kWp PV plus the Flint preset moves the model from A++ to A+++, which adds five WWS energy points and would take a hypothetical 134-point non-energy base to 187.

That does **not** prove that 2 kWp is the official PV requirement for the apartment. The official result depends on the complete NTA calculation, roof/PV assumptions, evidence and the advisor's attested software.

## Interpretation for Vincent vs Flint

The model shows why the product-declaration question matters. A heat pump with a higher declared generation efficiency can materially reduce the calculated primary-fossil score. But the Vincent declaration is from 2022 and explicitly targets NTA 8800:2020+A1:2020, while the current official workflow is based on the current NTA/software rules. BCRG's current policy also describes transition handling for older declarations. The decisive question is therefore whether the current attested software accepts BCRG declaration **20220186GG** for the specific new registration/relabeling case.

## WWS cross-check

For a multifamily dwelling, Huurcommissie currently assigns:

- A++++: 58 points
- A+++: 53 points
- A++: 48 points
- A+: 43 points
- A: 37 points
- B: 30 points
- C: 15 points
- D: 11 points

Therefore A++ → A+++ is exactly +5 points. If the apartment's non-energy WWS score really is 134, then A+++ would be 187. The 134 value is an input assumption here, not a calculated fact.

## Verification status

Automated tests: **4/4 passed**.

Exact public-reference tests cover the published EP2 label boundaries and simple physical/monotonicity checks. No claim of exact agreement with an attested EP package is made because no machine-readable attested reference project was available for this run.

## Recommended next verification

Give the advisor the exported JSON and ask them to return the same scenario from their attested software with:

1. EP1 / energy demand;
2. EP2 / primary fossil energy;
3. EP3 / renewable share;
4. label class;
5. heat-pump declaration identifier actually selected;
6. software version and NTA version;
7. PV assumptions;
8. the calculated WWS energy points.

A future fixture can then be added to this repository and tested automatically to numerical tolerance.
