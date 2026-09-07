# ISSO 54 v5 screening-gap closure

The current v5 test document identifies orientation (EPW004), thermal mass (EPW005), ground/crawlspace boundary (EPW006), infiltration (EPW007) and roller-shutter properties (EPW003c) as explicit test dimensions.

The repository now records fixtures for these dimensions in `tests/edr-v5-gap-cases.json` and executable screening checks in `tests/feature-screening.js`.

## Status

- Orientation: fixture captured; main calculator integration remains blocked until its solar model is replaced by the NTA 8800 geometry/climate calculation.
- Roller shutter: 0.2 m²K/W input captured; exact manual-operation treatment remains blocked pending the NTA rules/reference result.
- Thermal mass: Dm values 80/180/360/450 kJ/m²K captured; exact dynamic calculation remains blocked pending the NTA method implementation.
- Ground/crawlspace: Rbf and opening-area variants captured; exact chapter-8 ground calculation remains blocked.
- Infiltration: qv10=0.2 dm³/(s·m²) captured; exact pressure/leakage conversion remains blocked.

No item above is treated as an EDR PASS. The purpose of these fixtures is to prevent the known dimensions from disappearing into a generic screening input while the full NTA implementation is built.
