# NTA 8800 kernel implementation status

Date: 2026-09-08

Target: NTA 8800:2025+C1:2026, with the pinned OpenAEC Foundation revision `6e8738c075719e2fc8fcf918d969406f98927b07`.

## Implemented in the project kernel boundary

### Phase 1 — calculation foundation

- Version-pinned OpenAEC dependency set.
- Explicit, serializable project/input model.
- Structural and physical input validation with fail-closed behavior.
- Layered construction resistance/U-value calculation.
- Explicit opaque/opening geometry.
- Outdoor, ground, unheated-space and adjacent-zone transmission paths.
- Linear and point thermal-bridge terms.
- Monthly transmission-energy ledger.

### Phase 2 — building physics

- NTA §11 pressure-balance infiltration foundation.
- Windward/leeward/roof leakage paths.
- Temperature-dependent air density and hydrostatic pressure.
- Mechanical supply/extract mass balance.
- Heat-recovery discharge temperature treatment.
- Bisection pressure solution with the documented mass-balance tolerance.
- Monthly ventilation/infiltration heat-loss ledger.
- H.7 monthly heating/cooling demand ledger.
- Explicit internal gains (with 3 W/m² residential V1 basis where no monthly profile is supplied).
- Explicit window solar gains from supplied orientation climate data.
- Thermal-mass time constant and utilization-factor calculation.

### Phase 3 — service chain

- Heating end-energy ledger from demand and explicit efficiency/COP inputs.
- Cooling end-energy ledger.
- Residential DHW annual demand basis of 856 kWh/resident/year.
- Explicit storage-loss input handling.
- Lighting V1 forfaitaire or explicit annual-energy path.
- Ventilation fan-energy ledger.
- PV yield ledger requiring an explicit yield when PV capacity is non-zero.
- Pinned OpenAEC H.5 EP integration and label result object.

## Deliberate blockers / non-conformance gates

These are **not** silently approximated into an official result:

1. Full NTA H.9/H.10 installation methodology, including temperature-dependent emitter corrections and type-specific heat-pump corrections.
2. Full H.12/H.13 DHW storage/distribution/generator procedures; the service adapter currently uses the published residential annual demand basis and explicit storage-loss input.
3. Full H.14 lighting detail beyond the pinned OpenAEC V1 lumped method.
4. H.15 automation energy calculation.
5. Humidity energy integration.
6. Complete NTA 8800 EP1/EP2/EP3 renewable-energy accounting and current-version factor tables; the pinned OpenAEC EP crate is a V1 implementation and is therefore treated as an intermediate kernel dependency, not as evidence of attestation.
7. Complete climate/location/default-table coverage for every applicable building category.
8. Current ISSO 54 v5 EDR fixture execution for all 386 controlled reference cases.

## Acceptance rule

No code change is considered conformance-complete because an output merely looks plausible. The final gate remains:

`feature-complete implementation → 386 EDR/reference cases → intermediate diagnostics → root-cause corrections → required tolerance checks → QMS evidence/review → release`

Missing inputs, missing reference fixtures, unmapped outputs or unimplemented dependencies remain `BLOCKED`/`NOT_READY` rather than PASS.
