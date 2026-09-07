# NTA 8800 calculation-kernel roadmap

## Current finding

The existing `engine.js` is a screening model, not an NTA 8800 calculation kernel. Its climate, transmission, ventilation, DHW, installation and primary-energy calculations contain explicit screening approximations. Those approximations cannot be tuned case-by-case to satisfy the ISSO 54 ±1% EDR criterion without becoming a non-auditable surrogate.

The correct root-cause path is to replace the screening calculation path with a versioned NTA calculation kernel and keep the current browser estimator as a separate screening mode until the new kernel is validated.

## Reference implementation to evaluate

The OpenAEC Foundation's `crates-warehouse` is MIT-licensed and contains separate NTA 8800 crates for the calculation chain: model, tables, geometry, transmission, ventilation, demand, heating, cooling, DHW, humidity, lighting, automation, PV and EP integration. The repository describes the dependency chain and identifies NTA 8800:2025+C1:2026 as the target version. This is substantially closer to the required architecture than continuing to extend the current JavaScript screening formulas.

The OpenAEC crates must **not** be treated as already BRL 9501-attested. Their own `nta8800-ep` documentation describes V1 simplifications, including older primary-energy factors and simplified PV accounting. They are therefore a candidate calculation foundation that must itself be verified against the supplied ISSO 54 reference workbook.

## Implementation sequence

1. **Freeze the screening path.** Do not silently change existing homeowner-screening results while building the conformance kernel.
2. **Add a versioned NTA domain model.** Represent building, zones, constructions, openings, thermal bridges, climate, systems and energy carriers explicitly instead of collapsing them into the current `engine.js` scalar inputs.
3. **Import NTA tables.** Use controlled, versioned table data for NTA 8800:2025+C1:2026, including De Bilt climate, primary-energy factors, thermal-capacity tables, ventilation defaults, transmission defaults and rounding rules.
4. **Implement the calculation chain in dependency order:** geometry → transmission → ventilation/infiltration → monthly demand → heating/cooling → DHW → lighting/auxiliary/automation → PV/renewables → EP1/EP2/EP3/label outputs.
5. **Create explicit adapters for the ISSO 54 fixtures.** Each test must map to complete typed inputs and explicit NTA output posts. No test may rely on hidden defaults.
6. **Run the 386 reference cases.** Compare every mapped numerical output to the controlled workbook using the ±1% criterion. Produce a machine-readable result and a human-readable report.
7. **Close failures by subsystem.** Group deviations by root calculation dependency (e.g. ground transmission, solar geometry, ventilation, heat-pump generation, DHW, primary-energy accounting) rather than adding test-specific corrections.
8. **Only after numerical convergence:** integrate the validated kernel into the browser UI, preserve the screening mode separately, and begin the BRL 9501/QMS release evidence process.

## QMS evidence required for each kernel module

For each implemented module record:

- NTA clause/table/formula reference;
- source table/document version;
- units and conversion rules;
- input validation rules;
- deterministic calculation function;
- unit tests at formula boundaries;
- relevant ISSO 54 test IDs;
- reference expected values;
- actual values and relative error;
- defect/corrective-action records for deviations;
- code review/reviewer;
- release identifier.

## EDR acceptance rule

For a reference value `r != 0` and calculated value `a`:

`relative_error = (a - r) / r`

The test passes only when `abs(relative_error) <= 0.01`, subject to the exact output-specific rules in the current ISSO 54/BRL 9501 specification.

A missing input, missing reference result, unmapped output or unimplemented NTA dependency is `BLOCKED` / `NOT_READY`, never `PASS`.

## Current evidence

The supplied reference workbook is indexed in `data/edr/reference-index.json` and contains 323 base tests, 11 realistic-building tests and 52 Maatwerkadvies tests. The current GitHub Actions run on commit `1dbd39616d794b714f4da0790aa9d60ac0cc9ba4` passed the existing JavaScript regression suite, but that suite does **not** establish EDR conformance. The EDR runner remains deliberately fail-closed.
