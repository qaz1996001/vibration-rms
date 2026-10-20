# CLAUDE.md

**This file provides internal Claude Code development guidance for contributors. For the public project overview, build instructions, and API documentation, see [README.md](README.md).**

This file is a working reference for understanding the architecture, common patterns, and error handling specific to this repository's implementation discipline. It is not a public-facing document.

---

## Quick Architecture Recap

The project is a Rust port of a NumPy/SciPy vibration-analysis library, with Python (PyO3) and Node.js (NAPI-RS) bindings.

```
┌──────────────────┐
│ Python reference │ ← NumPy/SciPy ground truth
│ (vibration.py,   │   (kept in repo for golden data)
│  feature.py)     │
└────────┬─────────┘
         │ generate_golden_data.py
         ▼
   ┌─────────────────┐
   │ golden_data.json│ ← committed, rtol=1e-6 target
   └────────┬────────┘
            │
     ┌──────┴──────┐
     ▼             ▼
  ┌───────┐    ┌─────────────────┐
  │ Rust  │    │ Python/Node      │
  │ core  │ ← validates against golden data
  └───────┘    └─────────────────┘
```

Workspace structure:
- `core_view_rs/core/` — `vibration-rms-core` crate (pure Rust, no FFI)
- `core_view_rs/bindings-python/` — `vibration-rms-python` (PyO3, produces `vibration_rms.pyd`/`.so`)
- `core_view_rs/bindings-node/` — `vibration-rms-node` (NAPI-RS, produces `.node` binary)
- `scripts/golden_data.json` — golden reference outputs (committed to git, read by all three test suites)

---

## Development Commands

### Rust
```bash
cd core_view_rs

# Format
cargo fmt --all
# Lint
cargo clippy --all-targets --all-features -- -D warnings
# Test (includes golden_cross_validation.rs)
cargo test -p vibration-rms-core
# Just core tests
cargo test --lib
```

### Python
```bash
cd core_view_rs/bindings-python
uv sync
uv run maturin develop --release
uv run python test_bindings.py
```

### Node.js
```bash
cd core_view_rs/bindings-node
npm install
npm run build
node test_bindings.mjs
```

---

## Verification Checklist

After modifying `core/` or bindings:

**Rust side:**
- [ ] `cargo fmt --all -- --check` ✓
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` ✓
- [ ] `cargo test -p vibration-rms-core` (golden_cross_validation.rs must pass) ✓

**If modifying Python reference (`vibration.py` or `feature.py`):**
- [ ] `python scripts/generate_golden_data.py` (regenerate golden)
- [ ] Commit the updated `golden_data.json`
- [ ] All three test suites will re-validate against the new golden

**If modifying Python bindings:**
- [ ] `cd core_view_rs/bindings-python && uv run maturin develop && uv run python test_bindings.py` ✓

**If modifying Node bindings:**
- [ ] `cd core_view_rs/bindings-node && npm run build && node test_bindings.mjs` ✓

---

## Golden Data Cross-Validation

**This is the core engineering discipline of the project.**

All Rust/Python/Node outputs must match the NumPy/SciPy reference at `rtol=1e-6`. The golden data is committed to git so it serves as a stable, reproducible ground truth.

**If golden tests fail:**
1. Do NOT modify `golden_data.json` to make tests pass.
2. Compare the Rust implementation against the Python original line-by-line.
3. Look for:
   - FFT normalization mismatches (NumPy divides by `n` on inverse; RustFFT's inverse is not normalized)
   - Frequency bin handling (DC component, Nyquist)
   - Feature calculation edge cases (division by zero, NaN handling)
   - Boundary condition differences

**If the Python implementation itself changes:**
1. Update `vibration.py` / `feature.py`.
2. Regenerate golden: `python scripts/generate_golden_data.py`
3. Commit the new `golden_data.json`.
4. All test suites automatically re-validate.

---

## Anti-Patterns & Known Issues

| Date | ❌ What went wrong | ✅ Solution |
|------|------------------|-----------|
| 2026-03-12 | Using C++ build tools for Node.js native module | Use napi-rs (pure Rust), not node-gyp |
| 2026-03-12 | Setting `msvs_version` via `npm config` | Not supported in modern npm; use `GYP_MSVS_VERSION=2022` env var if needed |
| Early port | Direct `realfft` output without `/n` normalization | Divide inverse-FFT output by `n` to match NumPy's `irfft` behavior |
| Early port | Attempting ndarray boolean mask filtering for bandpass | ndarray doesn't support it; use `fft_utils::bandpass_range` to get index ranges instead |
| Early port | Leaving `freq_ticks[0]` as-is in `dis_pkpk` calculation | Set `freq_ticks[0] = 1.0` to avoid division by zero; set DC displacement to 0 |

### Numpy ↔ Rust FFT Normalization

```rust
// ❌ Wrong: direct realfft inverse
let mut output = ifft.make_output_vec();
ifft.process(&mut spec, &mut output).unwrap();
return output;  // NumPy irfft expects /n, but this doesn't divide

