# Defect register

**Document:** QM-DEF-001  
**Status:** Active controlled record  
**Scope:** Calculation-kernel defects and corrective actions

| ID | Discovered | Severity | Affected versions | Root cause | Correction and verification | Status |
|---|---|---|---|---|---|---|
| DH-001 | 2026-09-07 | Major (screening-output integrity) | All revisions through `98e2b7e` | The screening renewable-share expression added total PV generation and its self-consumed subset (`pv - export`), counting self-consumed PV twice. | Count PV generation once, expose source components, and add an exact balance regression test. Full automated suite and EDR harness rerun after the correction. | Corrected; remains outside EDR scope because the metric is not NTA EP3. |
| DH-002 | 2026-09-07 | Major (validation infrastructure) | NTA kernel CI revisions 32–40 | The Cargo command used `--locked=false`, which is not a valid Cargo CLI form. This caused the new NTA-kernel CI job to fail before compilation and therefore prevented validation of the pinned kernel boundary. | Captured the exact failure in a workflow artifact, removed the invalid flag, and retained failure-log upload for future QMS evidence. Verification is the new run on the corrected workflow; the result must be recorded before closure. | Open pending corrected CI result |

## Closure criteria

A defect is closed only after the correction, its regression test, the test run evidence, review, and release identifier are entered. A corrected screening defect does not establish NTA 8800 or BRL 9501 conformance.
