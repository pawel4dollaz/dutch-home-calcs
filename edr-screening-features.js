// Feature-level screening helpers derived from ISSO 54 v5 test dimensions.
// These are intentionally separate from the main NTA screening kernel until the
// corresponding NTA 8800 rules and controlled reference results are implemented.

export const ORIENTATION_FACTOR = Object.freeze({
  N: 0.35, NE: 0.55, E: 0.85, SE: 1.05,
  S: 1.10, SW: 1.05, W: 0.85, NW: 0.55
});

export function orientationSolarFactor(orientation) {
  const key = String(orientation || 'S').toUpperCase();
  if (!(key in ORIENTATION_FACTOR)) throw new Error(`Unsupported orientation: ${orientation}`);
  return ORIENTATION_FACTOR[key];
}

export function shutterThermalResistance(baseU, additionalResistance) {
  const u = Number(baseU), r = Number(additionalResistance);
  if (!Number.isFinite(u) || u <= 0) throw new Error('baseU must be positive');
  if (!Number.isFinite(r) || r < 0) throw new Error('additionalResistance must be non-negative');
  return r === 0 ? u : 1 / (1 / u + r);
}

export function infiltrationScreeningAch({ qv10SpecReken, typeFactor=1, areaM2, volumeM3, pressureExponent=0.5 } = {}) {
  const q=Number(qv10SpecReken), f=Number(typeFactor), a=Number(areaM2), v=Number(volumeM3), n=Number(pressureExponent);
  if (![q,f,a,v,n].every(Number.isFinite) || q<0 || f<=0 || a<=0 || v<=0 || n<=0) throw new Error('Invalid infiltration screening inputs');
  const ach10=(q*f*a/1000*3600)/v;
  return ach10*Math.pow(4/10,n);
}

export function crawlspaceScreeningFactor({ floorRc, rbf=6, openingsM2=0.0012 } = {}) {
  const rc=Number(floorRc), rb=Number(rbf), op=Number(openingsM2);
  if (![rc,rb,op].every(Number.isFinite) || rc<=0 || rb<0 || op<0) throw new Error('Invalid crawlspace inputs');
  // Relative factor only. This intentionally does not claim to implement NTA Chapter 8.
  return 1 / (1 + rc + rb + Math.min(1,op*100));
}

export function thermalMassScreeningRetention(dmKjM2K) {
  const dm=Number(dmKjM2K);
  if (!Number.isFinite(dm) || dm<=0) throw new Error('Dm must be positive');
  return Math.min(0.98, Math.max(0.92, 0.92 + 0.00016*(dm-80)));
}
