# BRL 9501 attestation readiness

## Status

**Not yet attestable.** This repository now contains an attestation-oriented quality system and EDR harness structure, but the calculation core is not yet a complete implementation of NTA 8800:2025+C1:2026 and the project has not undergone an independent BRL 9501 assessment.

This distinction is deliberate. BRL 9501 requires the calculation method to calculate the energy performance according to NTA 8800 and requires the EDR tests from ISSO Publication 54 to fall within their prescribed bands. The attestation is issued by an independent, accredited attestation institution; it cannot be self-declared by this project.

## Applicable framework

For the current 2026 regime the project must be controlled against:

- NTA 8800:2025+C1:2026;
- BRL 9501:2026 / the version designated in the current Dutch regulations;
- ISSO Publication 54, current version;
- for eventual residential EP registration: BRL 9500-W and the current ISSO Publication 82.1.

The 2022 public ISSO 54 document is retained as a historical EDR reference because it exposes the test structure and a substantial amount of test input publicly. It is **not** treated as the sole current attestation specification.

## Required work before an attestation application

### 1. Calculation completeness

The calculation kernel must implement the complete applicable NTA 8800 calculation chain rather than the current screening approximation. In particular this includes:

- monthly heat-demand calculation;
- transmission through all applicable boundary types, including ground and adjacent zones;
- solar gains, orientation, glazing, frame fractions, shading and solar-control devices;
- thermal bridges and the applicable forfaitaire/detail methods;
- infiltration and air-tightness rules;
- all ventilation system variants and electrical auxiliaries;
- space-heating emission, distribution, control and generation;
- heat-pump performance, interpolation/extrapolation and auxiliary energy;
- domestic-hot-water emission, distribution, storage, generation and recovery;
- cooling;
- PV and other renewable energy sources;
- primary-energy and renewable-energy accounting;
- EP1, EP2, EP3 and the current label conversion;
- current rounding and reporting rules;
- applicable 2026 additions to the energy-label output.

### 2. EDR conformance

The project needs machine-readable fixtures for every applicable ISSO 54 test and subtest. Each fixture should contain:

- test ID;
- NTA/ISSO version;
- complete input dataset;
- expected output values;
- permitted lower/upper band;
- calculation version;
- execution timestamp;
- actual output;
- absolute and relative deviation;
- pass/fail;
- reviewer/sign-off information.

No test should be silently skipped. If an expected result or input fixture is unavailable, the result must be `BLOCKED`, not `PASS`.

### 3. Release control

Every release affecting the calculation method or UI must:

1. receive a unique version;
2. identify whether the calculation kernel changed;
3. execute the full applicable regression suite;
4. execute all relevant EDR tests;
5. preserve test outputs;
6. generate release notes;
7. identify changed NTA/ISSO/BRL assumptions;
8. be independently reviewed before production release.

### 4. Configuration/version retention

When the NTA calculation method changes, the previous calculation kernel must remain reproducible for the required retention period. The project therefore needs immutable release tags and archived runtime environments, not merely source-code history.

### 5. Quality management system

The repository now contains a draft quality manual covering:

- organisation and responsibilities;
- document control;
- software configuration management;
- requirements traceability;
- change control;
- verification and validation;
- EDR testing;
- release approval;
- complaint handling;
- incident/non-conformity handling;
- corrective and preventive actions;
- retention and archival;
- independence/conflict-of-interest controls;
- supplier/dependency controls;
- security and access control;
- audit evidence.

This is a **template for implementation**, not evidence that the organisation already satisfies the BRL.

## Attestation package checklist

Before approaching an attestation institution, the following package should be complete:

- [ ] controlled requirements specification;
- [ ] NTA clause-to-code traceability matrix;
- [ ] complete EDR input/output fixture set;
- [ ] EDR results for all required tests;
- [ ] regression-test report;
- [ ] calculation-kernel architecture description;
- [ ] user-interface specification;
- [ ] user manual;
- [ ] installation/deployment documentation;
- [ ] version/release procedure;
- [ ] change-control log;
- [ ] defect/non-conformity log;
- [ ] complaint procedure and register;
- [ ] test-environment specification;
- [ ] source-code version-control evidence;
- [ ] reproducible build/release procedure;
- [ ] software retention/archive procedure;
- [ ] licence-holder registration process if applicable;
- [ ] evidence of internal EDR testing;
- [ ] management approval of the quality system;
- [ ] formal application to an accredited attestation institution.

## Important boundary

The goal is to make the project **ready to be assessed**, not to simulate the assessment. The independent attestation body must determine whether the calculation method and quality system satisfy BRL 9501.
