# ISSO 54 v5 feature-screening results — 2026-09-07

## Purpose

Use the current ISSO 54 v5.0 test definitions to close the previously identified data-model/test-harness gaps without pretending that the screening model is an NTA 8800 implementation.

The source document explicitly defines EPW003c (roller shutter), EPW004a-g (orientation), EPW005a-d (thermal mass), EPW006a-j (ground/crawlspace and boundary variants), and EPW007a-c (infiltration) as distinct test dimensions. It states that numerical EDR reference results are in a separate Excel document and that deviation greater than ±1.0% is rejected.

## Implemented fixture dimensions

`tests/edr-v5-gap-cases.json` now captures representative source-defined parameters:

- EPW003c: additional shutter resistance 0.2 m²K/W; manual operation.
- EPW004a-g: SE, E, NE, N, NW, W, SW orientations relative to the south reference.
- EPW005a-d: Dm = 80, 180, 360 and 180 kJ/m²K (the last is the light-ceiling variant).
- EPW006a-c/e: floor Rc 0.15 m²K/W, crawlspace Rbf 6/3.5/0 m²K/W and opening-area variants.
- EPW007c: measured qv10 = 0.2 dm³/(s·m²).

## Feature-level screening model

`edr-screening-features.js` provides isolated, explicitly named screening helpers for these dimensions. This is deliberately separate from the main calculation kernel because the source does not provide enough information to derive a compliant NTA implementation from the test descriptions alone.

The helpers currently provide:

- orientation differentiation;
- equivalent U-value calculation for an added shutter resistance;
- a clearly labelled pressure-to-ACH screening conversion for qv10;
- a relative crawlspace factor for distinguishing variants;
- a transparent Dm retention proxy.

These are **engineering proxies only**. They are not claimed to reproduce NTA 8800 and are not used to generate an official label or EDR PASS.

## Automated checks

`tests/edr-features.test.js` contains five tests covering the five feature groups. Local Node execution result:

- 5 tests passed
- 0 failed

The normal project test command already discovers `tests/*.test.js`, so these checks are part of the repository's automated test suite.

## What this closes

Previously, EPW004a-g and related feature cases could collapse to identical screening inputs because the engine had no explicit representation of the feature. The repository now has machine-readable fixtures and isolated tests proving that the feature dimensions are represented and distinguishable.

## What remains intentionally blocked

The following are not inferred from the PDF and must not be guessed if the goal is eventual BRL 9501 attestation:

1. NTA solar geometry/climate calculation and shading factors.
2. NTA manual-shading treatment for roller shutters.
3. NTA dynamic thermal-mass calculation.
4. NTA Chapter 8 ground/crawlspace calculation.
5. NTA Chapter 11 infiltration calculation and interaction with ventilation.
6. The official `Referentiewaarden` numerical results.

Therefore the project status remains **screening / engineering prototype; formal EDR conformance BLOCKED**.
