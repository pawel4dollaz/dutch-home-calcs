import test from 'node:test';
import assert from 'node:assert/strict';
import { labelFor, uFromR, calculate } from '../engine.js';

test('published EP2 label boundaries', () => {
  const cases = [[-1,'A++++'],[0,'A++++'],[0.01,'A+++'],[50,'A+++'],[50.01,'A++'],[75,'A++'],[75.01,'A+'],[105,'A+'],[105.01,'A'],[160,'A'],[160.01,'B'],[190,'B'],[190.01,'C'],[250,'C'],[250.01,'D'],[290,'D'],[290.01,'E'],[335,'E'],[335.01,'F'],[380,'F'],[380.01,'G']];
  for (const [ep,label] of cases) assert.equal(labelFor(ep),label,`EP2 ${ep}`);
});

test('Rc to U conversion', () => assert.equal(uFromR(2.5),0.4));

test('better envelope cannot increase heat demand', () => {
  const base={area:51,volume:128,internalGainsW:3,envelope:[{area:51,u:.4},{area:38,u:.35},{area:51,u:.5}],windows:[{area:9,u:1.4,gValue:.55}],ventilation:{ach:.7,heatRecovery:0},heating:{type:'heatpump',seasonalCOP:3.5,distributionLossFraction:.05,backupFraction:0},dhw:{people:2,litresPerPersonDay:35,efficiency:2.69},lightingKwhM2:8,auxKwhM2:4,coolingKwhM2:0,cooling:{type:'heatpump',cop:3},pv:{kWp:0,yieldFactor:1,selfUse:.35}};
  const worse=calculate(base); const better=calculate({...base,envelope:[{area:51,u:.2},{area:38,u:.2},{area:51,u:.2}]});
  assert.ok(better.usefulHeat<worse.usefulHeat);
});

test('PV cannot increase grid electricity in the screening model', () => {
  const base={area:51,volume:128,internalGainsW:3,envelope:[{area:51,u:.4}],windows:[{area:9,u:1.4,gValue:.55}],ventilation:{ach:.7,heatRecovery:0},heating:{type:'heatpump',seasonalCOP:3.5,distributionLossFraction:.05,backupFraction:0},dhw:{people:2,litresPerPersonDay:35,efficiency:2.69},lightingKwhM2:8,auxKwhM2:4,coolingKwhM2:0,cooling:{type:'heatpump',cop:3},pv:{kWp:0,yieldFactor:1,selfUse:.35}};
  const noPv=calculate(base); const withPv=calculate({...base,pv:{kWp:3,yieldFactor:1,selfUse:.5}});
  assert.ok(withPv.electricity.grid<=noPv.electricity.grid);
});

test('PV is counted once in the screening renewable-energy balance', () => {
  const input={area:100,volume:250,internalGainsW:0,envelope:[],windows:[],ventilation:{ach:0,heatRecovery:0},heating:{type:'electric',distributionLossFraction:0},dhw:{people:0,litresPerPersonDay:0,efficiency:1},lightingKwhM2:10,auxKwhM2:0,coolingKwhM2:0,cooling:{type:'electric',cop:1},pv:{kWp:1,yieldFactor:1,selfUse:0.5}};
  const result=calculate(input);
  assert.equal(result.electricity.pv,900);
  assert.equal(result.electricity.pvUsed,450);
  assert.equal(result.renewableSources.pvGeneration,900);
  const denominator=Math.max(1,result.usefulHeat+result.dhw.useful+result.cooling.useful);
  // This identity catches the original `pv + pvUsed` double count even when the
  // externally reported share is bounded to 1.
  assert.equal(result.renewableShare,Math.min(1,900/denominator));
});
