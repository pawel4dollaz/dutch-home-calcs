import test from 'node:test';
import assert from 'node:assert/strict';
import { calculate } from '../engine.js';

const base={
  area:96, volume:259.2, internalGainsW:3, thermalBridgeLossWk:0,
  envelope:[{name:'roof',area:48,u:1/6},{name:'ground',area:48,u:0.21},{name:'south',area:19.2,u:1/6},{name:'east',area:32.4,u:1/6},{name:'west',area:32.4,u:1/6},{name:'north',area:43.2,u:1/6}],
  windows:[{name:'south',area:24,u:1.8,gValue:.7,orientation:'S'}],
  ventilation:{ach:.7,heatRecovery:.8},
  heating:{type:'gas',efficiency:.95,distributionLossFraction:.05},
  dhw:{people:2,litresPerPersonDay:35,efficiency:.95},
  lightingKwhM2:8,auxKwhM2:4,coolingKwhM2:0,cooling:{type:'heatpump',cop:3},pv:{kWp:0,yieldFactor:1,selfUse:.35}
};

test('EPW004 orientation cases are no longer numerically identical',()=>{
  const south=calculate(base); const north=calculate({...base,windows:[{...base.windows[0],orientation:'N'}]});
  assert.notEqual(south.gains,north.gains);
  assert.notEqual(south.usefulHeat,north.usefulHeat);
});

test('EPW003c roller-shutter resistance is represented in screening input',()=>{
  const open=calculate(base); const shutter=calculate({...base,windows:[{...base.windows[0],additionalResistance:.2}]});
  assert.ok(Number.isFinite(shutter.usefulHeat));
  assert.notEqual(shutter.gains,open.gains);
});

test('EPW005 thermal mass is represented explicitly',()=>{
  const light=calculate({...base,thermalMassKjM2K:80});
  const heavy=calculate({...base,thermalMassKjM2K:450});
  assert.ok(light.thermalMass.dmKjM2K<heavy.thermalMass.dmKjM2K);
  assert.notEqual(light.thermalMass.retention,heavy.thermalMass.retention);
});

test('EPW007c blower-door input is accepted as a screening parameter',()=>{
  const result=calculate({...base,infiltration:{qv10SpecReken:.2,typeFactor:1.4}});
  assert.ok(result.infiltrationAch>0);
  assert.ok(Number.isFinite(result.usefulHeat));
});
