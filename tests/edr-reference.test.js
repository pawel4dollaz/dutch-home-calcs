import test from 'node:test';
import assert from 'node:assert/strict';
import { REFERENCE_INDEX, referenceSummary, compareRelative } from './edr-reference.js';

test('ISSO 54 v5 reference inventory is complete', () => {
  const s = referenceSummary();
  assert.equal(s.totalTests, 386);
  assert.equal(s.sheets.Basistests.tests, 323);
  assert.equal(s.sheets['Realistische gebouwen'].tests, 11);
  assert.equal(s.sheets.Maatwerkadvies.tests, 52);
});

test('reference provenance is pinned to the supplied workbook', () => {
  assert.equal(REFERENCE_INDEX.source_file, 'Referentiewaarden Energielabel 2025_correctie 16-04-2026.xlsx');
  assert.match(REFERENCE_INDEX.sha256, /^[0-9a-f]{64}$/);
  assert.equal(REFERENCE_INDEX.sample.EPW001a['QH;nd;net'], 42.69);
  assert.equal(REFERENCE_INDEX.sample.EPW001a['A_g'], 96);
});

test('the numerical EDR gate is +/-1%', () => {
  assert.equal(compareRelative(100, 100).pass, true);
  assert.equal(compareRelative(101, 100).pass, true);
  assert.equal(compareRelative(101.01, 100).pass, false);
  assert.equal(compareRelative(0, 0).pass, true);
});
