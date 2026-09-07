# Quality Manual — Dutch Home Calcs

**Document:** QM-001  
**Status:** Draft for implementation  
**Scope:** NTA 8800 calculation software / future BRL 9501 EDR Energieprestatie attestation

## 1. Purpose

This quality system controls the development, verification, release and maintenance of the Dutch Home Calcs calculation method and user interface.

It is designed around the structure of BRL 9501 and the EDR requirements in ISSO Publication 54.

It does not itself constitute a BRL 9501 certificate or attest.

## 2. Roles

At minimum the project must designate separate responsibilities for:

- Product owner — accountable for scope and priorities.
- Calculation lead — accountable for NTA implementation.
- Verification lead — independently reviews calculation changes.
- Release manager — controls production releases.
- Quality manager — controls the quality system and non-conformities.
- Support/complaints owner — records and closes user complaints.
- Final release approver — confirms all release gates are satisfied.

For a one-person organisation, incompatible roles must still be explicitly separated in the records; independent review cannot simply be self-declared as independent.

## 3. Document control

Controlled documents receive:

- document ID;
- title;
- version;
- status;
- author;
- reviewer;
- approval date;
- applicable NTA/BRL/ISSO versions;
- change history.

Superseded controlled documents remain retrievable and are never silently overwritten.

## 4. Requirements management

Every material calculation requirement receives a unique requirement ID, for example:

`NTA-8.15-001` — window U-value calculation.

Requirements are linked to:

`requirement → implementation → unit test → EDR test → release → evidence`.

A requirement may not be marked complete without objective evidence.

## 5. Calculation-kernel control

The calculation kernel is treated as a controlled safety/quality-critical component.

A calculation-kernel change requires:

1. issue/change record;
2. affected NTA clauses identified;
3. affected EDR tests identified;
4. implementation;
5. unit tests;
6. regression tests;
7. EDR tests;
8. result review;
9. release note;
10. approval before production release.

## 6. User-interface control

UI changes are classified as:

- presentation-only;
- input-semantic;
- output/reporting;
- calculation-affecting.

Even a UI change requires regression testing when it can alter an input value, unit, default, interpretation or output.

## 7. Test strategy

### 7.1 Unit tests

Test individual formulas and conversion functions.

### 7.2 Property tests

Examples:

- increasing insulation cannot increase transmission losses;
- increasing PV cannot increase grid electricity;
- zero PV produces zero PV generation;
- zero cooling demand produces zero cooling electricity;
- invalid physical values are rejected;
- units remain dimensionally consistent.

### 7.3 Integration tests

Verify the complete flow from building input through EP indicators and label output.

### 7.4 EDR tests

Execute every applicable ISSO 54 test/subtest for the attested scope. Store both inputs and outputs. A missing fixture is `BLOCKED`, never `PASS`.

### 7.5 Regression tests

Every release reruns the complete controlled regression suite. Results are compared with the previous approved release and with the applicable reference results.

## 8. EDR acceptance

For the publicly available historical ISSO 54 version 2.0 (May 2022), the published rejection criterion is more than ±1.0% from the specified result. The exact tolerance used for the current attestation must always come from the current applicable ISSO 54 version.

No rounding is applied before comparison unless the reference specification explicitly requires it.

## 9. Defects and non-conformities

Every calculation defect receives:

- unique ID;
- discovery date;
- affected release;
- severity;
- reproducible input;
- observed output;
- expected output;
- root cause;
- correction;
- regression test;
- reviewer;
- closure date.

A defect affecting an EDR test cannot be closed merely because the visible output looks plausible.

## 10. Release management

Version numbers distinguish at least:

- NTA calculation-kernel version;
- application version;
- data/reference version.

Each production release includes a release note listing:

- changed calculation logic;
- changed defaults;
- changed reference data;
- changed UI/input semantics;
- known limitations;
- EDR results;
- regression results.

## 11. Historical versions

When a governing NTA version changes, the old calculation kernel and its supporting reference data are retained for the required period and remain executable/reproducible.

Source control alone is insufficient if dependencies or runtime environments can change the result.

## 12. Complaints

Complaints are recorded with:

- complainant/date;
- affected software version;
- building/project identifier where appropriate;
- description;
- investigation owner;
- investigation result;
- corrective action;
- response to complainant;
- closure.

Potential calculation errors are escalated to the calculation lead and quality manager.

## 13. Independence

The organisation must document legal/economic relationships that could affect independence. Where an independent review is required, the reviewer must have sufficient separation from the implementation decision.

## 14. Access and security

Production releases are protected from uncontrolled modification.

At minimum:

- protected main branch;
- review before merge;
- tagged releases;
- restricted release credentials;
- auditable release history;
- dependency/version lock where practical;
- backups of controlled records.

## 15. External dependencies

External libraries, reference databases and hosted services that can affect a calculation are recorded with version/date. A calculation release must not silently begin using a different external dataset.

The planned NTA kernel currently pins the OpenAEC NTA 8800 crate family to commit `6e8738c075719e2fc8fcf918d969406f98927b07` in `rust/Cargo.toml`. This pin is a supplier/dependency-control record, not an assertion that the upstream project is BRL 9501-attested.

The controlled BouwZo reference workbook is not redistributed in this public repository. Its SHA-256 is recorded in `data/edr/reference-index.json`. `tools/import_edr_reference.py` is the controlled extraction mechanism; it must fail closed on missing sheets, duplicate test IDs or duplicate output posts and must emit the source hash in the generated index.

## 16. Audit evidence

The following evidence is retained for each release:

- source commit/tag;
- build identifier;
- environment/dependency manifest;
- test report;
- EDR report;
- reviewer approval;
- release note;
- known defect list;
- controlled source-data hashes;
- dependency revision/lock information.

## 17. Management review

At planned intervals the quality owner reviews:

- open defects;
- complaints;
- EDR trends;
- release stability;
- changes in NTA/BRL/ISSO requirements;
- audit findings;
- corrective actions.

The review is recorded and produces actions with owners and due dates.

## 18. Current implementation gap

This document is intentionally ahead of the software. The repository currently has only a partial screening calculation kernel, while the new Rust dependency boundary is a foundation for replacing that kernel with a versioned NTA implementation.

The next technical gate is complete NTA implementation followed by execution of the complete current ISSO 54 EDR suite and independent assessment.
