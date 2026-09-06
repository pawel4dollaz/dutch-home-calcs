# Validation and accuracy report

## Executive result

This project deliberately does **not** claim to be an official NTA 8800 energy-label engine. Dutch energy labels must be produced and registered by a qualified EP adviser using attested BRL 9501 calculation software. RVO confirms this workflow and says the registered EP-Online label is authoritative.

The calculator therefore has two goals:

1. make the energy-performance chain understandable and reproducible for a homeowner;
2. make it easy to hand the inputs and assumptions to an EP adviser for a controlled comparison.

A claim of exact agreement with an official label is only accepted here when an independently published reference value exists. Where no public reference value exists, the report records the test as an engineering/logic check rather than an exact NTA conformance test.

## Reference tests

| Test | Reference | Expected | Status |
|---|---|---:|---|
| EP2 A++++ boundary | RVO `Voorbeeldwoningen 2022 bestaande bouw` | EP2 < 0 | PASS |
| EP2 A+++ boundary | same RVO publication | EP2 ≤ 50 kWh/m²·yr | PASS |
| EP2 A++ boundary | same | EP2 ≤ 75 | PASS |
| EP2 A+ boundary | same | EP2 ≤ 105 | PASS |
| EP2 A boundary | same | EP2 ≤ 160 | PASS |
| EP2 B boundary | same | EP2 ≤ 190 | PASS |
| EP2 C boundary | same | EP2 ≤ 250 | PASS |
| EP2 D boundary | same | EP2 ≤ 290 | PASS |
| EP2 E boundary | same | EP2 ≤ 335 | PASS |
| EP2 F boundary | same | EP2 ≤ 380 | PASS |
| Rc 2.5 → U | physical relation U=1/R | 0.400 W/m²K | PASS |
| Envelope sensitivity | conservation/monotonicity check | better U cannot raise demand | PASS |
| PV sensitivity | conservation/monotonicity check | PV cannot raise grid electricity | PASS |

The automated tests are in `tests/engine.test.js` and run with `npm test`.

## Important current-methodology checks

RVO's current adviser guidance states that NTA 8800 is the determining method and that official calculation software is attested under BRL 9501. It also distinguishes a new registration from a relabel: a relabel within 24 months can use the software version of the original registration, whereas new registrations use the current software.

As of the project date (September 2026), the UI therefore labels itself as being in the NTA 8800:2025+C1:2026 context, but it does not pretend that a simplified browser calculation is equivalent to the complete attested calculation.

## Why this is not an “exact NTA” proof

The official method contains many details that cannot safely be inferred from a homeowner's short questionnaire: exact monthly climate data, geometry rules, adjacent-zone treatment, default/evidence rules, ventilation and infiltration defaults, solar and shading calculations, thermal bridges, distribution and emitter corrections, heat-pump generation/interpolation rules, domestic-hot-water system details, cooling, PV and renewable accounting, rounding, and product/declaration applicability.

The current OpenAEC public NTA crates are useful independent open-source reference material, but their own `nta8800-ep` documentation explicitly calls its EP integration a V1 with deliberate simplifications. They are therefore not treated as proof that this browser model is an attested replacement.

## What would constitute exact real-world verification

To upgrade this project from a transparent screening tool to a verified independent implementation, the following test fixture should be added for each release:

1. an anonymised project file exported by an attested EP-W/B or EP-W/D package;
2. the complete input data used by the adviser;
3. the adviser software version and NTA version;
4. the official outputs: EP1/EP2/EP3, renewable share, label class, monthly energy flows, and relevant subsystem results;
5. a machine-readable expected-output fixture;
6. a tolerance policy for every numeric output and an exact-match policy for label class.

Without those fixtures, claiming “matches real-world examples exactly” would be false precision.

## Sources

- RVO — Information for EP advisers: https://www.rvo.nl/onderwerpen/wetten-en-regels-gebouwen/informatie-epa
- RVO — Energy labels for homes: https://www.rvo.nl/onderwerpen/wetten-en-regels-gebouwen/energielabel-woningen
- RVO — Example homes 2022, existing construction: https://www.rvo.nl/sites/default/files/2023-01/brochure-voorbeeldwoningen-bestaande-bouw-2022.pdf
- Huurcommissie — WWS energy-performance points: https://www.huurcommissie.nl/support/beleidsboeken/waarderingsstelsel-zelfstandige-woonruimte/algemene-toelichting
- OpenAEC crates warehouse: https://github.com/OpenAEC-Foundation/crates-warehouse
