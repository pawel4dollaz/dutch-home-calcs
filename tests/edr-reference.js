// Reference workbook metadata and comparison gate for ISSO 54 v5.
// Source: user-supplied BouwZo workbook recorded in data/edr/reference-index.json.

import index from '../data/edr/reference-index.json' with { type: 'json' };

export const REFERENCE_INDEX = index;

export function referenceSummary() {
  return {
    totalTests: index.total_tests,
    sheets: index.sheets
  };
}

/**
 * ISSO 54 uses a +/-1% rejection criterion for numerical EDR comparison.
 * This helper intentionally accepts only already-mapped NTA outputs; it does
 * not infer that a screening-model output corresponds to an ISSO post.
 */
export function compareRelative(actual, expected, tolerance = 0.01) {
  if (!Number.isFinite(actual) || !Number.isFinite(expected)) {
    return { pass: false, reason: 'non-finite value' };
  }
  if (expected === 0) {
    return { pass: Math.abs(actual) <= tolerance, relativeError: null };
  }
  const relativeError = (actual - expected) / expected;
  return { pass: Math.abs(relativeError) <= tolerance, relativeError };
}
