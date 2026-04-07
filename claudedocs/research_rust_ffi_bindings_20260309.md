# Rust FFI Bindings Research Report

**Project**: a003_core_view (vibration analysis / signal feature extraction)
**Date**: 2026-03-09
**Goal**: Port core Python (numpy/scipy) computation to Rust, expose via bindings to both Node.js and Python. Target platforms: Windows and Linux only.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current Python Code Inventory](#2-current-python-code-inventory)
3. [Recommended Architecture](#3-recommended-architecture)
4. [Rust-to-Python Bindings](#4-rust-to-python-bindings)
5. [Rust-to-Node.js Bindings](#5-rust-to-nodejs-bindings)
6. [Rust DSP & Scientific Computing Ecosystem](#6-rust-dsp--scientific-computing-ecosystem)
7. [NumPy/SciPy to Rust Operation Mapping](#7-numpyscipy-to-rust-operation-mapping)
8. [Known Gotchas & Pain Points](#8-known-gotchas--pain-points)
9. [Recommended Cargo Dependencies](#9-recommended-cargo-dependencies)

---

## 1. Executive Summary

### Binding Choices

| Target | Tool | Why |
|--------|------|-----|
| **Python** | PyO3 + maturin + rust-numpy | Zero-copy numpy interop, production-proven (Polars, ruff, pydantic) |
| **Node.js** | NAPI-RS v3 | Zero-copy Float64Array, best cross-platform tooling, optional WASM fallback |

### Rust DSP Crates

| Role | Crate | Version |
|------|-------|---------|
| FFT (rfft/irfft) | `realfft` or `ndrustfft` | 3.5 / 0.6 |
| N-D arrays | `ndarray` | 0.17 |
| Statistics (skew, kurtosis) | `ndarray-stats` | 0.7 |
| Complex numbers | `num-complex` | 0.4 |

### Porting Scope

- **5 public functions** across 2 files (`vibration.py`, `feature.py`)
- Core ops: FFT (rfft, irfft, rfftfreq), basic stats (mean, var, std, sum, max, min), skewness, kurtosis
- All operations have mature Rust equivalents
- Main pain point: no boolean mask indexing in ndarray (use iterator workarounds)

### Verdict

The port is **feasible and production-ready**. The main effort will be in (a) boolean indexing workarounds and (b) carefully matching normalization conventions. Performance payoff should be substantial -- rustfft is competitive with FFTW.

---

## 2. Current Python Code Inventory

### `vibration.py` -- `Vibration` class (3 classmethods)

#### `acceleration_rms(data, sampling_rate, freq_lower_bound=10, freq_upper_bound=1000)`
- Input: acceleration data in G units, sampling rate
- Operations: `rfft`, `rfftfreq`, boolean mask filtering, `irfft`, element-wise arithmetic
- Returns: `(acc_rms, vel_rms, dis_pkpk)` -- acceleration RMS (G), velocity RMS (m/s), peak-peak displacement (m)

#### `acceleration_rms2(data, sampling_rate, freq_lower_bound=10, freq_upper_bound=1000)`
- Alternative PSD-based approach
- Operations: `rfft`, `rfftfreq`, boolean mask filtering, PSD calculation
- Returns: `(acc_rms, vel_rms, dis_pkpk)`

#### `get_fft_abs_data(np_data, sampling_rate)`
- Operations: `rfft`, `np.abs`, `np.angle`, `rfftfreq`
- Returns: `(fft_abs_data, freq_ticks, angle_data)` -- FFT magnitude, frequencies, phase angles

### `feature.py` -- 3 standalone functions

#### `time_domain_fea(signal_)` -- 13 features
Returns: max, min, mean, std(ddof=1), square-root amplitude, RMS, peak-peak, skewness, kurtosis, crest factor, clearance factor, shape factor, impulse factor

#### `freq_domain_fea(PL, f)` -- 12 features
Expects pre-computed FFT magnitude and frequency arrays. Returns spectral features f_12 through f_23 per Lei et al. (2007).

#### `both_Fea(signal_, PL, axis, f, create_time)`
Combines both feature sets with metadata dict.

### Operations Summary

| Operation | NumPy/SciPy | Count |
|-----------|-------------|-------|
| `np.fft.rfft` | Forward real FFT | 3 |
| `np.fft.irfft` | Inverse real FFT | 1 |
| `np.fft.rfftfreq` | Frequency bins | 3 |
| `np.abs` | Magnitude / absolute | 5 |
| `np.angle` | Phase angle | 1 |
| Boolean mask indexing | `arr[arr > val]` | 6 |
| `np.mean`, `np.sum`, `np.var`, `np.std` | Basic stats | ~10 |
| `scipy.stats.skew` | Skewness | 1 |
| `scipy.stats.kurtosis` | Kurtosis | 1 |
| `np.sqrt`, `np.max`, `np.min` | Element-wise | ~8 |

---

## 3. Recommended Architecture

### Cargo Workspace Layout

```
core_view_rs/
  Cargo.toml              # workspace root
  core/                   # Pure Rust DSP logic (no binding deps)
    Cargo.toml            # [lib] crate-type = ["rlib"]
    src/
      lib.rs
      vibration.rs        # Vibration methods
      feature.rs          # Time/freq domain features
      fft_utils.rs        # rfftfreq, mask helpers
  bindings-python/        # PyO3 + maturin + rust-numpy
    Cargo.toml            # [lib] crate-type = ["cdylib"]
    src/lib.rs
    pyproject.toml
  bindings-node/          # NAPI-RS v3
    Cargo.toml            # [lib] crate-type = ["cdylib"]
    src/lib.rs
    package.json
```

**Rationale**: Both binding crates depend on the shared `core` crate. This avoids code duplication and ensures identical computation regardless of which language calls it. PyO3 and NAPI-RS cannot coexist in the same crate.

---

## 4. Rust-to-Python Bindings

### 4.1 PyO3 Overview

- **Repository**: github.com/PyO3/pyo3 -- 15.4k stars
- **Latest**: v0.28.2 (February 2026), requires Rust 1.83+
- **Supports**: CPython 3.7+, PyPy 7.3+ (3.11+), GraalPy 25.0+ (3.12+)
- **Key macros**: `#[pyfunction]`, `#[pymodule]`, `#[pyclass]`
- **ABI3**: Build one wheel for multiple Python versions via `Py_LIMITED_API`

### 4.2 Comparison: PyO3 vs CFFI vs ctypes

| Criterion | PyO3 | CFFI | ctypes |
|-----------|------|------|--------|
| Performance | Fastest (0.14ms/call overhead) | Comparable to PyO3 | Slowest |
| Ergonomics | Best (`#[pyfunction]` macros) | Requires C header management | Manual everything |
| Numpy interop | rust-numpy (zero-copy) | Manual pointer handling | Manual |
| Build system | maturin (zero-config) | maturin or setuptools | stdlib |
| Ecosystem | Largest (Polars, ruff, pydantic) | Smaller | N/A |

**Academic benchmark** (University of Sao Paulo, arxiv.org/abs/2507.00264):
- PyO3 per-call overhead: **0.14ms** vs NumPy's 3.56ms (25x lower)
- PyO3 std deviation calc: 1,256ms vs NumPy's 1,594ms (PyO3 wins)
- Chunked execution: PyO3 advantage grows as chunk size decreases

### 4.3 rust-numpy (Zero-Copy NumPy Interop)

- **Repository**: github.com/PyO3/rust-numpy -- 1.3k stars
- **Latest**: v0.28.0 (February 2026)
- **Supports**: numpy >= 1.16

**Reading arrays (Python → Rust, zero-copy):**
```rust
use numpy::PyReadonlyArray1;
use pyo3::prelude::*;

#[pyfunction]
fn compute_rms(data: PyReadonlyArray1<f64>) -> f64 {
    let arr = data.as_array();  // zero-copy ArrayView1<f64>
    let sum_sq: f64 = arr.iter().map(|x| x * x).sum();
    (sum_sq / arr.len() as f64).sqrt()
}
```

**Returning arrays (Rust → Python, zero-copy):**
```rust
use numpy::PyArray1;
use ndarray::Array1;
use pyo3::prelude::*;

#[pyfunction]
fn get_fft_magnitude<'py>(py: Python<'py>, data: PyReadonlyArray1<f64>) -> &'py PyArray1<f64> {
    let result: Array1<f64> = core::fft_magnitude(data.as_array());
    PyArray1::from_owned_array(py, result)  // zero-copy ownership transfer
}
```

### 4.4 maturin Build System

- **Repository**: github.com/PyO3/maturin -- 5.5k stars
- **Latest**: v1.12.6 (March 2026)
- **Commands**: `maturin develop` (install to venv), `maturin build` (wheels), `maturin publish` (PyPI)

**Platform support**:
- Windows: x86, x64, ARM64 (integrates cargo-xwin for cross-compilation)
- Linux: x86, x86_64, ARMv7, AArch64, ppc64le, s390x (manylinux2014+, musllinux_1_1+)
- Cross-compilation via zig linker: `pip install ziglang`

**maturin vs setuptools-rust**:

| Feature | maturin | setuptools-rust |
|---------|---------|-----------------|
| Configuration | Zero/minimal | Requires setup.py mods |
| manylinux wheels | Out-of-the-box | Requires Docker |
| Best for | New Rust+Python projects | Adding Rust to existing packages |

### 4.5 Production Users of PyO3

- **Polars** -- DataFrame query engine (pyo3-polars crate)
- **pydantic-core** -- Core validation logic for pydantic
- **Ruff** -- Python linter
- **Rateslib** -- Fixed income library
- **River** -- Online machine learning (heavy stats in Rust)
- **Robyn** -- Async Python web framework with Rust runtime

---

## 5. Rust-to-Node.js Bindings

### 5.1 NAPI-RS Overview

- **Repository**: github.com/napi-rs/napi-rs -- 7.4k stars
- **Latest**: napi v3.8.2 (January 2026)
- **Major release**: NAPI-RS v3 (July 2025) -- four-year gap from v2

### 5.2 Comparison: NAPI-RS vs Neon vs wasm-bindgen vs FFI

**Benchmark (Apple M1 Max):**

| Operation | NAPI-RS | Neon | node-bindgen |
|-----------|---------|------|-------------|
| Sum (a + b) | 38.0M ops/s (100%) | 24.0M ops/s (62%) | 19.6M ops/s (52%) |
| String concat | 10.0M ops/s (100%) | 9.6M ops/s (97%) | 6.3M ops/s (63%) |
| Struct field | 4.3M ops/s (98%) | 3.6M ops/s (82%) | 2.5M ops/s (56%) |

**Performance ranking for Float64Array workloads:**

| Rank | Approach | Array Transfer | Best For |
|------|----------|---------------|----------|
| 1 | NAPI-RS (`&[f64]`) | Zero-copy | Synchronous DSP |
| 2 | NAPI-RS (owned `Float64Array`) | Ref-counted, no copy | Async DSP |
| 3 | Neon | Buffer via handles | Legacy projects |
| 4 | Koffi / ffi-rs | Manual pointers | Calling existing .dll/.so |
| 5 | WASM (wasm-bindgen) | Full copy both ways | Browser fallback only |

**WASM vs Native**: Native (NAPI-RS) ~8 GB/s vs WASM ~700 MB/s -- native is ~11x faster. WASM copies entire Float64Array on every call.

### 5.3 Zero-Copy Float64Array Interop

**Borrowed slice (synchronous, zero-copy):**
```rust
use napi_derive::napi;

#[napi]
pub fn process_signal(input: &[f64]) -> Vec<f64> {
    // `input` borrows JS Float64Array memory directly -- zero copy
    input.iter().map(|x| x * 2.0).collect()
}
```

**Owned type (async-safe, reference-counted):**
```rust
#[napi]
pub fn process_async(input: Float64Array) -> Float64Array {
    // Float64Array uses napi_ref wrapping, safe across threads
    // Mutations reflected back to JS
}
```

**Returning data (zero-copy ownership transfer):**
- `Vec<f64>` returned from Rust transfers ownership to V8, no copy
- V8's GC drops it when no longer referenced

### 5.4 Cross-Platform Build & Distribution

- `@napi-rs/cli` v2.13+: zero-config cross-build for 12 platform targets
- Default triples: `x86_64-pc-windows-msvc`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`
- Ships with **GitHub Actions CI templates** for automatic platform-specific npm packages
- No `node-gyp` required -- platform binaries as scoped npm packages with `optionalDependencies`

```json
{
  "napi": {
    "name": "core-view-rs",
    "triples": {
      "defaults": true,
      "additional": ["x86_64-unknown-linux-musl"]
    }
  },
  "optionalDependencies": {
    "@core-view-rs/win32-x64-msvc": "1.0.0",
    "@core-view-rs/linux-x64-gnu": "1.0.0"
  }
}
```

### 5.5 WASM Fallback Strategy

**NAPI-RS v3 dual-target** (July 2025): compile same code to both native and WASM (`wasm32-wasip1-threads`) with minimal changes. Use cases: browser playgrounds, StackBlitz, unsupported platforms.

**Limitations for DSP**: Mandatory copy on every typed array transfer. For 44.1 kHz stereo buffer (88,200 f64 = ~689 KB), copy cost ~0.1-0.3ms per call. 10x+ overhead vs native. **Use WASM as fallback only.**

### 5.6 Production Users of NAPI-RS

- **Rolldown** -- bundler (co-designed v3 ThreadsafeFunction API)
- **Rspack** -- bundler
- **Oxc** -- JavaScript parser/linter
- **SWC** -- JavaScript/TypeScript compiler

---

## 6. Rust DSP & Scientific Computing Ecosystem

### 6.1 FFT Crates

#### rustfft v6.4.1
- Pure Rust, SIMD-accelerated (AVX, SSE, Neon), any-size FFTs
- **Complex-to-complex only** -- not ideal for real-valued signals (wastes memory/compute)

#### realfft v3.5.0 -- RECOMMENDED for Vec-based work
- Wraps rustfft for real-valued FFT
- `RealToComplex`: N real → N/2+1 complex (= `np.fft.rfft`)
- `ComplexToReal`: N/2+1 complex → N real (= `np.fft.irfft`)

```rust
use realfft::RealFftPlanner;
let mut planner = RealFftPlanner::<f64>::new();
let r2c = planner.plan_fft_forward(length);
let mut input = r2c.make_input_vec();       // length N
let mut spectrum = r2c.make_output_vec();    // length N/2+1
r2c.process(&mut input, &mut spectrum).unwrap();
```

**CRITICAL**: realfft does NOT normalize output. After `c2r.process()`, divide each element by N to match `np.fft.irfft`.

#### ndrustfft v0.6.2 -- RECOMMENDED for ndarray-based work
- Wraps rustfft + realfft, operates on ndarray arrays directly
- Functions: `ndfft_r2c` / `ndifft_r2c` (+ parallel `_par` variants)
- **Applies scipy-style normalization by default** -- inverse output matches `np.fft.irfft` out of the box

| Scenario | Crate |
|----------|-------|
| Working with `Vec<f64>` | `realfft` |
| Working with `ndarray::Array` | `ndrustfft` |
| Complex-to-complex only | `rustfft` directly |

### 6.2 ndarray v0.17.2

- N-dimensional arrays: `Array` (owned), `ArrayView` (borrowed), `ArrayViewMut` (mutable)
- Fixed-dimension variants (1D-6D) and dynamic
- Pre-1.0 (0.17.x) but pace of breaking changes has slowed
- Requires `std` feature for `var_axis`, `std_axis`

### 6.3 ndarray-stats v0.7.0

Extension traits for ndarray:
- **SummaryStatisticsExt**: mean, skewness, kurtosis, central moments, weighted_mean, geometric_mean, harmonic_mean, weighted std with ddof
- **QuantileExt**: min, max, median, quantiles
- **CorrelationExt**: covariance, Pearson correlation
- **DeviationExt**: L1/L2 distances, MSE
- **HistogramExt**: histogram computation

### 6.4 num-complex v0.4.6

Stable, mature, universally adopted (used by rustfft, realfft, ndrustfft).

| Python (NumPy) | Rust (num-complex) |
|---|---|
| `np.abs(z)` | `z.norm()` |
| `np.angle(z)` | `z.arg()` |
| `z.real` / `z.imag` | `z.re` / `z.im` |
| `np.abs(complex_arr)` | `arr.mapv(\|z\| z.norm())` |
| `np.angle(complex_arr)` | `arr.mapv(\|z\| z.arg())` |
| -- | `z.norm_sqr()` -- `|z|^2` without sqrt (useful for PSD) |
| -- | `z.to_polar()` → `(r, theta)` |

### 6.5 Statistics: Skewness & Kurtosis

**ndarray-stats (recommended):**
```rust
use ndarray_stats::SummaryStatisticsExt;
let skew = array.skewness().unwrap();
let kurt = array.kurtosis().unwrap();
```

**CRITICAL kurtosis caveat:**
- ndarray-stats returns **Pearson's kurtosis** (normal = 3.0)
- scipy.stats.kurtosis defaults to **Fisher's** (normal = 0.0)
- To match scipy: `let fisher_kurtosis = array.kurtosis().unwrap() - 3.0;`

**Skewness bias**: ndarray-stats uses biased estimator (divides by N), matching scipy's default `bias=True`.

**Alternative crates**: `statrs` (distribution-focused), `scirs2-stats` (SciPy-compatible), `average` (moments).

---

## 7. NumPy/SciPy to Rust Operation Mapping

| NumPy / SciPy | Rust Equivalent | Notes |
|---|---|---|
| `np.fft.rfft(x)` | `r2c.process(&mut input, &mut spectrum)` | realfft or ndrustfft |
| `np.fft.irfft(X)` | `c2r.process(&mut spectrum, &mut output)` | realfft: manual /N; ndrustfft: auto |
| `np.fft.rfftfreq(n, d)` | Manual impl (see below) | Trivial 5-line function |
| `np.mean(a)` | `a.mean().unwrap()` | Returns `Option` |
| `np.sum(a)` | `a.sum()` | Direct |
| `np.var(a, ddof=0)` | `a.var_axis(Axis(0), 0.0)` | Requires `std` feature |
| `np.std(a, ddof=1)` | `a.std_axis(Axis(0), 1.0)` | ddof as f64 second arg |
| `np.max(a)` | `a.iter().cloned().fold(f64::NEG_INFINITY, f64::max)` | Or ndarray-stats QuantileExt |
| `np.min(a)` | Same pattern with `f64::min` | |
| `np.sqrt(a)` | `a.mapv(f64::sqrt)` | |
| `np.abs(a)` (real) | `a.mapv(f64::abs)` | |
| `np.abs(a)` (complex) | `a.mapv(\|z\| z.norm())` | ndarray + num-complex |
| `np.angle(a)` | `a.mapv(\|z\| z.arg())` | ndarray + num-complex |
| `scipy.stats.skew(a)` | `a.skewness().unwrap()` | ndarray-stats |
| `scipy.stats.kurtosis(a)` | `a.kurtosis().unwrap() - 3.0` | Adjust for Fisher |
| `a[a > thresh]` | `a.iter().filter(\|&&x\| x > thresh).cloned().collect()` | No native mask indexing |
| `a * b` (elementwise) | `&a * &b` | Operator overloading |
| `a ** 3` | `a.mapv(\|x\| x.powi(3))` | Via mapv |

### rfftfreq implementation

```rust
fn rfftfreq(n: usize, d: f64) -> Vec<f64> {
    let val = 1.0 / (n as f64 * d);
    (0..=(n / 2)).map(|i| i as f64 * val).collect()
}
```

### Boolean mask indexing workaround

```rust
// NumPy: result = a[a > threshold]
let result: Vec<f64> = a.iter()
    .filter(|&&x| x > threshold)
    .cloned()
    .collect();
let result = Array1::from(result);

// NumPy: a[a < lower] = 0
// Rust: in-place conditional zeroing
a.mapv_inplace(|x| if x < lower { 0.0 } else { x });
```

---

## 8. Known Gotchas & Pain Points

### Architecture

1. **Cannot combine PyO3 and NAPI-RS in the same crate** -- must use separate binding crates in a Cargo workspace sharing a common core library
2. **UniFFI does not officially support Node.js** -- only Kotlin, Swift, Python, Ruby. Not suitable for numeric array workloads

### PyO3 / rust-numpy

3. **ABI3 restrictions**: `#[text_signature]` not supported until Python 3.10+; `dict`/`weakref` on classes not until 3.9+
4. **rust-numpy ndarray version coupling**: depends on ndarray 0.15-0.16 (check compatibility with ndarray 0.17 via latest rust-numpy 0.28)
5. **GIL requirement**: Array views from `as_array()` are lifetime-bound to the GIL
6. **PyO3 workspace linking**: Core crate may have linking issues; separate binding crate is the solution

### NAPI-RS

7. **Electron zero-copy limitation**: Electron cannot create buffers zero-copy; NAPI-RS copies data instead
8. **Buffer performance overhead**: Documented in GitHub issue #1973 (fixed in v3.8.2)

### ndarray / DSP

9. **No boolean mask indexing** -- biggest ergonomic gap vs NumPy. Open issue: `rust-ndarray/ndarray#127`
10. **No rfftfreq built-in** -- trivial to implement manually
11. **realfft does NOT normalize** -- must manually divide by N for irfft to match NumPy
12. **ndrustfft auto-normalizes** -- matches scipy but differs from raw realfft
13. **Kurtosis convention mismatch** -- ndarray-stats returns Pearson's (normal=3.0); scipy defaults to Fisher's (normal=0.0). Subtract 3.0.
14. **No built-in max()/min() for f64 arrays** -- `f64` doesn't implement `Ord` (NaN). Use iterator patterns or ndarray-stats
15. **var/std axis-only** -- `var_axis`/`std_axis` require axis parameter; for whole-array, flatten first or compute manually
16. **ndarray is pre-1.0** (0.17.x) -- breaking changes possible between minor versions

---

## 9. Recommended Cargo Dependencies

### Core crate (`core/Cargo.toml`)

```toml
[package]
name = "core-view-core"
version = "0.1.0"
edition = "2021"

[dependencies]
ndarray = { version = "0.17", features = ["std"] }
ndarray-stats = "0.7"
num-complex = "0.4"
realfft = "3.5"
# Alternative: ndrustfft = "0.6"  (if using ndarray arrays throughout)
```

### Python binding crate (`bindings-python/Cargo.toml`)

```toml
[package]
name = "core-view-python"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
core-view-core = { path = "../core" }
pyo3 = { version = "0.28", features = ["extension-module"] }
numpy = "0.28"
```

### Node.js binding crate (`bindings-node/Cargo.toml`)

```toml
[package]
name = "core-view-node"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
core-view-core = { path = "../core" }
napi = { version = "3", features = ["default"] }
napi-derive = "3"

[build-dependencies]
napi-build = "2"
```

### Workspace root (`Cargo.toml`)

```toml
[workspace]
members = ["core", "bindings-python", "bindings-node"]
resolver = "2"
```

---

## Sources

- PyO3: https://github.com/PyO3/pyo3 (v0.28.2, Feb 2026)
- rust-numpy: https://github.com/PyO3/rust-numpy (v0.28.0, Feb 2026)
- maturin: https://github.com/PyO3/maturin (v1.12.6, Mar 2026)
- NAPI-RS: https://github.com/napi-rs/napi-rs (v3.8.2, Jan 2026)
- NAPI-RS v3 announcement: https://napi.rs/blog/announce-v3 (Jul 2025)
- realfft: https://docs.rs/realfft/3.5.0 (Jun 2025)
- ndrustfft: https://docs.rs/ndrustfft/0.6.2 (Jan 2026)
- ndarray: https://docs.rs/ndarray/0.17.2 (Jan 2026)
- ndarray-stats: https://docs.rs/ndarray-stats/0.7.0 (Dec 2025)
- num-complex: https://docs.rs/num-complex/0.4.6 (May 2024)
- Benchmark (PyO3 vs NumPy): https://arxiv.org/abs/2507.00264
- Benchmark (NAPI-RS vs Neon): https://github.com/Brooooooklyn/rust-to-nodejs-overhead-benchmark
- rustfft: https://docs.rs/rustfft/6.4.1 (Sep 2025)
