# DH-001 — PV renewable-share double-counting correction

**Date:** 2026-09-07  
**Affected calculation:** the browser engine's explicitly non-NTA screening renewable-share output

## Finding

The former expression counted annual PV production and then added `pv - export`. Since `export = pv - pvUsed`, the second term equals `pvUsed`; every self-consumed PV kWh was therefore counted twice.

## Correction

The engine now counts PV generation once and returns the component ledger:

- `renewableSources.heatPumpAmbient`
- `renewableSources.pvGeneration`

The renewable share is the sum of those source quantities divided by the existing screening denominator, bounded to 0–1. It is deliberately not labelled or used as NTA 8800 EP3.

## Regression evidence

The unit fixture sets PV to 900 kWh/y and self-use to 50%. It proves that the reported PV renewable contribution is 900 kWh/y and verifies the component-ledger identity against the unrounded screening denominator. The former expression would have produced 1,350 kWh/y before clamping.

## Scope limitation

NTA 8800 renewable-energy accounting, PV allocation, primary-energy factors, EP3 and rounding still require the controlled NTA version, associated tables, and complete EDR fixture mappings. DH-001 improves the integrity of the existing screening output only; it does not make any EDR case executable.
