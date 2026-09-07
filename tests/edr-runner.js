import { calculate } from '../engine.js';
import manifest from './edr-manifest.json' with { type: 'json' };
import { referenceSummary, compareRelative } from './edr-reference.js';

// EDR execution and EDR conformance are intentionally separate. The current engine
// remains a screening model. The runner now knows the complete ISSO 54 v5 inventory,
// but will not map a screening output to an ISSO post unless that mapping is explicit.

const summary = referenceSummary();
const base = manifest.historical_reference_building;

function screeningTranslation() {
  return {
    area: base.geometry.ag_m2,
    volume: base.geometry.volume_m3,
    internalGainsW: 3,
    thermalBridgeLossWk: 0,
    envelope: base.envelope.map(x => ({ name:x.name, area:x.area_m2, u:x.boundary==='ground'?0.21:1/x.rc_m2K_W })),
    windows: [{ name:'south HR++ windows', area:base.windows.area_m2, u:base.windows.u_window_W_m2K, gValue:base.windows.g }],
    ventilation:{ ach:0.7, heatRecovery:0.80 },
    heating:{ type:'gas', efficiency:0.95, distributionLossFraction:0.05, supplyTemperature:45 },
    dhw:{ people:2, litresPerPersonDay:35, efficiency:0.95 },
    lightingKwhM2:8, auxKwhM2:4, coolingKwhM2:0,
    cooling:{type:'heatpump',cop:3}, pv:{kWp:0,yieldFactor:1,selfUse:0.35}
  };
}

const screening = calculate(screeningTranslation());

console.log(JSON.stringify({
  source: manifest.source,
  reference_inventory: summary,
  current_engine: {
    status: 'SCREENING_ONLY',
    example: { ep2: screening.ep2, label: screening.label, usefulHeat: screening.usefulHeat }
  },
  edr_gate: compareRelative(101,100),
  conformance: {
    status: 'NOT_READY',
    executable_cases: 0,
    reason: 'The complete NTA 8800 calculation chain and explicit output mappings are not yet implemented. No screening result is treated as an EDR PASS.'
  }
}, null, 2));
