# Dutch Home Calcs — NTA 8800 Home Calculator

A transparent, offline-capable homeowner tool for exploring Dutch building-energy performance and preparing a reviewable input pack for an EP adviser.

## What it is

This repository contains a **screening calculator**, not attested BRL 9501 energy-label software. It intentionally shows its assumptions instead of hiding them behind a single label number.

The current UI covers:

- building area, volume, construction year and internal gains;
- opaque envelope surfaces with area/U-value;
- windows/openings with U-value and g-value;
- ventilation rate and heat recovery;
- air/water heat pumps, gas, direct electric, district heat and biomass as screening sources;
- seasonal COP/efficiency and supply-temperature context;
- DHW demand and system efficiency;
- lighting, auxiliary electricity and cooling;
- PV capacity, yield and self-consumption;
- published EP2 label boundaries;
- WWS energy-point cross-check for multifamily dwellings;
- product presets for the Vincent V45 and WeHeat Flint P40 BCRG declarations;
- calculation trace, JSON export and print-ready advisor report;
- offline browser use via a service worker.

## Why it does not claim to be an official label calculator

RVO states that official Dutch energy labels are determined using NTA 8800 and attested calculation software under BRL 9501, with registration by a qualified EP adviser. A browser estimator cannot honestly claim equivalence without a conformance test suite against an attested package.

The project therefore distinguishes three levels:

1. **Exact public-reference checks** — values published by RVO/Huurcommissie are tested exactly.
2. **Engineering checks** — units, monotonicity and conservation relationships are tested.
3. **Official conformance** — intentionally marked unverified until machine-readable reference projects from attested software are available.

This distinction is important: a plausible label estimate is not the same thing as an official label.

## Run locally

No build system is required for the browser tool. Serve the directory over HTTP so the service worker can operate:

```bash
python3 -m http.server 8080
```

Then open `http://localhost:8080/`.

Tests:

```bash
npm test
```

## Current project scenario

`data/pawel-apartment.json` contains the current working scenario for the 51 m² top-floor apartment discussed in the project.

`reports/pawel-scenario-2026-09-06.md` records the run and explicitly separates known facts from provisional assumptions.

## Key result from the current screening run

Using the same provisional envelope assumptions for every scenario:

- generic heat pump, COP 3.50 → **A+ screening**;
- Vincent preset, η≈4.35 → **A+ screening**;
- Flint P40 preset, η=5.909 → **A++ screening**;
- Flint preset + 2.0 kWp PV → **A+++ screening**.

With a hypothetical non-energy WWS base of 134 points, the last case gives 134 + 53 = **187 WWS points**.

These are **screening results, not the apartment's verified official label**. The wall/floor geometry, ventilation, thermal bridges, exact product declaration interpolation/applicability, current NTA defaults and other inputs still need to be checked.

## Sources

- RVO — Information for EP advisers: https://www.rvo.nl/onderwerpen/wetten-en-regels-gebouwen/informatie-epa
- RVO — Energy labels for homes: https://www.rvo.nl/onderwerpen/wetten-en-regels-gebouwen/energielabel-woningen
- RVO — Example homes 2022 existing construction: https://www.rvo.nl/sites/default/files/2023-01/brochure-voorbeeldwoningen-bestaande-bouw-2022.pdf
- Huurcommissie — WWS for independent dwellings: https://www.huurcommissie.nl/support/beleidsboeken/waarderingsstelsel-zelfstandige-woonruimte/algemene-toelichting
- BCRG — Vincent declaration: https://mijn.bcrg.nl/media/20220186GK_gelijkwaardigheidsverklaring_Vincent_V45-Combi__06-05-22.pdf
- BCRG — Flint P40 declaration: https://mijn.bcrg.nl/media/documents/2024/GK/20240324GK.pdf
- BCRG — Flint P40 + WBL-200 declaration: https://mijn.bcrg.nl/media/documents/2025/GK/20250226GK.pdf
- OpenAEC NTA reference crates: https://github.com/OpenAEC-Foundation/crates-warehouse
