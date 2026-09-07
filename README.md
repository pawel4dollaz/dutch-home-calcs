# Dutch Home Calcs — NTA 8800 Home Calculator

A transparent, offline-capable homeowner tool for exploring Dutch building-energy performance and preparing a reviewable input pack for an EP adviser.

## Status

**Screening software + attestation-preparation framework. Not BRL 9501-attested.**

The screening engine's PV renewable-source ledger was corrected on 2026-09-07 (DH-001). This removes double-counting of self-consumed PV from the screening renewable-share output; it does not implement NTA 8800 EP3 or make an EDR case pass. See `reports/DH-001-PV-ACCOUNTING-2026-09-07.md` and `docs/DEFECT-REGISTER.md`.

The repository now includes an ISSO 54 EDR test manifest, a fail-closed EDR execution harness, and a draft BRL 9501 quality-management system. Missing official reference outputs are explicitly reported as `BLOCKED`; they are never turned into artificial passes.

The current calculation core remains a screening model. It is not yet a complete NTA 8800:2025+C1:2026 implementation and therefore cannot issue an official Dutch energy label.

## EDR / BRL 9501 work

`tests/edr-manifest.json` records the published historical ISSO 54 version 2.0 (May 2022) structure: 277 tests in total, including 219 dwelling tests and 58 utility-building tests. The public document states a historical ±1.0% rejection band. The current 2026 attestation must use the current ISSO 54 version designated with BRL 9501:2026.

`tests/edr-runner.js` executes a representative set of the publicly documented reference-building cases through the current screening kernel. It deliberately reports conformance as `BLOCKED` because the official reference result workbook is a separate controlled document and because the current kernel is not yet complete NTA 8800 software.

`docs/QUALITY-MANUAL.md` is the draft quality system covering requirements traceability, calculation-kernel change control, EDR testing, regression testing, release control, complaints, non-conformities, retention and audit evidence.

`docs/ATTESTATION-READINESS.md` is the BRL 9501 readiness checklist and identifies the remaining technical and organisational gates before an independent attestation application.

Run the ordinary regression tests:

```bash
npm test
```

Run the EDR harness:

```bash
npm run edr
```

## What it is

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
3. **Official conformance** — intentionally unverified until the complete current EDR fixtures have been executed and independently assessed.

This distinction is important: a plausible label estimate is not the same thing as an official label.

## Run locally

No build system is required for the browser tool. Serve the directory over HTTP so the service worker can operate:

```bash
python3 -m http.server 8080
```

Then open `http://localhost:8080/`.

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
- ISSO — Publication 54 historical public EDR document: https://documenten.isso.nl/s/Ym8-N6LW2khaIUJL0XEgRaFsscBPV5dn/ISSO%2054%20-%2012-05-2022.pdf
- OpenAEC NTA reference crates: https://github.com/OpenAEC-Foundation/crates-warehouse
