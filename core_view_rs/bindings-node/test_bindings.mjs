/**
 * Cross-validation tests for the NAPI-RS Node.js bindings.
 *
 * Validates that the Rust-backed native module produces results
 * matching the original Python (NumPy/SciPy) implementation via golden data.
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { createRequire } from 'module';
import { strict as assert } from 'assert';

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

// Load native binding
const native = require('./index.js');

// Load golden data
const GOLDEN = JSON.parse(readFileSync(join(__dirname, '..', 'scripts', 'golden_data.json'), 'utf-8'));
const SIGNAL = GOLDEN.signal;
const SR = GOLDEN.sampling_rate;
const RTOL = 1e-6;

function assertClose(actual, expected, label, rtol = RTOL) {
  const diff = Math.abs(actual - expected);
  const rel = Math.abs(expected) > 1e-15 ? diff / Math.abs(expected) : diff;
  assert(
    rel < rtol,
    `${label}: actual=${actual.toExponential(10)}, expected=${expected.toExponential(10)}, rel_diff=${rel.toExponential(3)}`
  );
}

function assertArrayClose(actual, expected, label, rtol = RTOL) {
  assert.equal(actual.length, expected.length, `${label}: length mismatch (${actual.length} vs ${expected.length})`);
  for (let i = 0; i < actual.length; i++) {
    assertClose(actual[i], expected[i], `${label}[${i}]`, rtol);
  }
}

// ─── Tests ───────────────────────────────────────────────────────────

const tests = [];

function test(name, fn) {
  tests.push({ name, fn });
}

// acceleration_rms

test('accelerationRms matches golden data', () => {
  const result = native.accelerationRms(SIGNAL, SR);
  assert.equal(result.length, 3, 'should return [acc, vel, dis]');
  const py = GOLDEN.acceleration_rms;
  assertClose(result[0], py.acc_rms, 'acc_rms');
  assertClose(result[1], py.vel_rms, 'vel_rms');
  assertClose(result[2], py.dis_pkpk, 'dis_pkpk');
});

test('accelerationRms with custom band', () => {
  const result = native.accelerationRms(SIGNAL, SR, 20.0, 500.0);
  assert.equal(result.length, 3);
  assert(result[0] > 0, 'acc_rms should be positive');
  assert(result[1] > 0, 'vel_rms should be positive');
  assert(result[2] > 0, 'dis_pkpk should be positive');
});

test('accelerationRms default params match explicit', () => {
  const r1 = native.accelerationRms(SIGNAL, SR);
  const r2 = native.accelerationRms(SIGNAL, SR, 10.0, 1000.0);
  assertArrayClose(r1, r2, 'default vs explicit');
});

// acceleration_rms2

test('accelerationRms2 matches golden data', () => {
  const result = native.accelerationRms2(SIGNAL, SR);
  assert.equal(result.length, 3);
  const py = GOLDEN.acceleration_rms2;
  assertClose(result[0], py.acc_rms, 'acc_rms');
  assertClose(result[1], py.vel_rms, 'vel_rms');
  assertClose(result[2], py.dis_pkpk, 'dis_pkpk');
});

// get_fft_abs_data

test('getFftAbsData lengths match golden', () => {
  const result = native.getFftAbsData(SIGNAL, SR);
  assert.equal(result.length, 3, 'should return [fft_abs, freqs, angles]');
  assert.equal(result[0].length, GOLDEN.get_fft_abs_data.fft_abs.length, 'fft_abs length');
  assert.equal(result[1].length, GOLDEN.get_fft_abs_data.freqs.length, 'freqs length');
  assert.equal(result[2].length, GOLDEN.get_fft_abs_data.angles.length, 'angles length');
});

test('getFftAbsData freqs match golden', () => {
  const result = native.getFftAbsData(SIGNAL, SR);
  assertArrayClose(result[1], GOLDEN.get_fft_abs_data.freqs, 'freqs', 1e-12);
});

test('getFftAbsData magnitude match golden', () => {
  const result = native.getFftAbsData(SIGNAL, SR);
  assertArrayClose(result[0], GOLDEN.get_fft_abs_data.fft_abs, 'fft_abs');
});

// time_domain_features

test('timeDomainFeatures matches golden data', () => {
  const td = native.timeDomainFeatures(SIGNAL);
  assert.equal(td.length, 13, 'should return 13 features');
  assertArrayClose(td, GOLDEN.time_domain_features, 'time_domain');
});

// freq_domain_features

test('freqDomainFeatures matches golden data', () => {
  const pyFftAbs = GOLDEN.get_fft_abs_data.fft_abs;
  const pyFreqs = GOLDEN.get_fft_abs_data.freqs;
  const fd = native.freqDomainFeatures(pyFftAbs, pyFreqs);
  assert.equal(fd.length, 12, 'should return 12 features');
  assertArrayClose(fd, GOLDEN.freq_domain_features, 'freq_domain');
});

// end-to-end pipeline

test('end-to-end pipeline', () => {
  const [fftAbs, freqs, angles] = native.getFftAbsData(SIGNAL, SR);
  const rms = native.accelerationRms(SIGNAL, SR);
  const td = native.timeDomainFeatures(SIGNAL);
  const fd = native.freqDomainFeatures(fftAbs, freqs);

  assert.equal(td.length, 13);
  assert.equal(fd.length, 12);
  assert(rms[0] > 0);

  // Centroid should be within Nyquist
  const nyquist = SR / 2;
  assert(fd[4] > 0 && fd[4] < nyquist, `Centroid ${fd[4]} should be in (0, ${nyquist})`);
});

test('end-to-end freq features from Rust FFT', () => {
  const [fftAbs, freqs] = native.getFftAbsData(SIGNAL, SR);
  const fd = native.freqDomainFeatures(fftAbs, freqs);
  // Looser tolerance for full end-to-end (FFT precision differences)
  assertArrayClose(fd, GOLDEN.freq_domain_features, 'freq_domain_e2e', 1e-4);
});

// ─── Runner ──────────────────────────────────────────────────────────

let passed = 0;
let failed = 0;

for (const { name, fn } of tests) {
  try {
    fn();
    console.log(`  PASS  ${name}`);
    passed++;
  } catch (e) {
    console.log(`  FAIL  ${name}: ${e.message}`);
    failed++;
  }
}

console.log(`\n${passed + failed} tests: ${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