// ✅ Correct: match NumPy's normalization
for v in output.iter_mut() { *v /= n as f64; }
```

---

## Common Feature Patterns

### Pattern: Dual naming (English names + Lei et al. paper IDs)

Both bindings (Python and Node) support:
```
core_view.time_domain_features(signal, features=['rms', 'kurtosis'])
core_view.time_domain_features(signal, features=['t4', 't7'])  # equivalent
```

Implementation: `core/src/feature.rs` has `TdFeature::from_name()` and `FdFeature::from_name()` that map both English names and paper IDs to enums. The binding layers use these same helpers (`parse_td_features` in Python, `parse_td_features` in Node).

### Pattern: Selective feature computation

When the caller specifies feature names, use the `*_select` variants from `core/src/feature.rs`:
```rust
core::time_domain_features_select(&signal, &[TdFeature::Rms, TdFeature::Kurtosis])
```
This avoids computing unused features, improving performance.

### Pattern: NumPy zero-copy via PyO3

```rust
// ✅ Correct: PyReadonlyArray1 for input (no copy)
fn time_domain_features<'py>(
    py: Python<'py>,
    signal: PyReadonlyArray1<'py, f64>,
    ...
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let slice = signal.as_slice().unwrap();  // zero-copy view
    ...
}
```

Do not use `Vec<f64>` for input arrays — that forces NumPy to copy the entire array.

---

## Error Handling

This is a pure compute library (no I/O, no services). There are no custom exception types.

- **Rust core**: functions return values directly; internal `unwrap()` is only on FFT planner initialization (which cannot realistically fail).
- **Python bindings**: `ValueError` is raised for unknown feature names via `parse_td_features`/`parse_fd_features`.
- **Node bindings**: napi-rs automatically converts panics to JavaScript `Error`. The binding layer uses `napi::Error` with `Status::InvalidArg` for unknown feature names.

### Common error scenarios

| Error | Cause | Fix |
|-------|-------|-----|
| `ValueError: Unknown time-domain feature: 'xxx'` | Typo in feature name string | Use `t1`–`t11` or English names (`rms`, `kurtosis`, etc.) |
| `golden_data.json not found` | Missing call to `generate_golden_data.py` | `python scripts/generate_golden_data.py` before running tests |
| NaN in `dis_pkpk` | DC component (freq_ticks[0] = 0) not handled | Check `vibration.py` line 18: `freq_ticks[0] = 1.0` |
| FFT magnitude mismatch with golden | Not dividing inverse-FFT output by `n` | Apply `/n` normalization to match NumPy `irfft` |

---

## Debug Commands

```bash
# Detailed Rust backtrace and logging
cd core_view_rs
RUST_BACKTRACE=full cargo test -p vibration-rms-core -- --nocapture

# Single golden test
cargo test -p vibration-rms-core --test golden_cross_validation acceleration_rms -- --nocapture

# Python bindings: maturin verbose output
cd core_view_rs/bindings-python
uv run maturin develop --verbose

# Confirm .pyd/.so import
uv run python -c "import vibration_rms; print(vibration_rms.__file__)"

# Node.js bindings: inspect exported functions
cd core_view_rs/bindings-node
node -e "console.log(Object.keys(require('./index.js')))"
```

---

## Key Files Reference

| Purpose | File |
|---------|------|
| Workspace config | `core_view_rs/Cargo.toml` |
| Core Rust lib | `core_view_rs/core/src/{lib,vibration,feature,fft_utils}.rs` |
| Core Cargo | `core_view_rs/core/Cargo.toml` |
| Golden cross-validation | `core_view_rs/core/tests/golden_cross_validation.rs` |
| Python bindings Rust | `core_view_rs/bindings-python/src/lib.rs` |
| Python binding config | `core_view_rs/bindings-python/{Cargo.toml,pyproject.toml,core_view.pyi}` |
| Python tests | `core_view_rs/bindings-python/test_bindings.py` |
| Node bindings Rust | `core_view_rs/bindings-node/src/lib.rs` |
| Node binding config | `core_view_rs/bindings-node/{Cargo.toml,package.json,build.rs}` |
| Node tests | `core_view_rs/bindings-node/test_bindings.mjs` |
| Golden data | `core_view_rs/scripts/golden_data.json` |
| Golden data generation | `core_view_rs/scripts/generate_golden_data.py` |
| Python reference (keep) | `vibration.py`, `feature.py` |

---

## References

- Lei, Y., He, Z., Zi, Y., & Hu, Q. (2007). *Fault diagnosis of rotating machinery based on multiple ANFIS combination with GAs.* Mechanical Systems and Signal Processing, 21(5), 2280–2294.
  - Feature set source and naming convention (`T₁–T₁₁`, `F₁–F₁₂`)
- [`claudedocs/research_rust_ffi_bindings_20260309.md`](claudedocs/research_rust_ffi_bindings_20260309.md)
  - FFI binding architecture deep-dive (PyO3 vs alternatives, NAPI-RS notes, FFT normalization)

---

**Last updated**: 2026-10-20  
**Primary language**: Rust 1.70+ with Python/Node.js FFI layers  
**Platform**: Windows 11, Linux (Ubuntu)
