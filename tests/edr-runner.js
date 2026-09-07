import { calculate } from '../engine.js';
import manifest from './edr-manifest.json' with { type: 'json' };

// This runner deliberately distinguishes an EDR execution from an EDR conformance pass.
// The public ISSO 54 document contains the test definitions, but the official reference
// result workbook is a separate document. Until that workbook is available to the project,
// numerical conformance must remain BLOCKED rather than guessed.

const base = manifest.historical_reference_building;

function screeningTranslation(overrides = {}) {
  const envelope = base.envelope.map(x => ({
    name: x.name,
    area: x.area_m2,
    u: x.boundary === 'ground' ? 0.21 : 1 / x.rc_m2K_W
  }));
  return {
    area: base.geometry.ag_m2,
    volume: base.geometry.volume_m3,
    internalGainsW: 3,
    thermalBridgeLossWk: 0,
    envelope,
    windows: [{ name: 'south HR++ windows', area: base.windows.area_m2, u: base.windows.u_window_W_m2K, gValue: base.windows.g }],
    ventilation: { ach: 0.7, heatRecovery: 0.80 },
    heating: { type: 'gas', seasonalCOP: 0.95, efficiency: 0.95, distributionLossFraction: 0.05, backupFraction: 0, supplyTemperature: 45 },
    dhw: { people: 2, litresPerPersonDay: 35, efficiency: 0.95 },
    lightingKwhM2: 8,
    auxKwhM2: 4,
    coolingKwhM2: 0,
    cooling: { type: 'heatpump', cop: 3 },
    pv: { kWp: 0, yieldFactor: 1, selfUse: 0.35 },
    ...overrides
  };
}

const cases = [
  ['EPW001', screeningTranslation()],
  ['EPW002a', screeningTranslation({ year: 1964, envelope: [
    {name:'roof',area:48,u:1/0.22},{name:'ground floor',area:48,u:1/0.15},
    {name:'south wall',area:19.2,u:1/0.85},{name:'east wall',area:32.4,u:1/0.85},
    {name:'west wall',area:32.4,u:1/0.85},{name:'north wall',area:43.2,u:1/0.85}
  ], windows:[{name:'single glazing',area:24,u:5.4,gValue:0.85}] })],
  ['EPW002b', screeningTranslation({ year: 1995, envelope: [
    {name:'roof',area:48,u:0.4},{name:'ground floor',area:48,u:0.4},
    {name:'south wall',area:19.2,u:0.4},{name:'east wall',area:32.4,u:0.4},
    {name:'west wall',area:32.4,u:0.4},{name:'north wall',area:43.2,u:0.4}
  ] })],
  ['EPW003c', screeningTranslation()],
  ['EPW004a', screeningTranslation()],
  ['EPW004b', screeningTranslation()],
  ['EPW004c', screeningTranslation()],
  ['EPW004d', screeningTranslation()],
  ['EPW004e', screeningTranslation()],
  ['EPW004f', screeningTranslation()],
  ['EPW004g', screeningTranslation()]
];

const results = cases.map(([id, input]) => {
  const result = calculate(input);
  const finite = [result.usefulHeat, result.ep2, result.electricity.grid].every(Number.isFinite);
  return {
    id,
    execution: finite ? 'EXECUTED' : 'ERROR',
    conformance: 'BLOCKED',
    reason: 'Official ISSO 54 reference result workbook is not bundled; current engine is a screening model and does not implement the full NTA calculation chain.',
    screening: finite ? { ep2: result.ep2, label: result.label, usefulHeat: result.usefulHeat } : null
  };
});

console.log(JSON.stringify({
  source: manifest.source,
  published_scope: manifest.published_scope,
  cases: results,
  policy: 'Never convert missing reference data into a PASS.'
}, null, 2));
