# ISSO 54 v5 reference-data integration — 2026-09-07

## Source package

The user supplied the BouwZo workbook `Referentiewaarden Energielabel 2025_correctie 16-04-2026.xlsx` plus ISSO 54 v5 Appendices 3A–3K and 4–14. Appendices 12 and 14 complete the product-specific declaration set.

The workbook contains three numerical reference sheets:

- `Basistests`: 323 EDR tests, 28 output rows, 9,044 populated values
- `Realistische gebouwen`: 11 tests, 28 output rows, 308 populated values
- `Maatwerkadvies`: 52 tests, 37 output rows, 987 populated values

Total inventory: **386 named tests** and **10,339 populated reference values** across the three numerical sheets.

Workbook SHA-256: `7de9a51aaad2aef44b2296ef9ec9f82b5c70f2cfe8ba2dba1eaba3f9f611a319`

## Eight-step plan status

1. **Normalize reference values — DONE at source-analysis level.** The workbook structure, test inventory, output posts, units and numeric values have been inspected. A compact provenance/index is committed; the full workbook remains the controlled source artifact rather than being copied wholesale into the public repository.

2. **Parse realistic-building recording forms — SOURCE SET COMPLETE; STRUCTURED MAPPING IN PROGRESS.** All 11 Appendix 3A–3K forms are now available and indexed. They cover EPWRealB01/D01, B02/D02, B03/D04, B05/B06/B07, D05 and EPURealB01.

3. **Map tests to required inputs/outputs — FRAMEWORK STARTED.** Test IDs and output posts are now controlled data. Complete input-to-equation mapping remains tied to the NTA 8800 implementation work.

4. **Expand EDR runner — DONE at inventory level.** The runner consumes the v5 inventory metadata rather than the old small hand-written case list.

5. **Classify runnable vs blocked — DONE.** The project distinguishes screening execution from EDR conformance. Missing NTA modules or mappings are reported as NOT_READY, never as PASS.

6. **Implement calculation modules — NOT COMPLETE.** The existing `engine.js` is still a screening model. The v5 reference suite now supplies the target-driven backlog for implementing the actual NTA calculation chain.

7. **Add ±1% gate — DONE.** `compareRelative()` implements the ISSO 54 numerical gate and has regression tests, including zero-reference handling.

8. **Preserve provenance/audit trail — DONE for the current source package.** The workbook hash and source filename are recorded. Appendix 3 and product-declaration manifests record the available source set. Original copyrighted PDFs are not copied into the public repository.

## Product declarations

The complete Appendix 4–14 set is now available in the working source package, including:

- Panasonic L-series / BCRG `20230295GK` for EPW406aa
- Vincent V45-combi + WPV 150L / BCRG `20220186GG` for EPW406ab
- Jinko Solar JKM470N-60HL4 / BCRG `20230252GK` for EPW501e

## Next technical gate

The project is now ready to move from **reference-data acquisition** to **target-driven NTA implementation**. The next work should prioritize modules that unlock the largest number of EDR tests, while using the reference workbook as the numerical regression oracle. A genuine EDR PASS must only be reported after an explicit NTA output mapping exists and the result is within the ±1% criterion.
