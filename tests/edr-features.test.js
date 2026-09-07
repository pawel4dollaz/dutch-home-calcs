import test from 'node:test';
import assert from 'node:assert/strict';
import { orientationSolarFactor, shutterThermalResistance, infiltrationScreeningAch, crawlspaceScreeningFactor, thermalMassScreeningRetention } from '../edr-screening-features.js';

test('EPW004 orientation set is represented',()=>{
  const values=['S','SE','E','NE','N','NW','W','SW'].map(orientationSolarFactor);
  assert.equal(values.length,8); assert.ok(new Set(values).size>1); assert.equal(orientationSolarFactor('S'),1.10);
});

test('EPW003c additional shutter resistance is represented',()=>{
  assert.ok(shutterThermalResistance(1.8,.2)<1.8);
  assert.equal(shutterThermalResistance(1.8,0),1.8);
});

test('EPW007c blower-door value can be converted to screening ACH',()=>{
  const ach=infiltrationScreeningAch({qv10SpecReken:.2,typeFactor:1.4,areaM2:96,volumeM3:259.2});
  assert.ok(ach>0 && ach<1);
});

test('EPW006 crawlspace variants remain distinguishable',()=>{
  const tight=crawlspaceScreeningFactor({floorRc:.15,rbf:6,openingsM2:.0006});
  const poor=crawlspaceScreeningFactor({floorRc:.15,rbf:0,openingsM2:.0012});
  assert.notEqual(tight,poor);
});

test('EPW005 Dm variants remain distinguishable',()=>{
  const light=thermalMassScreeningRetention(80); const heavy=thermalMassScreeningRetention(450);
  assert.ok(light<heavy); assert.ok(light>=.92 && heavy<=.98);
});
