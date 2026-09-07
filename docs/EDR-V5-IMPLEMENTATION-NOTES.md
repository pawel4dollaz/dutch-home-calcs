# ISSO 54 v5 feature implementation notes

The v5 EDR document explicitly tests orientation (EPW004), thermal mass (EPW005), floor/ground boundary (EPW006), infiltration (EPW007), overhangs (EPW008) and roller shutters (EPW003c).

## Closed at the data-model level

The repository now has explicit machine-readable fixture fields for:

- window/main-facade orientation;
- additional shutter resistance;
- thermal mass Dm;
- crawlspace Rbf and ventilation-opening area;
- measured qv10 specific air leakage.

## Still intentionally blocked at the calculation level

These features require NTA 8800 rules and/or controlled reference results to be implemented correctly. We will not replace them with guessed constants and call the result EDR-compliant.

1. Orientation and solar geometry: requires the NTA climate/radiation method rather than a generic compass multiplier.
2. Roller shutters: requires the NTA treatment of manual solar shading and additional thermal resistance.
3. Thermal mass: requires the NTA dynamic heat-balance treatment for Dm and the applicable time-step/usage rules.
4. Ground/crawlspace: requires the NTA Chapter 8 ground-boundary method, including geometry/perimeter and crawlspace parameters.
5. Infiltration: requires the NTA Chapter 11 pressure/leakage conversion, building-height/type factors and interaction with ventilation.
6. Overhangs and side obstructions: require solar geometry and shading calculations.

This distinction is deliberate: the current application remains a screening calculator, while the fixtures ensure that these EDR dimensions are represented explicitly and cannot be silently lost.
