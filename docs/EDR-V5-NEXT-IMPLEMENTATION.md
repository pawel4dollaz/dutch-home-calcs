# Next implementation gate

The next code change should integrate the feature dimensions into the main calculation kernel only when the relevant NTA 8800:2025+C1:2026 equations, tables and reference values have been obtained.

Do not substitute the feature-level screening proxies in `edr-screening-features.js` into the official calculation path. They exist to make the test dimensions explicit and testable while preserving a fail-closed attestation posture.

Priority order:

1. Import the official `Referentiewaarden` workbook and map outputs to the ISSO 54 test IDs.
2. Implement the official climate/solar geometry model and wire EPW004/008/009/010/011/012 cases into the kernel.
3. Implement the Chapter 8 ground/AOR model for EPW006.
4. Implement Chapter 11 infiltration and ventilation variants for EPW007/EPW101+.
5. Implement the dynamic heat-balance treatment of Dm for EPW005.
6. Implement roller shutters and other window/shading variants for EPW003/EPW011/EPW012.
7. Expand the runner to execute every applicable test and compare every required output within ±1.0%.
